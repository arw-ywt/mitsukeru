## [0.2.0] - 2026-11-13

### Summary
Version 0.2.0 is a major milestone achieving **zero default dependencies** out-of-the-box alongside comprehensive **memory-safety and resource-limit hardening**.  
All external crates (`serde`, `serde_json`, `libm`, `schemars`, `web-time`) are now fully optional, while all unsafe code in the 2D DP solver has been eliminated and unified input/arithmetic safety ceilings are enforced across both the Rust core and all WebAssembly entrypoints.

### Breaking Changes
- **Empty Default Features (`default = []`)**:
  - `std`, `serde`, and `serde_json` are no longer included by default.
  - Standard Rust consumers relying on the default OS clock (`preset_fast()` / `time_limit_ms`) must enable the `std` feature; JSON serialization requires `serde`.
- **Clock Requirement in `no_std`**:
  - In `no_std` mode (when `std` is disabled), `Optimizer::new` no longer bundles an OS timer. Calling `.solve()` with a time limit (`time_limit_ms`) without supplying a clock will return `Err(OptimizeError::InvalidInput)`. Provide an external hardware timer via `.with_clock(...)` or disable time limits via `.preset_exact().with_max_nodes(...)`.
- **Fallible (`Result`-Returning) Solver, Analyzer, and Core APIs**:
  - To reject integer overflows, dimension/item ceiling violations, and excessive memory allocations cleanly instead of panicking or wrapping, the following APIs now return `Result<..., OptimizeError>`:
    - `Flat2dDpSolver::new`, `Flat2dDpSolver::solve`
    - `BranchAndBoundSolver::solve`
    - `GraphHeuristics::predict`, `GraphHeuristics::predict_with_clock`
    - `InfeasibilityAnalyzer::analyze`
    - `solve_branchless_dp_core` (changed from `Result<DpResult, String>` to `Result<DpResult, OptimizeError>`), `solve_branch_and_bound_core`, `analyze_infeasibility_core`
  - `CnfBuilder::with_limits`, `alloc_var`, `add_clause`, and gate synthesis helpers now return `Result<..., SatCompileError>`.
- **Widened `CapacityConflict::consumed_weight` (`u32` -> `u64`)**:
  - `CapacityConflict::consumed_weight` is now `u64` to accurately report multi-item bundle consumptions that exceed `u32::MAX`.
- **Enforced Safety & Architectural Ceilings**:
  - `MAX_SUPPORTED_ITEMS = 2048` is now enforced uniformly across all solvers, analyzers, and WASM entrypoints (previously truncated silently in direct solver calls).
  - `MAX_CAPACITY_DIMS = 8` is now enforced (`> 8` dimensions return `Err(OptimizeError::InvalidInput)`).
  - `MAX_SUPPORTED_RULES = 32_768`, `MAX_TOP_K = 256`, total item `value` sum `<= u32::MAX`, finite non-negative `time_limit_ms`, `DEFAULT_MAX_DP_MEMORY_BYTES = 64 MiB`, and default SAT circuit limits (`250_000` vars / `1_000_000` clauses) are enforced.
- **Unified WebAssembly Error Format (`mitsukeru-wasm`)**:
  - All 6 WASM functions (`optimize`, `solve_branchless_dp`, `compile_to_3sat`, `solve_branch_and_bound`, `analyze_infeasibility`, `predict_heuristics`) now enforce unified ceilings, and DP/BnB/analyzer/heuristic entrypoints reject invalid inputs with structured `OptimizeError` objects (`{ type: "InvalidInput", message }`).
- **`schemars` Feature Dependency**:
  - The `schemars` feature now explicitly implies `std` and `serde`, and `pub use schemars;` is properly gated on `feature = "schemars"` (fixing compilation under `--no-default-features --features std`).

### Migration Guide (v0.1.0 to v0.2.0)

#### If you use standard Rust (`std` clock / `preset_fast` / Serde JSON):
Update your `Cargo.toml` dependency to opt into `std` (and `serde` if using JSON):
```diff
- mitsukeru = "0.1"
+ mitsukeru = { version = "0.2", features = ["std", "serde"] }
```

If you call standalone solvers or `*_core` functions directly, propagate or handle the `Result`:
```diff
- let mut dp = Flat2dDpSolver::new(cap1, cap2);
- let (best, ids) = dp.solve(&items);
+ let mut dp = Flat2dDpSolver::new(cap1, cap2)?;
+ let (best, ids) = dp.solve(&items)?;
```

#### If you use `no_std` / Bare-Metal / Microcontrollers:
No additional flags required! `mitsukeru` is now completely zero-dependency by default:
```toml
mitsukeru = "0.2"
```

To enable execution time limits on bare-metal systems, supply your own hardware timer implementing the `Clock` trait:
```rust
use mitsukeru::clock::Clock;
use mitsukeru::Optimizer;

struct MyHardwareClock;
impl Clock for MyHardwareClock {
    fn now_micros(&self) -> u64 {
        read_systick_timer_micros()
    }
}

let solution = Optimizer::new(&input)
    .with_clock(&MyHardwareClock)
    .with_time_limit_ms(50.0)
    .solve()?;
```

### Security & Bug Fixes
- **Eliminated `unsafe` & Overflow/Underflow UBs in 2D-DP (`dp_branchless.rs`, `solve_branchless_dp_core`)**:
  - Removed all `unsafe` pointer traversal (`curr_ptr`, `prev_ptr`, `hist_ptr`) and `get_unchecked` indexing in favor of safe slice splitting (`split_at_mut`) and bounds-checked indexing, resolving pre-allocation pointer `.sub(1)` UB and `bit_idx` underflow on zero-weight (`[0, 0]`) items.
  - Replaced unchecked grid and history size arithmetic with `checked_add` / `checked_mul` and enforced a pre-allocation memory budget (`DEFAULT_MAX_DP_MEMORY_BYTES = 64 MiB`), preventing 32-bit (`wasm32` / embedded) and 64-bit `usize` wrap-around and OOM DoS.
  - Fixed the `self.cap2 - iw2` subtraction underflow via an early boundary guard (`if iw1 > self.cap1 || iw2 > self.cap2 { continue; }`) and added `checked_add` on DP value accumulation.
- **Fixed >8-Dimension Panic & Cross-Solver Inconsistencies (`branch_and_bound.rs`, `heuristics.rs`)**:
  - Fixed a slice out-of-bounds panic (`&next_weights[..num_dims]`) in `BranchAndBoundSolver` when `capacities.len() > 8` and silent ignoring of dimensions `>= 8` in `GraphHeuristics` by enforcing `MAX_CAPACITY_DIMS = 8` uniformly.
- **Fixed `u32` Arithmetic Overflow Bypass & Misreporting (`branch_and_bound.rs`, `heuristics.rs`, `analyzer.rs`)**:
  - Upgraded internal weight accumulation and capacity comparisons to `u64`, preventing release-mode `u32` wrap-around from allowing overweight item combinations to bypass capacity limits or go undetected during infeasibility analysis.
  - Added upfront `u32::MAX` total item `value` validation and `saturating_add` / `checked_add` guards on value sums.
- **Closed Resource Exhaustion Vectors Across Core & WASM (`validation.rs`, `sat_compiler.rs`, `mitsukeru-wasm`)**:
  - Bounded `top_k`, `rules.len()`, 2D-DP memory, and 3-SAT variable/clause counts (with `checked_add` on `CnfBuilder::next_var`), and rejected `NaN` / negative / infinite `time_limit_ms`.

### Added
- **Unified Validation Layer (`validation.rs`)**:
  - Added `validate_problem_input`, `validate_optimizer_config`, and public ceiling constants (`MAX_SUPPORTED_ITEMS`, `MAX_CAPACITY_DIMS`, `MAX_SUPPORTED_RULES`, `MAX_TOP_K`, `DEFAULT_MAX_DP_MEMORY_BYTES`, `DEFAULT_MAX_SAT_VARS`, `DEFAULT_MAX_SAT_CLAUSES`).
- **Configurable Solver Resource Budgets**:
  - Added `Flat2dDpSolver::new_with_budget` for custom 2D-DP memory limits.
  - Added `SatCompileError::ResourceLimitExceeded`, `CnfBuilder::with_limits`, and `CompilerConfig` ceilings (`max_items`, `max_dimensions`, `max_rules`, `max_vars`, `max_clauses`).
- **Pure Zero-Dependency Fallback for Mathematics (`heuristics.rs`)**:
  - Implemented an exact argument-reduction and 6th-order Taylor polynomial approximation with IEEE 754 floating-point bit manipulation for sigmoid scoring. Eliminates the runtime necessity of `libm` in `no_std` mode while maintaining deterministic score parity.
- **External Clock Inversion-of-Control**:
  - Added full support for custom timer injection via `.with_clock(&dyn Clock)`, allowing bounded search on embedded architectures without requiring the standard library.
- **Comprehensive Regression Tests & Verification**:
  - Added unit regression tests covering capacity/history overflows, `u32` weight/value overflows, zero-weight DP items, `>8` dimensions, `top_k` bounds, SAT circuit budgets, and core entrypoint limits (verified under `--release` and Miri).
  - Added `tests/test_dp_branchless.rs` and `tests/test_no_std_clock.rs`.
- **CI Enhancements**:
  - Added automated assertions to verify zero default dependencies (`cargo tree`).
  - Added JSON schema drift detection (`schema_drift_test.rs`).
  - Added ARM Cortex-M cross-compilation and strict Clippy validation (`thumbv7em-none-eabi`).

### Changed
- Refactored `BranchAndBoundSolver` loop indexing to comply with strict pedantic Clippy rules.
- Updated `crates/mitsukeru-wasm` to explicitly depend on `mitsukeru` with `features = ["std", "serde"]` (using `web-time` for browser/WASM `time_limit_ms` support).
- Updated repository documentation (`README.md`) and badges to advertise `dependencies: 0 (default)` and document enforced safety ceilings.

## [0.1.0] - 2026-11-11

### Added
- Initial release of the `mitsukeru` constraint optimization engine.
- High-performance multi-dimensional Knapsack solver using branch-and-bound search.
- Branchless 2D Dynamic Programming (DP) solver core.
- Graph structural heuristics and 3-SAT reduction compiler.
- Automated infeasibility conflict diagnostics report.
- WebAssembly bindings (`mitsukeru-wasm`) and TypeScript/Node.js package.