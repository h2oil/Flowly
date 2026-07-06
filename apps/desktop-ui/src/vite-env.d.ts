/// <reference types="vite/client" />

declare module "*.wasm?inline" {
  const dataUri: string;
  export default dataUri;
}

declare module "mammoth/mammoth.browser" {
  export function convertToHtml(
    input: { arrayBuffer: ArrayBuffer },
    options?: { styleMap?: string[] }
  ): Promise<{ value: string; messages: unknown[] }>;
}

declare module "pdfjs-dist/build/pdf.worker.min.mjs?url" {
  const url: string;
  export default url;
}
