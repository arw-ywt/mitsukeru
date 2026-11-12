# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-11-13

### Summary
Version 0.2.0 is a major milestone achieving **zero default dependencies** out-of-the-box.  
All external crates (`serde`, `serde_json`, `libm`, `schemars`, `web-time`) are now fully optional. This release significantly reduces binary sizes, eliminates supply-chain risks, and guarantees deterministic execution on bare-metal embedded targets (`no_std`), WebAssembly, and native high-throughput environments.

### Breaking Changes
- **Empty Default Features (`default = []`)**:
  - `std`, `serde`, and `serde_json` are no longer included by default.
  - Rust consumers utilizing JSON serialization or `#[derive(Serialize, Deserialize)]` must now explicitly opt in via the `serde` feature flag.
- **Clock Requirement in `no_std`**:
  - In `no_std` mode (when `std` is disabled), `Optimizer::new` no longer bundles an OS timer. Calling `.solve()` with a time limit (`time_limit_ms`) without supplying a clock will return `Err(OptimizeError::InvalidInput)`. Provide an external hardware timer via `.with_clock(...)` or disable time limits via `.preset_exact().with_max_nodes(...)`.
- **`schemars` Feature Dependency**:
  - The `schemars` feature now explicitly implies `std` and `serde`, as Schema generation macros strictly require standard library primitives.

### Migration Guide (v0.1.0 to v0.2.0)

#### If you use Serde / JSON parsing in Rust:
Update your `Cargo.toml` dependency to enable the `serde` feature:
```diff
- mitsukeru = "0.1"
+ mitsukeru = { version = "0.2", features = ["serde"] }
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
- **Resolved Out-of-Bounds Memory Safety Vulnerability in 2D-DP (`dp_branchless.rs`)**:
  - Fixed a critical subtraction underflow where an item whose first weight fits within `cap1` but whose second weight exceeds `cap2` caused `self.cap2 - iw2` to wrap around in release builds.
  - Added capacity boundary validation to prevent out-of-bounds pointer traversal and potential memory corruption during unsafe DP table lookups.

### Added
- **Pure Zero-Dependency Fallback for Mathematics (`heuristics.rs`)**:
  - Implemented an exact argument-reduction and 6th-order Taylor polynomial approximation with IEEE 754 floating-point bit manipulation for sigmoid scoring. Eliminates the runtime necessity of `libm` in `no_std` mode while maintaining deterministic score parity.
- **External Clock Inversion-of-Control**:
  - Added full support for custom timer injection via `.with_clock(&dyn Clock)`, allowing bounded search on embedded architectures without requiring the standard library.
- **Comprehensive Regression Tests**:
  - Added `tests/test_dp_branchless.rs` validating multi-dimensional capacity boundary limits.
  - Added `tests/test_no_std_clock.rs` verifying timer injection and fail-safe error handling in zero-dependency mode.
- **CI Enhancements**:
  - Added automated assertions to verify zero default dependencies (`cargo tree`).
  - Added JSON schema drift detection (`schema_drift_test.rs`).
  - Added ARM Cortex-M cross-compilation and strict Clippy validation (`thumbv7em-none-eabi`).

### Changed
- Refactored `BranchAndBoundSolver` loop indexing to comply with strict pedantic Clippy rules.
- Updated `crates/mitsukeru-wasm` to explicitly depend on `mitsukeru` with `features = ["serde", "libm"]`.
- Updated repository documentation and badges to advertise `dependencies: 0 (default)`.


## [0.1.0] - 2026-03-15

### Added
- Initial release of the `mitsukeru` constraint optimization engine.
- High-performance multi-dimensional Knapsack solver using branch-and-bound search.
- Branchless 2D Dynamic Programming (DP) solver core.
- Graph structural heuristics and 3-SAT reduction compiler.
- Automated infeasibility conflict diagnostics report.
- WebAssembly bindings (`mitsukeru-wasm`) and TypeScript/Node.js package.