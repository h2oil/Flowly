// End-to-end verification: imports (.docx / .pdf / .txt / HTML paste) and the
// LIVE voice path (WASM aligner in the browser fed by a scripted fake
// SpeechRecognition — real mics can't run headless). Run with:
//   npm run build && node e2e/run.mjs [path-to-playwright-module]
// Exits non-zero on any failure.

import { createServer } from "node:http";
import { readFileSync, existsSync } from "node:fs";
import { extname, join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const dist = join(here, "..", "dist");
const pwPath = process.argv[2] ?? "playwright";
const { chromium } = await import(pwPath);

const MIME = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".mjs": "text/javascript",
  ".css": "text/css",
  ".wasm": "application/wasm",
};
const server = createServer((req, res) => {
  const path = req.url === "/" ? "/index.html" : req.url.split("?")[0];
  const file = join(dist, path);
  if (!existsSync(file)) {
    res.writeHead(404);
    res.end();
    return;
  }
  res.writeHead(200, { "content-type": MIME[extname(file)] ?? "application/octet-stream" });
  res.end(readFileSync(file));
});
await new Promise((r) => server.listen(4574, r));

let failures = 0;
const check = (name, ok, detail = "") => {
  console.log(`${ok ? "PASS" : "FAIL"}  ${name}${detail ? ` — ${detail}` : ""}`);
  if (!ok) failures++;
};

// The words the fake recognizer "hears" — the sample script read aloud with
// fillers, two misrecognitions, and an off-script ad-lib in the middle.
const SPOKEN =
  ("welcome back everyone and thanks for joining the flowly launch demo today " +
    "we built a teleprompter that um actually listens " +
    "as you speak the text moves with your voise through pauses stumbles and retakes " +
    // ad-lib: off-script words — the engine must hold, not scroll
    "by the way my coffee machine broke again this morning total disaster honestly what a mess " +
    "here are the numbers that matter revenue reached three point five million dollars this quarter " +
    "growing forty seven percent year over year with one thousand two hundred teams onboarded since twenty twenty four " +
    "if you present from a windows machine flowly keeps your eyes on the lens " +
    "and your script out of the screen share thanks for watching goodbye").split(" ");

const FAKE_SPEECH = `
  class FakeSpeechRecognition {
    constructor() {
      this.continuous = false; this.interimResults = false; this.lang = "";
      this.onresult = null; this.onerror = null; this.onend = null;
      this._timer = null;
    }
    start() {
      const words = ${JSON.stringify(SPOKEN)};
      let i = 0; let utterStart = 0;
      this._timer = setInterval(() => {
        if (!this.onresult) return;
        if (i >= words.length) { clearInterval(this._timer); return; }
        i++;
        const isFinal = i - utterStart >= 4 || i === words.length;
        const slice = words.slice(utterStart, i).join(" ");
        const result = { isFinal, 0: { transcript: slice }, length: 1 };
        this.onresult({ results: { length: 1, 0: result } });
        if (isFinal) utterStart = i;
      }, 210); // ~285 wpm feed; the engine's clamps keep motion sane
    }
    stop() { if (this._timer) clearInterval(this._timer); this.onend?.(); }
  }
  window.SpeechRecognition = FakeSpeechRecognition;
`;

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1000, height: 700 } });
page.on("pageerror", (e) => check("no page errors", false, String(e)));
await page.addInitScript(FAKE_SPEECH);
await page.goto("http://localhost:4574/");

// ---- library seeds with the sample ----
await page.waitForSelector('[data-testid="script-list"]');
check("sample script seeded", (await page.locator(".script").count()) === 1);

// ---- file imports ----
const fixtures = join(here, "fixtures");
await page.setInputFiles('[data-testid="file-input"]', [
  join(fixtures, "update.docx"),
  join(fixtures, "briefing.pdf"),
  join(fixtures, "notes.txt"),
]);
await page.waitForFunction(() => document.querySelectorAll(".script").length === 4, null, { timeout: 20000 });
const titles = await page.locator(".script__title").allTextContents();
check("docx imported", titles.includes("update"), titles.join(", "));
check("pdf imported", titles.includes("briefing"));
check("txt imported", titles.includes("notes"));

const stored = await page.evaluate(() => localStorage.getItem("flowly.scripts.v1"));
const lib = JSON.parse(stored);
const byTitle = (t) => lib.find((s) => s.title === t)?.body ?? "";
check("docx heading -> #", byTitle("update").includes("# Quarterly update"), JSON.stringify(byTitle("update").slice(0, 80)));
check("docx bold -> **", byTitle("update").includes("**record levels**"));
check("pdf text extracted", byTitle("briefing").includes("annual board briefing"));
check("txt speaker label -> stage note", byTitle("notes").includes("(( Sarah ))"), JSON.stringify(byTitle("notes").slice(0, 60)));
check("txt smart quotes normalized", byTitle("notes").includes('"full show"'));

// ---- HTML paste ----
await page.locator('[data-testid="paste-zone"]').evaluate((el) => {
  const dt = new DataTransfer();
  dt.setData("text/plain", "Launch plan\nShip the pill to beta users.");
  dt.setData(
    "text/html",
    "<h2>Launch plan</h2><p>Ship the <b>pill</b> to beta users.</p><script>evil()</script>"
  );
  el.dispatchEvent(new ClipboardEvent("paste", { clipboardData: dt, bubbles: true, cancelable: true }));
});
await page.waitForFunction(() => document.querySelectorAll(".script").length === 5);
const lib2 = JSON.parse(await page.evaluate(() => localStorage.getItem("flowly.scripts.v1")));
const pasted = lib2.find((s) => s.title === "Pasted script")?.body ?? "";
check("paste keeps heading", pasted.includes("## Launch plan"), JSON.stringify(pasted));
check("paste keeps bold", pasted.includes("**pill**"));
check("paste drops script tags", !pasted.includes("evil"));

// ---- live voice: WASM aligner driven by fake speech ----
await page.locator(".script", { hasText: "Flowly launch demo" }).getByText("● Start with voice").click();
await page.waitForSelector(".pill--active", { timeout: 15000 });
check("voice take starts", true);

// Record every engine-state class the pill ever wears — polling can miss the
// brief LOST window between ad-lib and re-anchor.
await page.evaluate(() => {
  window.__seenStates = new Set();
  const record = () => {
    const pill = document.querySelector(".pill");
    if (!pill) return;
    for (const c of pill.classList) if (c.startsWith("pill--")) window.__seenStates.add(c);
  };
  record();
  new MutationObserver(record).observe(document.body, {
    subtree: true,
    attributes: true,
    attributeFilter: ["class"],
  });
});

const positions = [];
for (let i = 0; i < 40; i++) {
  await page.waitForTimeout(700);
  const past = await page.locator(".w--past").count();
  positions.push(past);
  if ((await page.locator(".pill--done").count()) > 0) break;
}
const sawLost = await page.evaluate(() => window.__seenStates.has("pill--lost"));
const final = positions[positions.length - 1];
check("cursor followed the voice", final > 55, `final past-words=${final}, trace=${positions.join(",")}`);
check("engine went LOST during ad-lib", sawLost);
// The display must never retract more than the dead-reckoning cap (8): the
// only legitimate backward glide is a re-anchor correcting speculative drift.
const bounded = positions.every((p, i) => i === 0 || p >= positions[i - 1] - 8);
check("backward motion bounded by dead-reckoning cap", bounded, positions.join(","));

await browser.close();
server.close();
console.log(failures === 0 ? "\nALL CHECKS PASSED" : `\n${failures} CHECK(S) FAILED`);
process.exit(failures === 0 ? 0 : 1);
