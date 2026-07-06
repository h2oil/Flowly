# Data, Script Format & Online Sync

## Summary

Local-first **SQLite** store on Windows; hand-rolled **revision-based document sync** (whole-doc snapshots + immutable version history) against **Supabase** (hosted Postgres + Auth + Realtime + RLS). Conflict policy v1: **rev-checked last-writer-wins with a "conflicted copy" fork** — never a silent overwrite, never data loss (automatic 3-way merge is a v1.1 upgrade). CRDTs (Yjs) deliberately deferred: v1 has no concurrent co-editing requirement, and the plain-text canonical format converts cleanly later. First run is **account-free and fully local**; sign-in is an additive "claim," not a migration.

## Script document format — "Flowly Prompter Markdown"

Plain UTF-8 Markdown subset, not a JSON AST (diffable, versionable, portable, CRDT-able later, pasteable from anywhere):

```
# Intro                          ← heading = section (nav/jump target)
Welcome back — today we're shipping Flowly. //pause
[SLOW] The one feature you asked for: **voice following**.
(( ad-lib the demo joke here ))  ← unspoken stage note, shown dim
[PAUSE 2s]
```

Directives: `//pause`, `[PAUSE 2s]`, `[SLOW]/[FAST]/[NORMAL]`, `**bold**`, `(( unspoken notes ))`, `#` sections. **The load-bearing contract:** the format compiles deterministically to a spoken-token stream with a source map — `tokens[] = {word, variants[], charStart, charEnd}` — with directives and `(( ))` notes excluded from alignment (notes act as ad-lib wildcards). Renderer, aligner, and take markers share this one coordinate system. Compiled token maps are **cached keyed by `body_hash`** so re-arming an unchanged script is instant; any grammar change invalidates caches by construction because the hash changes.

## Local data layer

One SQLite file at `%APPDATA%\Flowly\flowly.db`, WAL mode, via rusqlite — synchronous, in-process, no ORM. All IDs **UUIDv7** (time-ordered, globally unique → local rows upload without re-keying).

```sql
CREATE TABLE folder (
  id TEXT PRIMARY KEY, parent_id TEXT REFERENCES folder(id),
  name TEXT NOT NULL, sort_order INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL,
  deleted_at INTEGER,                  -- soft delete; hard-purge after sync
  base_rev INTEGER NOT NULL DEFAULT 0, dirty INTEGER NOT NULL DEFAULT 0);

CREATE TABLE script (
  id TEXT PRIMARY KEY, folder_id TEXT REFERENCES folder(id),
  title TEXT NOT NULL DEFAULT 'Untitled',
  body TEXT NOT NULL DEFAULT '',       -- Flowly Prompter Markdown, UTF-8
  body_hash TEXT NOT NULL,             -- sha256; cheap change detection + token-map cache key
  language TEXT NOT NULL DEFAULT 'en', wpm_target INTEGER,
  created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, deleted_at INTEGER,
  base_rev INTEGER NOT NULL DEFAULT 0, -- server rev this copy derives from
  dirty INTEGER NOT NULL DEFAULT 0,
  encrypted INTEGER NOT NULL DEFAULT 0); -- E2E "vault" flag (v1.x)

CREATE TABLE script_version (          -- immutable, append-only, synced
  id TEXT PRIMARY KEY, script_id TEXT NOT NULL REFERENCES script(id),
  rev INTEGER NOT NULL, body TEXT NOT NULL,
  label TEXT,                          -- user pin, e.g. "Course take v2"
  created_by_device TEXT, created_at INTEGER NOT NULL,
  UNIQUE(script_id, rev));

CREATE TABLE take_marker (             -- device-local in v1 (not synced)
  id TEXT PRIMARY KEY, script_id TEXT NOT NULL,
  script_version_id TEXT NOT NULL,     -- markers pin to a FROZEN version
  char_start INTEGER NOT NULL, char_end INTEGER,
  kind TEXT NOT NULL CHECK(kind IN ('take_start','take_end','bookmark','stumble')),
  note TEXT, created_at INTEGER NOT NULL);

CREATE TABLE setting (                 -- 'bar.position.<monitorId>' → JSON, etc.
  key TEXT PRIMARY KEY, value TEXT NOT NULL,
  updated_at INTEGER NOT NULL, synced INTEGER NOT NULL DEFAULT 0);

CREATE TABLE outbox (                  -- push queue, drained in order
  seq INTEGER PRIMARY KEY AUTOINCREMENT,
  entity TEXT NOT NULL, entity_id TEXT NOT NULL,
  op TEXT NOT NULL, queued_at INTEGER NOT NULL);
```

Key move for retakes: **entering prompter mode freezes the current body as a `script_version`** (two lines of code, ships in v1 even though the take-marker UI may come later); take markers reference the frozen version + char offsets, so later edits can never shift or corrupt them — "jump back to re-record take 3" is always exact. Bar positions live in `setting` with `synced=0`: positions are device-specific; syncing them would be a bug.

## Backend: Supabase

| Option | Magic link + Google/Microsoft | Realtime | Ops burden | Verdict |
|---|---|---|---|---|
| **Supabase** | Built-in incl. **Azure AD** | Postgres changes + broadcast | ~zero (Pro $25/mo) | ✅ Pick |
| Firebase | Yes | Firestore listeners | ~zero | Doc model fights rev-based versioning; GCP lock-in |
| PocketBase (self-host) | OAuth yes; magic link DIY | SSE | You run VPS/backups/SMTP; pre-1.0 | Best exit hatch, wrong trade for a small team |
| Custom API | Roll your own | Roll your own | High | Not for v1 |
| CRDT stack (Yjs/Automerge) | Separate auth anyway | Native | Medium | Solves co-editing v1 doesn't have |

Supabase wins on exactly what day one needs: **email magic link + Google + Azure AD OAuth** (the exec/sales audience lives in M365), **RLS** (`owner_id = auth.uid()` on every table = one-line multi-tenancy), Realtime as a doorbell, and plain portable Postgres underneath (credible exit to self-hosted Supabase or a custom API). PowerSync/ElectricSQL rejected for v1: doc-centric script sync is ~500 LOC we fully control.

**Desktop auth:** system browser + PKCE + `flowly://auth-callback` deep link — never an embedded webview (Google blocks them; corporate IT distrusts them). **Custom SMTP (Resend/Postmark) from day one** — Supabase default SMTP is rate-limited and lands in spam. Note: Supabase free-tier projects pause after 7 days idle; production runs on Pro ($25/mo) from launch. Pin the project to **eu-west-2 (London)** for the UK-GDPR posture (see [07-operations.md](07-operations.md)).

## Conflict handling (v1)

Server holds `rev BIGINT` per script, bumped by trigger. Push sends `{id, base_rev, body, …}`:

1. `base_rev == server rev` → accept, `rev++`, append `script_version`, return new rev.
2. Mismatch → **409** with the server copy → client immediately forks: *"Keynote (conflicted copy — Desktop, 6 Jul)"*, keeps both, badges it in the UI.

Lossless, ~zero code risk, and the canonical conflict (web edit while desktop offline) is rare for single-author scripts. **v1.1:** attempt automatic 3-way merge first (diff-match-patch against the common-ancestor `script_version`), forking only on overlapping edits. Full CRDT is a v2 upgrade only if live co-editing ships. Watch item: habitual two-machine offline editors can breed conflicted copies — the v1.1 merge plus a compare UI is the answer if beta telemetry shows it.

## Sync protocol

- **Push:** 2 s debounce after edit; on save, network regain, app quit. Drain `outbox` in order.
- **Pull:** app launch; window focus; realtime nudge; 60 s timer fallback; and **always immediately before entering prompter mode** — never read a stale exec speech. Prompter mode itself reads only the frozen local `script_version`, so sync can never move text under a live take.
- **Change feed:** cursor-based `GET /changes?since={sync_seq}` on a per-user monotonic Postgres sequence (not timestamps — clock skew).
- **Realtime = doorbell only:** one channel per user; server broadcasts `{"changed":"script:<id>"}`; client pulls over HTTPS. Payloads never ride the websocket (keeps message sizes trivial and the E2E option protocol-clean).
- **Whole-doc, not deltas:** scripts are 1–50 KB; gzip the body and stop.
- **History policy:** append `script_version` on rev bump; dedupe by `body_hash`; coalesce within a 5-minute editing session; explicit "Pin this version"; prune unpinned >90 days keeping last 20 — **enforced in v1, not later**.
- **Devices:** a `device` table (UUID minted at install, DPAPI-protected; name, last_seen, app_version) powers "sign out that device" and labels conflict forks.

## Account-free mode

First run = fully local, no sign-up wall, everything except sync works. Because IDs are already UUIDv7 and rows carry `base_rev`/`dirty`, **"Sign in" is just a claim**: set `owner_id`, enqueue all rows, push through the normal protocol. No migration wizard. Sign-out keeps local data; account deletion wipes the server and leaves local intact.

## Script ingestion — the authoring funnel

Real scripts arrive as Word docs, Google Docs, Notion pages, and messy pastes. Ingestion is the front half of the compiler:

1. **Paste (~80 % of usage).** Read both `text/html` and `text/plain` clipboard flavors; sanitize HTML (allowlist `h1-h3, p, b, i, ul/ol, br`) and convert to Prompter Markdown; headings become section markers the aligner uses as resync anchors.
2. **.docx** — covers Word *and* Google Docs/Notion via "Download as .docx" (mammoth.js with a style map; comments/track-changes stripped). Offline, no server round-trip.
3. **.txt / .md / .rtf** — direct.
4. **.pdf** — degraded with an explicit "review before recording" banner; reject scanned PDFs with a clear error rather than shipping OCR in v1.

**Paste normalization (deterministic, golden-file-tested in CI):** Unicode NFKC; smart quotes → straight; em/en-dash and ellipsis preserved as pause hints; strip zero-width chars/soft hyphens; NBSP → space; collapse whitespace; speaker labels like `JAMES:` become non-spoken stage-direction tokens.

**Verbalization layer (feeds the TokenMap — each written token compiles to 1–8 spoken variants):** numbers/currency/years via num2words-style rules (`$3.5M` → {"three point five million dollars", "three and a half million dollars", …}; `2024` → {"twenty twenty-four", "two thousand twenty-four"}); ~500-entry pronounceable-acronym dictionary (NASA, SaaS), unknown all-caps ≤5 chars expand letter-by-letter (`SQL` → {"S Q L", "sequel"}); symbols/units (%, &, km/h, phone numbers digit-by-digit, URLs "dot com"); ordinals/dates.

**Acceptance:** 25 real scripts (10 .docx, 5 Notion pastes, 5 PDFs, 5 raw) import with zero manual fixes on 22+; a numbers-heavy sales script ("$3.5M ARR, 47 % YoY, Q3 2026") aligns end-to-end with no stall >1 token.

## Privacy

- **In transit:** TLS 1.3. **At rest (server):** AES-256 + RLS isolation. **At rest (client):** SQLite under the user profile; optional DPAPI-wrapped key/SQLCipher in settings.
- **Audio never leaves the device** — on-device ASR, and **no audio-upload code path exists in the v1 binary** (the cloud-ASR trait slot stays empty until a deliberate opt-in build). This is an auditable architectural invariant and the headline privacy claim.
- **Optional E2E "Vault scripts" (v1.x, designed now):** per-script XChaCha20-Poly1305 (libsodium), Argon2id passphrase KDF; server stores ciphertext only; web editor decrypts in-browser. Honest trade-offs surfaced in UI: no server-side preview/search, no recovery if the passphrase is lost. The doorbell-only realtime and whole-doc protocol mean E2E slots in without protocol changes.
- **Telemetry:** opt-in, count-only; script bodies/titles never transmitted; crash dumps scrubbed (see [07-operations.md](07-operations.md)).
