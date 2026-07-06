/* tslint:disable */
/* eslint-disable */

export class WasmAligner {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Feed one hypothesis (flowly-asr JSON shape). Returns the event array.
     */
    feed(hypothesis_json: string, now_ms: number): string;
    /**
     * Manual override; always wins. Returns the event array.
     */
    jump_to(token_index: number, now_ms: number): string;
    constructor(source: string);
    /**
     * Clock tick (drives silence -> PAUSED). Returns the event array.
     */
    tick(now_ms: number): string;
    wpm(): number;
}

/**
 * Compile a script and return renderer-facing token info as JSON:
 * `[{display, line, sentenceStart}]`.
 */
export function compile_tokens(source: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmaligner_free: (a: number, b: number) => void;
    readonly compile_tokens: (a: number, b: number) => [number, number];
    readonly wasmaligner_feed: (a: number, b: number, c: number, d: number) => [number, number];
    readonly wasmaligner_jump_to: (a: number, b: number, c: number) => [number, number];
    readonly wasmaligner_new: (a: number, b: number) => number;
    readonly wasmaligner_tick: (a: number, b: number) => [number, number];
    readonly wasmaligner_wpm: (a: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
