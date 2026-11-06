
<h1 align="center">
mitsukeru
</h1>

[![js-standard-style](https://cdn.rawgit.com/feross/standard/master/badge.svg)](https://github.com/feross/standard)
[![Crates.io Version](https://img.shields.io/crates/v/your-package-name.svg?logo=rust&style=flat-square)](https://crates.io/crates/your-package-name)
[![Docs.rs](https://img.shields.io/docsrs/your-package-name.svg?logo=rust&style=flat-square)](https://docs.rs/your-package-name)
[![npm version](https://img.shields.io/npm/v/your-package-name.svg?logo=npm&style=flat-square)](https://www.npmjs.com/package/your-package-name)
[![npm downloads](https://img.shields.io/npm/dm/your-package-name.svg?logo=npm&style=flat-square)](https://www.npmjs.com/package/your-package-name)
[![JSON Status](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fapi.example.com%2Fdata.json&query=%24.status&label=API%20Status&style=flat-square)](https://example.com)
[![JSON Format](https://img.shields.io/badge/data-JSON-informational?logo=json&style=flat-square)](https://www.json.org/)
![Zero Dependencies](https://img.shields.io/badge/dependencies-0-brightgreen)
[![Benchmark](https://github.com/arw-ywt/リポジトリ名/actions/workflows/benchmark.yml/badge.svg)](https://github.com/arw-ywt/リポジトリ名/actions/workflows/benchmark.yml)

`mitsukeru` is a lightweight, deterministic, and explainable constraint optimization engine written in Pure Rust, designed to ease spec changes and quickly spot conflicting conditions during optimal combination search.

It can be compiled to native Rust code for high-performance backend services or to WebAssembly (Wasm) for client-side browser apps.

- [What you gain](#what-you-gain)
- [Installation](#installation)
- [When to use](#)
- [When NOT to use](#)
- [Quickstart](#)
  - [Usage in Browser](#)
- [](#)
- [](#)

## What you gain

- Input: JSON
- Output: structured reports and human-readable messages
- Very fast
- Zero dependency
- Serverless support
- Open Source

## Installation

```bash
npm i mitsukeru
```

## When to use

Want to see it in action first? Try the [Demo](https://your-username.github.io/mitsukeru/) on GitHub Pages.

- Finding optimal combinations based on user-selected options (See Demo)
- Simplifying conditional logic caused by frequent option or specification changes (See Demo)
- Running under strict security or offline environments where external servers are unavailable (See Demo)

## When NOT to use

- Massive industrial scheduling (use HiGHS or Google OR-Tools instead etc.)
- Continuous / Floating-point optimization (use scipy.optimize etc.)

## Quickstart

### Usage in Browser

```typescript
import init, { optimize } from "./pkg/mitsukeru.js";

async function run() {
  await init();

  const config = {
    items: [
      { id: 0, name: "Basic Plan", weights: [10], value: 50 },
      { id: 1, name: "Pro Plan",   weights: [25], value: 120 },
    ],
    capacities: [20],
    rules: [],
  };

  try {
    const solution = optimize(config);
    console.log("Max Score:", solution.best_value);
    console.log("Selected IDs:", solution.selected_item_ids);
  } catch (error) {
    // When infeasible, returns the structured ConflictReport object
    // error.logical_chain_conflicts[0].path_ids can be used to highlight UI elements
    console.error("Selection failed. Conflicting path IDs:", 
      error.logical_chain_conflicts.map(c => c.path_ids)
    );
}

run();
```



<!--
As applications grow, managing options, prerequisites, and resource limits (like budgets, weights, or slot counts) quickly turns into a maze of fragile `if/else` logic. Every time rules change, existing conditions break.

`mitsukeru` solves this by decoupling your business rules from your UI code. You define your candidates, multi-dimensional capacities, and logical rules (**Requires / Conflicts**) cleanly in JSON. 

Under the hood, it combines high-performance branch-and-bound algorithms with automated conflict diagnosis. Instead of returning opaque errors, it tells you precisely *why* a configuration is broken—allowing you to build bulletproof configuration screens, SaaS plan selectors, and dynamic pricing engines in Rust or WebAssembly (Wasm) with sub-millisecond latency.
-->



## Use Case

### When to use
- Interactive UI / Web Configurators: Options selection (e.g., e-commerce, CPQ, vehicle/PC customization) where $N \le 60$ and immediate 60fps feedback is required.
- Offline & Edge Environments: Embedded systems, IoT, or client-side web apps where network round-trips to optimization servers are unacceptable.
- Rule-Heavy Decision Making: Scenarios with complex prerequisite (`Requires`) and mutual exclusion (`Conflicts`) logic where failures must be explained to end-users.

### When NOT to use
- Massive Industrial Scheduling: Problems with thousands of variables or logistics routing (Use HiGHS or Google OR-Tools).
- Continuous / Floating-point Optimization: Problems requiring real/fractional decision variables (e.g., portfolio optimization, continuous blending).



## How to perform

Measurements were performed using [Criterion.rs](https://github.com/bheisler/criterion.rs) with 100 sample iterations on a standard multi-constraint scenario (`Test A`: 14 items, 2 resource dimensions, 5 dependency/conflict rules).

| Metric | Measured Value | Note |
| :--- | :--- | :--- |
| Mean Execution Time | 2.22 µs (±30 ns) | Capable of ~450,000 solves/sec |
| Search Tree Nodes | 78 nodes | Pruned by 96.3% via surrogate fractional relaxation |
| Memory Allocation | 0 bytes (in search hot path) | Uses stack-allocated `BitSet256` |
| Time per 60fps Frame | 0.013% of frame budget | Leaves 99.98% of frame time for UI rendering |

(All benchmarks run on Rust stable, release profile, statistical outliers < 2%).



## Quality Assurance & Correctness

`mitsukeru` guarantees reliability and reproducible behavior through four mechanisms:

1. Strict Determinism (Zero Float Bugs)
   - All internal weights, capacities, and bounding logic operate exclusively on integers (`u32`) and bitmasks (`u64`).
   - Completely immune to floating-point rounding errors (`1e-7` tolerance issues), ensuring identical results across all platforms, architectures, and browser engines.

2. Schema Drift Detection Test
   - The official specification (`schemas/v1/mitsukeru-schema.json`) is tied directly to the Rust data model via automated continuous integration.
   - Tests automatically fail if internal structs are modified without updating the published schema.

3. Structured & Explainable Infeasibility Analysis
   - In addition to solving feasible instances, the engine traverses dependency graphs via BFS to detect logical chain conflicts and capacity overflows.
   - Instead of opaque error codes, it returns machine-readable structured reports (`ConflictReport`, containing `root_id`, `path_ids`, and `bundle_ids`) alongside human-readable messages, enabling UI highlights (e.g., auto-highlighting conflicting items in red).

4. SAT-Reduction Cross Validation
   - Contains a built-in Tseitin/Adder-Tree compiler that reduces the knapsack problem directly into DIMACS 3-SAT format.
   - Enables dual-engine verification where solutions can be formally validated against third-party SAT solvers (e.g., Z3, Kissat).



## What You Gain

When a user's configuration is invalid or impossible to satisfy, traditional solvers simply return an opaque `Infeasible` error. `mitsukeru` features a built-in graph-based analyzer (`InfeasibilityAnalyzer`) that detects and explains three major classes of configuration conflicts.

### 1. Logical Chain Conflicts (Prerequisite vs. Exclusion)
Detects when a chain of required dependencies structurally leads to an item that has been explicitly forbidden by a mutual exclusion rule.
- **Example:** Item $A$ requires Item $B$, and Item $B$ requires Item $C$, but a conflict rule states that $A$ cannot coexist with $C$ ($A \to B \to C$ conflicts with $A$).
- **Structured Output:** Returns the exact root item ID, conflicting item ID, and the full dependency path (`path_ids: [0, 1, 2]`) for precise UI highlighting (e.g., rendering the conflicting node path in red).

### 2. Bundle Capacity Overflows
Detects when a selected item—either individually or together with all its mandatory dependencies (a "bundle")—exceeds multi-dimensional resource limits (e.g., total weight, power budget, or slot count).
- **Example:** Selecting a high-end GPU bundle requires 3 mandatory sub-components, pushing the total power consumption to 550W against a maximum capacity of 500W.
- **Structured Output:** Identifies the bottleneck dimension index, the allowed limit, the actual consumption, and the exact list of item IDs causing the overflow (`bundle_ids`).

### 3. Unachievable Target Values
Detects when a requested objective target (e.g., a minimum score or profit threshold) cannot be reached under the given constraints and item pools.
- **Example:** The user requests a target score of 500 pt, but the maximum mathematically achievable score under current resource caps is 450 pt.
- **Structured Output:** Returns both the requested target value and the maximum achievable bound.



## What You Need

One of the core philosophies of `mitsukeru` is zero system dependencies:

- **NO** C/C++ compiler required (unlike HiGHS, CBC, or SCIP)
- **NO** CMake or native build toolchains
- **NO** Python runtime or external solver binaries

#### System Requirements:
- For Rust projects: Rust 1.70+ (Stable)
- For Web / Node.js projects: Node.js 18+ (if building from source: `wasm-pack`)



## Installation

### 1. In Rust Projects
Add `mitsukeru` to your `Cargo.toml`:

```toml
[dependencies]
mitsukeru = "0.1.0"
serde_json = "1.0" # Optional: for loading configurations from JSON
```

Or via cargo CLI:
```bash
cargo add mitsukeru
```

### 2. In Web / TypeScript Projects
You can compile `mitsukeru` to WebAssembly using `wasm-pack`:

```bash
# Build the WebAssembly package for bundlers (Vite, Webpack, Next.js)
wasm-pack build --target web --release
```

## Quickstart

### A. Basic Usage in Rust

```rust
use mitsukeru::{Optimizer, KnapsackInput, Item, BusinessRule, OptimizeError};

fn main() {
    // 1. Define candidates, capacities, and rules
    let input = KnapsackInput {
        items: vec![
            Item { id: 0, name: "Sunroof".into(),      weights: vec![15, 50], value: 100 },
            Item { id: 1, name: "Reinforced Roof".into(), weights: vec![20, 30], value: 60 },
            Item { id: 2, name: "Roof Rack".into(),    weights: vec![10, 20], value: 80 },
        ],
        capacities: vec![30, 80], // [Weight capacity, Power budget]
        rules: vec![
            // Selecting Sunroof (0) requires Reinforced Roof (1)
            BusinessRule::Requires { from_id: 0, to_id: 1 },
            // Sunroof (0) and Roof Rack (2) cannot be equipped together
            BusinessRule::Conflicts { a_id: 0, b_id: 2 },
        ],
        target_value: None,
    };

    // 2. Solve with validation and explanation
    match Optimizer::new(&input).solve() {
        Ok(solution) => {
            println!("Optimal Score: {}", solution.best_value);
            println!("Selected Item IDs: {:?}", solution.selected_item_ids);
            println!("Elapsed Time: {:.3} ms (Nodes: {})", 
                solution.execution_time_ms, solution.explored_nodes);
        }
        Err(OptimizeError::Infeasible(report)) => {
            eprintln!("Configuration is infeasible!");
            for conflict in report.logical_chain_conflicts {
                eprintln!(" - Conflict at item #{}: {} (Path IDs: {:?})", 
                    conflict.root_id, conflict.message, conflict.path_ids);
            }
            for overflow in report.capacity_conflicted { // capacity_conflicts
                eprintln!(" - Capacity overflow on dim {} by items: {:?}", 
                    overflow.dimension + 1, overflow.bundle_ids);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```


### B. Usage in TypeScript / Browser (Wasm)

```typescript
import init, { optimize } from "./pkg/mitsukeru.js";

async function run() {
  await init();

  const config = {
    items: [
      { id: 0, name: "Basic Plan", weights: [10], value: 50 },
      { id: 1, name: "Pro Plan",   weights: [25], value: 120 },
    ],
    capacities: [20],
    rules: [],
  };

  try {
    const solution = optimize(config);
    console.log("Max Score:", solution.best_value);
    console.log("Selected IDs:", solution.selected_item_ids);
  } catch (error) {
    // When infeasible, returns the structured ConflictReport object
    // error.logical_chain_conflicts[0].path_ids can be used to highlight UI elements
    console.error("Selection failed. Conflicting path IDs:", 
      error.logical_chain_conflicts.map(c => c.path_ids)
    );
}

run();
```

## Editor Autocompletion via JSON Schema

You can write configuration files with instant validation and autocomplete in editors like VS Code by specifying the `$schema` header:

```json
{
  "$schema": "./schemas/v1/mitsukeru-schema.json",
  "items": [
    {
      "id": 0,
      "name": "Winter Package",
      "weights": [10, 5],
      "value": 150
    }
  ],
  "capacities": [50, 20],
  "rules": [
    {
      "type": "Requires",
      "from_id": 0,
      "to_id": 1
    }
  ]
}
```

To regenerate the schema file locally after updating data structures:
```bash
cargo run --bin export_schema
```

## License

MIT License