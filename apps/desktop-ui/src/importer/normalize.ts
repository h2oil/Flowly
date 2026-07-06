// Paste/import normalization (docs/plan/05-data-and-sync.md §ingestion).
// Deterministic and golden-file-testable: Unicode NFKC, smart quotes →
// straight, zero-width chars stripped, NBSP → space, speaker labels turned
// into unspoken stage notes so the aligner never chases "JAMES:".

export function normalizePlainText(raw: string): string {
  let text = raw.normalize("NFKC");
  text = text
    .replace(/\r\n?/g, "\n")
    .replace(/[‘’‛]/g, "'")
    .replace(/[“”‟]/g, '"')
    .replace(/[​‌‍﻿­]/g, "")
    .replace(/ /g, " ");
  // Speaker labels at line start become stage notes: "JAMES: Hello" → "(( James )) Hello"
  text = text.replace(/^([A-Z][A-Z .'-]{1,28}):[ \t]+/gm, (_, label: string) => {
    const pretty = label
      .toLowerCase()
      .replace(/(^|[\s.'-])([a-z])/g, (m) => m.toUpperCase());
    return `(( ${pretty.trim()} )) `;
  });
  // Collapse horizontal whitespace runs; keep newlines meaningful.
  text = text
    .split("\n")
    .map((l) => l.replace(/[ \t]+/g, " ").trimEnd())
    .join("\n");
  // At most one blank line between paragraphs.
  text = text.replace(/\n{3,}/g, "\n\n").trim();
  return text + "\n";
}

/** Convert clipboard/docx HTML to Prompter Markdown. Allowlist-driven walk —
 * scripts, styles, and unknown containers contribute nothing but their text. */
export function htmlToPrompterMarkdown(html: string): string {
  const doc = new DOMParser().parseFromString(html, "text/html");
  const out: string[] = [];

  const inline = (node: Node): string => {
    if (node.nodeType === Node.TEXT_NODE) return node.textContent ?? "";
    if (node.nodeType !== Node.ELEMENT_NODE) return "";
    const el = node as Element;
    const tag = el.tagName.toLowerCase();
    if (tag === "script" || tag === "style" || tag === "noscript") return "";
    if (tag === "br") return "\n";
    const inner = Array.from(el.childNodes).map(inline).join("");
    if ((tag === "b" || tag === "strong") && inner.trim()) return `**${inner.trim()}** `;
    return inner;
  };

  const block = (el: Element): void => {
    const tag = el.tagName.toLowerCase();
    if (tag === "script" || tag === "style" || tag === "noscript" || tag === "head") return;
    if (/^h[1-6]$/.test(tag)) {
      const level = Math.min(Number(tag[1]), 3);
      const text = inline(el).replace(/\*\*/g, "").trim();
      if (text) out.push(`${"#".repeat(level)} ${text}`);
      return;
    }
    if (tag === "p" || tag === "li" || tag === "blockquote" || tag === "figcaption" || tag === "td" || tag === "th") {
      const text = inline(el).trim();
      if (text) out.push(text);
      return;
    }
    if (el.children.length === 0) {
      const text = inline(el).trim();
      if (text) out.push(text);
      return;
    }
    for (const child of Array.from(el.children)) block(child);
  };

  block(doc.body);
  return normalizePlainText(out.join("\n\n"));
}

/** Best-effort RTF to plain text: drops control groups and words, decodes
 * hex/unicode escapes. Good enough for notes; complex layouts should arrive
 * as .docx. */
export function rtfToText(rtf: string): string {
  let s = rtf;
  // Remove binary and destination groups we never want ({\fonttbl…}, {\*\…}).
  for (const dest of ["fonttbl", "colortbl", "stylesheet", "info", "pict", "themedata", "listtable"]) {
    s = s.replace(new RegExp(`\\{\\\\${dest}[^{}]*(?:\\{[^{}]*\\}[^{}]*)*\\}`, "g"), "");
  }
  s = s.replace(/\{\\\*[^{}]*(?:\{[^{}]*\}[^{}]*)*\}/g, "");
  s = s.replace(/\\par[d]?\b/g, "\n");
  s = s.replace(/\\line\b/g, "\n");
  s = s.replace(/\\tab\b/g, " ");
  s = s.replace(/\\u(-?\d+)\??/g, (_, n) => String.fromCharCode(((Number(n) % 65536) + 65536) % 65536));
  s = s.replace(/\\'([0-9a-fA-F]{2})/g, (_, h) => String.fromCharCode(parseInt(h, 16)));
  s = s.replace(/\\[a-zA-Z]+-?\d*\s?/g, "");
  s = s.replace(/[{}]/g, "");
  return normalizePlainText(s);
}
