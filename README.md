# mitsukeru

`mitsukeru` is a lightweight, deterministic, and explainable constraint optimization engine written in Pure Rust. 

It can be compiled to native Rust code for high-performance backend services or to WebAssembly (Wasm) for client-side browser apps.

## What is mitsukeru?

`mitsukeru` solves multi-dimensional knapsack problems with business rules 
(item dependencies and mutual exclusions) directly inside your process or browser.
Unlike general-purpose MILP solvers that require native toolchains and return 
opaque error codes, `mitsukeru` runs entirely in WebAssembly with sub-millisecond 
latency and explains *why* a configuration is impossible.

## Use Case

### When to use
- Interactive UI / Web Configurators: Options selection (e.g., e-commerce, CPQ, vehicle/PC customization) where $N \le 60$ and immediate 60fps feedback is required.
- Offline & Edge Environments: Embedded systems, IoT, or client-side web apps where network round-trips to optimization servers are unacceptable.
- Rule-Heavy Decision Making: Scenarios with complex prerequisite (`Requires`) and mutual exclusion (`Conflicts`) logic where failures must be explained to end-users.

### When NOT to use
- Massive Industrial Scheduling: Problems with thousands of variables or logistics routing (Use HiGHS or Google OR-Tools).
- Continuous / Floating-point Optimization: Problems requiring real/fractional decision variables (e.g., portfolio optimization, continuous blending).



## Benchmark & Performance

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



## What You Need

One of the core philosophies of `mitsukeru` is zero system dependencies:

- X No C/C++ compiler required (unlike HiGHS, CBC, or SCIP)
- X No CMake or native build toolchains
- X No Python runtime or external solver binaries

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