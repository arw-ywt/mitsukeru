<h1 align="center">
mitsukeru
</h1>

<div align="center">
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/arw-ywt/mitsukeru/main/assets/logo_dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/arw-ywt/mitsukeru/main/assets/logo.svg">
  <img alt="mitsukeru" src="logo.svg" width="500">
</picture>
</div>

<p align="center">
  <a href="https://github.com/arw-ywt/mitsukeru/actions"><img src="https://img.shields.io/github/actions/workflow/status/arw-ywt/mitsukeru/ci.yml?branch=main&label=CI&logo=github&style=flat-square" alt="CI" /></a>
  <img src="https://img.shields.io/badge/dependencies-0%20(default)-success?style=flat-square" alt="0 dependencies (default)" />
  <img src="https://img.shields.io/badge/no__std-supported-blue?style=flat-square" alt="no_std supported" />
  <img src="https://img.shields.io/badge/WebAssembly-supported-654FF0?logo=webassembly&logoColor=white&style=flat-square" alt="WebAssembly" />
  <img src="https://img.shields.io/badge/MSRV-1.88+-orange?logo=rust&style=flat-square" alt="MSRV" />
  <br/>
  <a href="https://crates.io/crates/mitsukeru"><img src="https://img.shields.io/crates/v/mitsukeru?logo=rust&style=flat-square" alt="Crates.io Version" /></a>
  <a href="https://docs.rs/mitsukeru"><img src="https://img.shields.io/docsrs/mitsukeru?logo=docs.rs&style=flat-square" alt="Docs.rs" /></a>
  <a href="https://www.npmjs.com/package/mitsukeru"><img src="https://img.shields.io/npm/v/mitsukeru?logo=npm&style=flat-square" alt="npm version" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square" alt="License: MIT" /></a>
</p>

A zero-dependency (by default), deterministic, safety-critical constraint optimization engine written in 100% pure Rust.
<br/>
Pass items and business rules as declarative data instead of writing entangled `if/else` or `switch/case` logic. Finds mathematically optimal combinations in microseconds and automatically explains why constraints cannot be satisfied. Runs anywhere from bare-metal microcontrollers (`no_std`) and high-throughput backend services to browsers via WebAssembly.

- [Installation](#installation)
- [Quick Start](#quick-start)
  - [Rust (Native / Embedded)](#1-rust-native--embedded)
  - [TypeScript / JavaScript (Browser / Node.js)](#2-typescript--javascript-browser--nodejs)
  - [Infeasibility Diagnostic Output](#3-infeasibility-diagnostic-output)
  - [Playground](#4-playground)
- [Key Features](#key-features)
- [Safety-Critical & Architecture](#safety-critical--architecture)
- [When to Use](#when-to-use)
- [When NOT to Use](#when-not-to-use)
- [Options & Presets](#options--presets)
- [Benchmarks](#benchmarks)
- [License](#license)

## Installation

### Rust (Cargo)
Add `mitsukeru` to your `Cargo.toml`:

```toml
[dependencies]
# Standard environments (includes OS/browser clock for time_limit_ms):
mitsukeru = { version = "0.2", features = ["std"] }

# With JSON serialization support:
# mitsukeru = { version = "0.2", features = ["std", "serde"] }

# Bare-metal / embedded microcontrollers (no_std + alloc, zero dependencies by default):
# mitsukeru = "0.2"s
```

### TypeScript / JavaScript (npm)
For browser, Node.js, and edge runtimes:

```bash
npm i mitsukeru
```

## Quick Start

### 1. Rust (Native / Embedded)

```rust
use mitsukeru::{BusinessRule, Item, Optimizer, ProblemInput};

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let input = ProblemInput {
        items: vec![
            Item { id: 0, name: "Pro Camera".into(), weights: vec![10, 5], value: 500 },
            Item { id: 1, name: "Carbon Tripod".into(), weights: vec![3, 2], value: 120 },
            Item { id: 2, name: "LED Light A".into(), weights: vec![5, 3], value: 200 },
            Item { id: 3, name: "LED Light B".into(), weights: vec![5, 3], value: 210 },
        ],
        capacities: vec![20, 10], // [Max Budget, Max Weight]
        rules: vec![
            BusinessRule::Requires { from_id: 0, to_id: 1 }, // Camera requires Tripod
            BusinessRule::Conflicts { a_id: 2, b_id: 3 }, // Light A and B cannot be mixed
        ],
        target_value: None,
    };

    // Solve with Top-2 candidates
    let solution = Optimizer::new(&input)
        .preset_fast()
        .with_top_k(2)
        .solve()?;

    println!("Best Value    : {} pt", solution.best_value);
    println!("Selected Items: {:?}", solution.selected_item_ids); // [0, 1, 3]

    if let Some(candidates) = solution.top_candidates {
        for (rank, cand) in candidates.iter().enumerate() {
            println!("Rank {}: {} pt -> {:?}", rank + 1, cand.value, cand.selected_item_ids);
        }
    }

    Ok(())
}
```

### 2. TypeScript / JavaScript (Browser / Node.js)

```typescript
import { optimize } from "mitsukeru";

try {
  const result = optimize({
    items: [
      { id: 0, name: "Pro Camera", weights: [10, 5], value: 500 },
      { id: 1, name: "Carbon Tripod", weights: [3, 2], value: 120 },
      { id: 2, name: "LED Light A", weights: [5, 3], value: 200 },
      { id: 3, name: "LED Light B", weights: [5, 3], value: 210 }
    ],
    capacities: [20, 10], // [Max Budget, Max Weight]
    rules: [
      { type: "requires", from_id: 0, to_id: 1 }, // Camera requires Tripod
      { type: "conflicts", a_id: 2, b_id: 3 } // Light A and B cannot be mixed
    ],
    config: {
      strategy: "mixed",
      time_limit_ms: 2.0,
      top_k: 2
    }
  });

  console.log("Optimal Score:", result.best_value); // 830
  console.log("Selected IDs :", result.selected_item_ids); // [0, 1, 3]
} catch (err: any) {
  if (err.type === "Infeasible") {
    console.error("Conflict Report:", err.report);
  } else if (err.type === "InvalidInput") {
    console.error("Invalid Input:", err.message);
  }
}
```

### 3. Infeasibility Diagnostic Output

If the business constraints are logically contradictory, `mitsukeru` traces the exact cause and returns an actionable report:

```json
{
  "type": "Infeasible",
  "report": {
    "is_feasible": false,
    "logical_chain_conflicts": [
      {
        "root_id": 0,
        "conflicting_id": 2,
        "path_ids": [0, 1, 2],
        "message": "Chain Conflict: [Pro Camera -> Carbon Tripod -> LED Light A] requires 'LED Light A', but it conflicts with root 'Pro Camera'"
      }
    ],
    "capacity_conflicts": [],
    "target_conflict": null
  }
}
```

### 4. Playground

Want to try JSON inputs and see JSON outputs in real time? Visit our [Playground](https://arw-ywt.github.io/mitsukeru/)!

## Key Features

- Sub-Millisecond Bit-Parallel Engine

Powered by a SIMD/bit-parallel branch-and-bound core (`BitSet<const W>`), evaluating millions of states per second with zero heap allocations in inner hot loops.

- Embedded & `no_std` First-Class Support

The core calculation engine is completely decoupled from OS primitives (`#![no_std] + alloc`). Deployable to bare-metal microcontrollers (ARM Cortex-M, RISC-V, ESP32) with zero C dependencies.

- Dependency Injected Clock

OS/browser clocks are automatically used in standard environments, while custom hardware timers (e.g., SysTick) can be injected via the `Clock` trait in embedded or deterministic unit testing environments.

- Mixed Auto

Analyzes constraint graph density and item size at runtime. Dispatches pure branch-and-bound for sparse problems and automatically enables structural graph heuristics to prune dense combinatorial search spaces by up to 100x.

- Top-K Alternative Options

Retrieves the top-$K$ best distinct configurations (Plan A, Plan B, Plan C) ordered by objective score, enabling flexible human-in-the-loop decision-making.

- WebAssembly Ready

Compiles cleanly to `wasm32-unknown-unknown` for high-speed client-side execution in browsers, Cloudflare Workers, and serverless edge functions.

## Architecture

The repository is architected as an isolated Cargo Workspace:

```text
mitsukeru/
├── crates/
│   ├── mitsukeru-core/
│   └── mitsukeru-wasm/
├── docs/
└── benches/
```

In `no_std` environments without an OS clock, specify a hardware timer or step counter by implementing the `Clock` trait:

```rust
use mitsukeru::{Clock, Optimizer};

struct MyHardwareSysTick;
impl Clock for MyHardwareSysTick {
    fn now_micros(&self) -> u64 {
        unsafe { read_systick_register() }
    }
}

let clock = MyHardwareSysTick;
let solution = Optimizer::new(&input)
    .with_clock(&clock)
    .with_time_limit_ms(5.0)
    .solve()?;
```
*(If a time limit is specified in a `no_std` environment without an injected clock, `mitsukeru` fails fast and returns `Err(OptimizeError::InvalidInput)` instead of silently hanging).*

## When to Use

### Rule Management
- Replacing complex, nested `if/else` or `switch/case` trees with data-driven declarative rules.
- Decoupling configuration logic, eligibility checks, and bundle constraints from application binaries.
- Needing root-cause conflict explanations when configurations fail.

### Operational Requirements
- High-frequency trading, avionics, robotics, or embedded systems where deterministic execution and zero OS latency are strictly required.
- Edge / Serverless deployments where external solver binaries (e.g., Gurobi, CPLEX, SCIP) cannot be linked.
- Interactive web apps requiring instantaneous constraint validation without server round-trips.

## When NOT to Use

- Continuous (divisible) linear programming models (e.g., fractional portfolio allocation). `mitsukeru` solves discrete 0-1 combinatorial problems.

- Problems exceeding the enforced safety & architectural ceilings excess inputs fail fast with `OptimizeError::InvalidInput` or `SatCompileError::ResourceLimitExceeded` to prevent OOM and non-deterministic behavior:

| Dimension / Resource | Enforced Limit (Default) | Constant / Config |
| :--- | :---: | :--- |
| Candidate items (`items.len()`) | 2,048 | `MAX_SUPPORTED_ITEMS` |
| Capacity dimensions (`capacities.len()`) | 8 | `MAX_CAPACITY_DIMS` |
| Business rules (`rules.len()`) | 32,768 | `MAX_SUPPORTED_RULES` |
| Alternative candidates (`top_k`) | 256 | `MAX_TOP_K` |
| Total item `value` sum | `u32::MAX` | Checked on input *(weights use `u64` internally)* |
| 2D DP memory budget | 64 MiB | `DEFAULT_MAX_DP_MEMORY_BYTES` / `Flat2dDpSolver::new_with_budget` |
| 3-SAT circuit size | 250k vars / 1M clauses | `CompilerConfig` (`DEFAULT_MAX_SAT_*`) |

## Options & Presets

### Presets
- `Optimizer::new(&input).preset_fast()` *(Default)*
  Optimized for synchronous / UI execution. Wall-clock budget of 2.0 ms, max 100,000 nodes.
- `Optimizer::new(&input).preset_accurate()`
  High-accuracy mode. Wall-clock budget of 100.0 ms, max 1,000,000 nodes.
- `Optimizer::new(&input).preset_exact()`
  Unbounded exploration proving mathematically global optimality.

### Fluent Methods
- `.with_strategy(SolverStrategy::Pure | Heuristic | Mixed)` — Force exact BnB, graph heuristics, or auto-selection.
- `.with_time_limit_ms(limit: f64)` — Set maximum execution budget in milliseconds. Must be finite and `>= 0.0` (negative numbers, `NaN`, and `Infinity` result in `InvalidInput`).
- `.with_max_nodes(nodes: u64)` — Bound explored search tree nodes (ideal for real-time systems).
- `.with_top_k(k: usize)` — Retain top $K$ (Up to `MAX_TOP_K = 256`) alternative feasible solutions.
- `.with_clock(&'a dyn Clock)` — Inject a custom monotonic time source.

## Benchmarks

Micro-benchmarked with Criterion v0.5 across varying problem profiles:

| Test Case | Candidates ($N$) | Constraint Structure | Pure BnB | Heuristic BnB | Mixed (Adaptive) | Optimality |
| :--- | :---: | :--- | :---: | :---: | :---: | :---: |
| Test A | 14 | Paired prerequisites | 1.82 µs | 3.66 µs | 2.24 µs *(Pure)* | Proven Optimal |
| Test B | 10 | Sequential dependency chain | 1.18 µs | 3.38 µs | 2.29 µs *(Pure)* | Proven Infeasible |
| Test C | 7 | Multi-body exclusion rules | 0.68 µs | 1.89 µs | 1.05 µs *(Pure)* | Proven Optimal |
| Test D | 12 | Asymmetric capacity bounds | 2.53 µs | 4.14 µs | 2.93 µs *(Pure)* | Proven Optimal |
| Test E | 18 | Multi-stage alternating rules| 11.48 µs | 13.75 µs | 12.06 µs *(Pure)* | Proven Optimal |
| Test F | 26 | Periodic cyclic constraints | 19.23 µs | 21.54 µs | 21.54 µs *(Heuristic)*| Proven Optimal |
| Test G | 28 | Dense honey-trap clusters | 10.60 µs | 5.08 µs | 5.36 µs *(Heuristic)*| Proven Optimal |

To run benchmarks locally:
```bash
cargo bench --bench solver_bench
```

## License

This project is licensed under the [MIT License](LICENSE).