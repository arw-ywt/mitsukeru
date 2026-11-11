/* tslint:disable */
/* eslint-disable */

/**
 * Performs static analysis to identify constraint conflicts or causes of infeasibility.
 *
 * # Errors
 *
 * Returns a [`JsValue`] if the serialization or deserialization of a JS object fails.
 */
export function analyze_infeasibility(input_value: any, bnb_best_value: number): any;

/**
 * Reduces the combinatorial problem into 3-SAT (in DIMACS CNF format).
 *
 * # Errors
 *
 * Returns a [`JsValue`] error in the following cases:
 * - If deserialization of the JavaScript input object fails.
 * - If any business rule references an invalid/out-of-bounds item index.
 * - If internal variable allocation overflows memory bounds.
 * - If serialization of the resulting CNF object into JavaScript fails.
 */
export function compile_to_3sat(input_value: any): any;

/**
 * Performs optimization based on the input data and settings.
 *
 * # Errors
 *
 * Returns a [`JsValue`] error if serialization or deserialization of the JS object fails, or if the optimization process fails.
 */
export function optimize(input_value: any, config_value?: any | null): any;

/**
 * Performs initial solution prediction using graph heuristics.
 *
 * # Errors
 *
 * Returns a [`JsValue`] if serialization or deserialization of a JS object fails.
 */
export function predict_heuristics(input_value: any): any;

/**
 * Performs a search using the Branch and Bound method.
 *
 * # Errors
 *
 * Returns a [`JsValue`] if the serialization or deserialization of a JS object fails.
 */
export function solve_branch_and_bound(input_value: any): any;

/**
 * Invokes the 2D dynamic programming core solver.
 *
 * # Errors
 *
 * Returns a [`JsValue`] if an error occurs, such as a failure to deserialize a JS object or an insufficient capacity constraint.
 */
export function solve_branchless_dp(input_value: any): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly analyze_infeasibility: (a: any, b: number) => [number, number, number];
    readonly compile_to_3sat: (a: any) => [number, number, number];
    readonly optimize: (a: any, b: number) => [number, number, number];
    readonly predict_heuristics: (a: any) => [number, number, number];
    readonly solve_branch_and_bound: (a: any) => [number, number, number];
    readonly solve_branchless_dp: (a: any) => [number, number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
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
