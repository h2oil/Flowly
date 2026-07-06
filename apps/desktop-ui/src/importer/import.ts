// File import: real scripts arrive as Word docs, Google Docs/Notion exports,
// PDFs, and messy pastes — never hand-authored Prompter Markdown. Paths in
// priority order per the plan: paste (HTML flavor preferred) → .docx (covers
// Word AND "Download as .docx" from Google Docs/Notion) → .txt/.md → .rtf →
// .pdf (degraded, review-before-recording). Everything runs in-browser;
// nothing is uploaded.

import { htmlToPrompterMarkdown, normalizePlainText, rtfToText } from "./normalize";

export interface ImportResult {
  title: string;
  body: string;
  kind: "text" | "markdown" | "docx" | "pdf" | "rtf" | "html";
  warning?: string;
}

export async function importFile(file: File): Promise<ImportResult> {
  const ext = (file.name.split(".").pop() ?? "").toLowerCase();
  const title = file.name.replace(/\.[^.]+$/, "") || "Imported script";

  switch (ext) {
    case "txt": {
      return { title, kind: "text", body: normalizePlainText(await file.text()) };
    }
    case "md":
    case "markdown": {
      // Markdown is already prompter-shaped; just normalize characters.
      return { title, kind: "markdown", body: normalizePlainText(await file.text()) };
    }
    case "html":
    case "htm": {
      return { title, kind: "html", body: htmlToPrompterMarkdown(await file.text()) };
    }
    case "rtf": {
      return { title, kind: "rtf", body: rtfToText(await file.text()) };
    }
    case "docx": {
      const mammoth = await import("mammoth/mammoth.browser");
      const { value } = await mammoth.convertToHtml(
        { arrayBuffer: await file.arrayBuffer() },
        { styleMap: ["comment-reference => "] }
      );
      const body = htmlToPrompterMarkdown(value);
      if (!body.trim()) throw new Error("That .docx contained no readable text.");
      return { title, kind: "docx", body };
    }
    case "pdf": {
      const body = await pdfToText(await file.arrayBuffer());
      if (!body.trim()) {
        throw new Error("No text layer found — this PDF looks scanned. Export the source document instead.");
      }
      return {
        title,
        kind: "pdf",
        body,
        warning: "PDF layout can scramble text — review before recording.",
      };
    }
    case "doc":
      throw new Error("Legacy .doc isn't supported — save it as .docx and import that.");
    default:
      throw new Error(`Unsupported file type ".${ext}". Use .docx, .pdf, .txt, .md, .rtf, or .html.`);
  }
}

/** Import pasted clipboard content, preferring the HTML flavor (keeps
 * headings/bold from Docs/Notion/Word). */
export function importPaste(plain: string, html?: string): ImportResult {
  if (html && /<[a-z][\s\S]*>/i.test(html)) {
    const body = htmlToPrompterMarkdown(html);
    if (body.trim()) return { title: "Pasted script", kind: "html", body };
  }
  return { title: "Pasted script", kind: "text", body: normalizePlainText(plain) };
}

async function pdfToText(data: ArrayBuffer): Promise<string> {
  const pdfjs = await import("pdfjs-dist");
  const workerUrl = (await import("pdfjs-dist/build/pdf.worker.min.mjs?url")).default;
  pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;
  const doc = await pdfjs.getDocument({ data }).promise;
  const pages: string[] = [];
  for (let p = 1; p <= doc.numPages; p++) {
    const page = await doc.getPage(p);
    const content = await page.getTextContent();
    // Rebuild lines from glyph runs: group by rounded Y, order by X.
    const rows = new Map<number, { x: number; str: string }[]>();
    for (const item of content.items as { str: string; transform: number[] }[]) {
      if (!item.str || !item.str.trim()) continue;
      const y = Math.round(item.transform[5]);
      const x = item.transform[4];
      if (!rows.has(y)) rows.set(y, []);
      rows.get(y)!.push({ x, str: item.str });
    }
    const lines = Array.from(rows.entries())
      .sort((a, b) => b[0] - a[0])
      .map(([, runs]) =>
        runs
          .sort((a, b) => a.x - b.x)
          .map((r) => r.str)
          .join(" ")
      );
    pages.push(lines.join("\n"));
  }
  return normalizePlainText(pages.join("\n\n"));
}
