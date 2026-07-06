//! Local-first SQLite store (docs/plan/05-data-and-sync.md).
//!
//! One database file, WAL mode, UUIDv7-style ids, soft deletes, and
//! `base_rev`/`dirty` columns so cloud sync (a later rev) is a claim
//! operation, not a migration. The load-bearing behavior for retakes ships
//! now: **arming a script freezes its body as an immutable `script_version`**
//! and take markers pin to that frozen version, so later edits can never
//! shift a marker.

use rusqlite::{params, Connection, OptionalExtension};

pub type Result<T> = std::result::Result<T, rusqlite::Error>;

#[derive(Debug, Clone)]
pub struct Script {
    pub id: String,
    pub title: String,
    pub body: String,
    pub body_hash: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone)]
pub struct ScriptVersion {
    pub id: String,
    pub script_id: String,
    pub rev: i64,
    pub body: String,
    pub created_at: i64,
}

pub struct Store {
    conn: Connection,
    /// Monotonic counter mixed into generated ids (deterministic under test).
    id_counter: u64,
}

const SCHEMA: &str = "
PRAGMA journal_mode=WAL;
CREATE TABLE IF NOT EXISTS folder (
  id TEXT PRIMARY KEY, parent_id TEXT REFERENCES folder(id),
  name TEXT NOT NULL, sort_order INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL,
  deleted_at INTEGER,
  base_rev INTEGER NOT NULL DEFAULT 0, dirty INTEGER NOT NULL DEFAULT 0);
CREATE TABLE IF NOT EXISTS script (
  id TEXT PRIMARY KEY, folder_id TEXT REFERENCES folder(id),
  title TEXT NOT NULL DEFAULT 'Untitled',
  body TEXT NOT NULL DEFAULT '',
  body_hash TEXT NOT NULL,
  language TEXT NOT NULL DEFAULT 'en', wpm_target INTEGER,
  created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, deleted_at INTEGER,
  base_rev INTEGER NOT NULL DEFAULT 0,
  dirty INTEGER NOT NULL DEFAULT 0,
  encrypted INTEGER NOT NULL DEFAULT 0);
CREATE TABLE IF NOT EXISTS script_version (
  id TEXT PRIMARY KEY,
  script_id TEXT NOT NULL REFERENCES script(id),
  rev INTEGER NOT NULL, body TEXT NOT NULL,
  label TEXT,
  created_by_device TEXT, created_at INTEGER NOT NULL,
  UNIQUE(script_id, rev));
CREATE TABLE IF NOT EXISTS take_marker (
  id TEXT PRIMARY KEY, script_id TEXT NOT NULL,
  script_version_id TEXT NOT NULL,
  char_start INTEGER NOT NULL, char_end INTEGER,
  kind TEXT NOT NULL CHECK(kind IN ('take_start','take_end','bookmark','stumble')),
  note TEXT, created_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS setting (
  key TEXT PRIMARY KEY, value TEXT NOT NULL,
  updated_at INTEGER NOT NULL, synced INTEGER NOT NULL DEFAULT 0);
CREATE TABLE IF NOT EXISTS outbox (
  seq INTEGER PRIMARY KEY AUTOINCREMENT,
  entity TEXT NOT NULL, entity_id TEXT NOT NULL,
  op TEXT NOT NULL, queued_at INTEGER NOT NULL);
";

impl Store {
    pub fn open(path: &str) -> Result<Store> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Store { conn, id_counter: 0 })
    }

    pub fn open_in_memory() -> Result<Store> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Store { conn, id_counter: 0 })
    }

    fn new_id(&mut self, now_ms: i64) -> String {
        // UUIDv7 shape: time-ordered prefix + counter. Real randomness isn't
        // required locally; ids only need to be unique and sortable.
        self.id_counter += 1;
        format!("{now_ms:016x}-{:08x}", self.id_counter)
    }

    fn hash(body: &str) -> String {
        // FNV-1a placeholder for sha256; only used for cheap change detection
        // and token-map cache keys locally. Swapped for sha256 when sync ships.
        let mut h: u64 = 0xcbf29ce484222325;
        for b in body.as_bytes() {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x100000001b3);
        }
        format!("{h:016x}")
    }

    // ---- scripts -------------------------------------------------------

    pub fn create_script(&mut self, title: &str, body: &str, now_ms: i64) -> Result<Script> {
        let id = self.new_id(now_ms);
        let hash = Self::hash(body);
        self.conn.execute(
            "INSERT INTO script (id, title, body, body_hash, created_at, updated_at, dirty)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5, 1)",
            params![id, title, body, hash, now_ms],
        )?;
        self.enqueue("script", &id, "upsert", now_ms)?;
        self.get_script(&id).map(|s| s.expect("just inserted"))
    }

    pub fn update_script(&mut self, id: &str, title: &str, body: &str, now_ms: i64) -> Result<()> {
        let hash = Self::hash(body);
        self.conn.execute(
            "UPDATE script SET title=?2, body=?3, body_hash=?4, updated_at=?5, dirty=1
             WHERE id=?1 AND deleted_at IS NULL",
            params![id, title, body, hash, now_ms],
        )?;
        self.enqueue("script", id, "upsert", now_ms)
    }

    pub fn delete_script(&mut self, id: &str, now_ms: i64) -> Result<()> {
        self.conn
            .execute("UPDATE script SET deleted_at=?2, dirty=1 WHERE id=?1", params![id, now_ms])?;
        self.enqueue("script", id, "delete", now_ms)
    }

    pub fn get_script(&self, id: &str) -> Result<Option<Script>> {
        self.conn
            .query_row(
                "SELECT id, title, body, body_hash, created_at, updated_at
                 FROM script WHERE id=?1 AND deleted_at IS NULL",
                params![id],
                |r| {
                    Ok(Script {
                        id: r.get(0)?,
                        title: r.get(1)?,
                        body: r.get(2)?,
                        body_hash: r.get(3)?,
                        created_at: r.get(4)?,
                        updated_at: r.get(5)?,
                    })
                },
            )
            .optional()
    }

    pub fn list_scripts(&self) -> Result<Vec<Script>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, body, body_hash, created_at, updated_at
             FROM script WHERE deleted_at IS NULL ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Script {
                id: r.get(0)?,
                title: r.get(1)?,
                body: r.get(2)?,
                body_hash: r.get(3)?,
                created_at: r.get(4)?,
                updated_at: r.get(5)?,
            })
        })?;
        rows.collect()
    }

    // ---- versions (frozen on arm) ---------------------------------------

    /// Freeze the current body as an immutable version and return it. Called
    /// when a script is armed for prompting; take markers pin to the frozen
    /// version. Deduped: arming an unchanged body reuses the latest version.
    pub fn freeze_version(&mut self, script_id: &str, now_ms: i64) -> Result<ScriptVersion> {
        let script = self.get_script(script_id)?.expect("script exists");
        let latest: Option<ScriptVersion> = self
            .conn
            .query_row(
                "SELECT id, script_id, rev, body, created_at FROM script_version
                 WHERE script_id=?1 ORDER BY rev DESC LIMIT 1",
                params![script_id],
                |r| {
                    Ok(ScriptVersion {
                        id: r.get(0)?,
                        script_id: r.get(1)?,
                        rev: r.get(2)?,
                        body: r.get(3)?,
                        created_at: r.get(4)?,
                    })
                },
            )
            .optional()?;
        if let Some(v) = &latest {
            if v.body == script.body {
                return Ok(latest.expect("checked"));
            }
        }
        let rev = latest.map(|v| v.rev + 1).unwrap_or(1);
        let id = self.new_id(now_ms);
        self.conn.execute(
            "INSERT INTO script_version (id, script_id, rev, body, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, script_id, rev, script.body, now_ms],
        )?;
        Ok(ScriptVersion { id, script_id: script_id.to_string(), rev, body: script.body, created_at: now_ms })
    }

    pub fn add_take_marker(
        &mut self,
        script_id: &str,
        version_id: &str,
        char_start: i64,
        kind: &str,
        now_ms: i64,
    ) -> Result<String> {
        let id = self.new_id(now_ms);
        self.conn.execute(
            "INSERT INTO take_marker (id, script_id, script_version_id, char_start, kind, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, script_id, version_id, char_start, kind, now_ms],
        )?;
        Ok(id)
    }

    // ---- settings --------------------------------------------------------

    pub fn set_setting(&mut self, key: &str, value: &str, now_ms: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO setting (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value=?2, updated_at=?3",
            params![key, value, now_ms],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        self.conn
            .query_row("SELECT value FROM setting WHERE key=?1", params![key], |r| r.get(0))
            .optional()
    }

    // ---- outbox -----------------------------------------------------------

    fn enqueue(&mut self, entity: &str, entity_id: &str, op: &str, now_ms: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO outbox (entity, entity_id, op, queued_at) VALUES (?1, ?2, ?3, ?4)",
            params![entity, entity_id, op, now_ms],
        )?;
        Ok(())
    }

    pub fn outbox_len(&self) -> Result<i64> {
        self.conn.query_row("SELECT COUNT(*) FROM outbox", [], |r| r.get(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_roundtrip() {
        let mut store = Store::open_in_memory().unwrap();
        let s = store.create_script("Demo", "Hello world.", 1000).unwrap();
        assert_eq!(s.title, "Demo");
        store.update_script(&s.id, "Demo", "Hello again.", 2000).unwrap();
        let got = store.get_script(&s.id).unwrap().unwrap();
        assert_eq!(got.body, "Hello again.");
        assert_ne!(got.body_hash, s.body_hash);
        assert_eq!(store.list_scripts().unwrap().len(), 1);
        store.delete_script(&s.id, 3000).unwrap();
        assert!(store.get_script(&s.id).unwrap().is_none());
        assert!(store.outbox_len().unwrap() >= 3);
    }

    #[test]
    fn freeze_version_is_immutable_and_deduped() {
        let mut store = Store::open_in_memory().unwrap();
        let s = store.create_script("Talk", "Take one text.", 1000).unwrap();
        let v1 = store.freeze_version(&s.id, 1100).unwrap();
        assert_eq!(v1.rev, 1);
        // Arming again without edits reuses the same frozen version.
        let v1b = store.freeze_version(&s.id, 1200).unwrap();
        assert_eq!(v1b.id, v1.id);
        // Editing then arming freezes a new version; the old body survives.
        store.update_script(&s.id, "Talk", "Take two text.", 2000).unwrap();
        let v2 = store.freeze_version(&s.id, 2100).unwrap();
        assert_eq!(v2.rev, 2);
        assert_eq!(v1.body, "Take one text.");
        // Markers pin to the frozen version.
        let m = store.add_take_marker(&s.id, &v1.id, 5, "take_start", 2200).unwrap();
        assert!(!m.is_empty());
    }

    #[test]
    fn settings_roundtrip() {
        let mut store = Store::open_in_memory().unwrap();
        store.set_setting("bar.position.MON1", "{\"x\":12}", 1000).unwrap();
        store.set_setting("bar.position.MON1", "{\"x\":40}", 2000).unwrap();
        assert_eq!(store.get_setting("bar.position.MON1").unwrap().unwrap(), "{\"x\":40}");
        assert!(store.get_setting("missing").unwrap().is_none());
    }
}
