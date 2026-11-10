<h1 align="center">
mitsukeru
</h1>

<div align="center">
<img src="https://raw.githubusercontent.com/arw-ywt/mitsukeru/main/assets/logo.svg" alt="mitsukeru logo" width="500" />
</div>

[![CI](https://img.shields.io/github/actions/workflow/status/arw-ywt/mitsukeru/ci.yml?branch=main&label=CI&logo=github&style=flat-square)](https://github.com/arw-ywt/mitsukeru/actions)
![WebAssembly](https://img.shields.io/badge/WebAssembly-supported-654FF0?logo=webassembly&logoColor=white&style=flat-square)
![MSRV](https://img.shields.io/badge/MSRV-1.75+-orange?logo=rust&style=flat-square)
<br/>
[![Crates.io Version](https://img.shields.io/crates/v/mitsukeru?logo=rust&style=flat-square)](https://crates.io/crates/mitsukeru)
[![Docs.rs](https://img.shields.io/docsrs/mitsukeru?logo=docs.rs&style=flat-square)](https://docs.rs/mitsukeru)
[![npm version](https://img.shields.io/npm/v/mitsukeru?logo=npm&style=flat-square)](https://www.npmjs.com/package/mitsukeru)
<br/>
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)


**A zero-C-dependency, reproducible constraint optimization engine written in 100% pure Rust.**
<br/>
Pass items and rules as data instead of writing complex `if/else` branching. Finds optimal combinations in microseconds and automatically explains why constraints cannot be satisfied.


- [Installation](#installation)
- [Quick Start](#quick-start)
  - [1. Define the Problem & Run](#1-define-the-problem--run)
  - [2. Output](#2-output)
  - [3. Go to playgraound](#3-go-to-playground)
- [Features](#features)
- [When to use](#when-to-use)
- [When NOT to use](#when-not-to-use)
- [Options](#options)
  - [Presets](#presets)
  - [Builder Methods](#builder-methods)
- [Related Projects](#related-projects)
  - [Spaghetti vs. Mitsukeru](#spaghetti-vs-mitsukeru)
  - [The Needle in the Web](#the-needle-in-the-web)
  - [Societal Scale Optimization](#societal-scale-optimization)
- [License](#license)

## Installation

```bash
npm i mitsukeru
```

## Quick Start

Here is a quick example of how to optimize item selection under capacity limits and business rules (e.g., *Item A requires Item B*, and *Item X conflicts with Item Y*).

### 1. Define the Problem & Run
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
      { type: "conflicts", a_id: 2, b_id: 3 }     // Light A and B cannot be mixed
    ],
    config: {
      strategy: "mixed",
      time_limit_ms: 2.0,
      top_k: 2
    }
  });

  console.log("Success:", result);

} catch (err) {
  // Strongly typed Discriminated Union for errors
  if (err.type === "Infeasible") {
    console.error("Conflict Report:", err.report);
  }
}
```

### 2. Output

```json
{
  "best_value": 830,
  "selected_item_ids": [0, 1, 3],
  "execution_time_ms": 0.02,
  "explored_nodes": 12,
  "status": "optimal",
  "strategy_used": "mixed",
  "top_candidates": [
    { "value": 830, "selected_item_ids": [0, 1, 3] },
    { "value": 820, "selected_item_ids": [0, 1, 2] }
  ],
  "upper_bound": 830.0,
  "optimality_gap": 0.0
}
```

If the rules are logically impossible to satisfy, `mitsukeru` instantly generates an actionable human-readable report instead of failing silently:

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

# 3. Go to playgraound

Want to input and output JSON yourself? [Here](https://arw-ywt.github.io/mitsukeru/docs), you can try out our interactive playground on your own device.

## Features

- **Sub-Millisecond Performance**

  Powered by a bit-parallel branch-and-bound core (`BitSet<const W>`), the solver evaluates millions of nodes per second with zero heap allocations in hot loops, delivering optimal or near-optimal solutions in milliseconds.

- **Mixed Auto (Auto-Tuning Hybrid Engine)**

  Automatically analyzes problem scale and constraint density at runtime. For sparse or small problems, it instantly dispatches a pure exact solver; for dense constraint networks, it leverages constraint graph analysis and structural heuristics to prune search spaces by up to 100x.

- **Explainable Infeasibility Analysis**

  When a problem cannot be satisfied due to contradictory rules, it automatically runs a diagnostic report explaining *why* (e.g., tracing exact logical chain conflicts, bundle capacity overflows, or unachievable targets).

- **Top-K Alternative Solutions**

  Go beyond the single optimal answer. Instantly retrieve top-ranked alternative solutions (Plan A, Plan B, Plan C) sorted in descending order of value, enabling flexible decision-making.

- **100% Pure Rust & WebAssembly Ready**

  Zero C/C++ native dependencies. It compiles cleanly to `wasm32-unknown-unknown`, allowing you to run high-performance constraint optimization directly in browsers, Node.js, or serverless Edge environments.

## When to Use

### Rule Management and Architecture
- Writing complex conditional selection logic without nested `if/else` or `switch/case` statements
- Maintaining and updating business constraints regularly through data schemas rather than code deployments
- Decoupling selection criteria, eligibility requirements, and limits from application code

### Solution Requirements
- Finding multiple diverse and high-quality alternatives
- Considering transitive and chained logical constraints where item selections trigger subsequent requirements or mutual exclusions
- Analyzing the root cause of infeasible inputs and identifying the exact chain of conflicting constraints

### Deployment and Performance
- Executing directly on the client side in a browser or inside serverless and edge environments
- Deploying as a self-contained, zero-dependency library without external runtimes or external solver processes
- Obtaining mathematically guaranteed optimal decisions within sub-millisecond execution budgets

## When NOT to Use

- When you encounter a problem where the number of items exceeds 2048 (NOT SUPPORTED, even in the mode with no time limit and no limit on search nodes)
- When you optimize continuous or divisible variables—such as in percentage allocations or linear programming models
- When you handle problems with high-dimensional capacity limits (e.g., ten or more simultaneous resource dimensions) where surrogate relaxation bounds become loose
- When you run offline batch processes where execution latency, memory footprint, and external runtime dependencies are not operational constraints

If you are looking for a solver for the above purpose, use general-purpose MIP/LP solvers such as Gurobi, SCIP, or Google OR-Tools.

## Options

You can configure solver execution using presets or fluent builder methods on the Optimizer instance.

### Presets
- `Optimizer::new(&input).preset_fast()` (Default)
  Configured for low-latency and synchronous execution. Applies a 2.0 ms wall-clock limit and a 100,000 explored node cap.
- `Optimizer::new(&input).preset_accurate()`
  Configured for higher exploration depth. Applies a 100.0 ms wall-clock limit and a 1,000,000 explored node cap.
- `Optimizer::new(&input).preset_exact()`
  Removes time limits and node constraints to compute and prove the exact mathematical optimum.

### Builder Methods
- `.with_strategy(SolverStrategy::Pure | Heuristic | Mixed)`
  Explicitly sets the search mode. `Pure` executes standard branch-and-bound search, `Heuristic` initializes search state using graph-based predictive embeddings, and `Mixed` automatically selects between them based on rule density and item count.
- `.with_time_limit_ms(limit_ms: f64)`
  Sets a custom wall-clock execution limit in milliseconds. When exceeded, the solver returns the best feasible solution found along with an optimality gap.
- `.with_max_nodes(max_nodes: u64)`
  Sets an upper bound on explored search tree nodes.
- `.with_top_k(k: usize)`
  Configures the engine to record and return up to k distinct, high-scoring candidate solutions ordered by objective value.

## Related Projects

Below, we present practical examples of how this engine is used.

### [Spaghetti vs. Mitsukeru](https://arw-ywt.github.io/mitsukeru/examples/spaghetti/)

Easily replace `if/else` or `switch/case` statements, `json-rules-engine`, or tools like HiGHS and Google OR-Tools using `mitsukeru`.


### [The Needle in the Web](https://arw-ywt.github.io/mitsukeru/examples/needle/)
Build a personalized set of offers or product features that fits a visitor’s budget and technical constraints. `mitsukeru` selects a high-value combination, respects dependencies and incompatibilities, and can return alternative plans for the user to compare. Run the optimization in the browser with WebAssembly; your application supplies the candidate scores and business rules.


### [Societal Scale Optimization](https://arw-ywt.github.io/mitsukeru/examples/societal/)

Help planners compare portfolios of community support actions when budgets, staff, and supplies are limited. Encode operational prerequisites and incompatible assignments as explicit rules, then rank feasible plans by a transparent impact score. `mitsukeru` can support repeatable planning at a defined regional or operational scope; people remain responsible for setting priorities and reviewing the outcomes.


## Benchmarks

### Environment
- Hardware: Intel Core i7 / AMD Ryzen (x86_64), 16 Cores, 32 GB RAM
- OS: Windows 11 / Ubuntu 22.04 LTS
- Rust Version: 1.75+ (Target: `x86_64-pc-windows-msvc`, Release profile, LTO enabled)
- Harness: Criterion v0.5 with black_box barriers against dead-code elimination

### Stress Sweep
Based on the number of items ($N$), each item's two weights (<code>weights</code>) and value (<code>value</code>) are determined deterministically using prime numbers (37, 41, 29) and modulo operations, along with increased capacity, target score, and constraints. Chain depth is kept shallow (at most 2–3 levels). The search constraints are a 2.0 ms timeout and a limit of 100,000 total search nodes.

| Items ($N$) | Strategy  | Status     | Best Value | Gap (%)   | Explored Nodes | Time (ms)  | Throughput (Mnodes/s) |
| :---:  | :---:     | :---:      | :---:      | :---:     | :---:        | :---:      | :---:    |
| 16     | Pure      | Optimal    | 611 pt     | 0.00%     | 541          | 0.07       | 8.2      |
| 16     | Heuristic | Optimal    | 611 pt     | 0.00%     | 541          | 0.06       | 9.6      |
| 16     | Mixed     | Optimal    | 611 pt     | 0.00%     | 541          | 0.02       | 32.8     |
| 32     | Pure      | Optimal    | 1266 pt    | 0.00%     | 236          | 0.04       | 6.6      |
| 32     | Heuristic | Optimal    | 1266 pt    | 0.00%     | 236          | 0.03       | 6.9      |
| 32     | Mixed     | Optimal    | 1266 pt    | 0.00%     | 236          | 0.02       | 9.6      |
| 64     | Pure      | Optimal    | 2553 pt    | 0.00%     | 579          | 0.08       | 7.6      |
| 64     | Heuristic | Optimal    | 2553 pt    | 0.00%     | 579          | 0.07       | 7.7      |
| 64     | Mixed     | Optimal    | 2553 pt    | 0.00%     | 579          | 0.12       | 4.8      |
| 128    | Pure      | Optimal    | 5082 pt    | 0.00%     | 21,085       | 2.03       | 10.4     |
| 128    | Heuristic | Optimal    | 5082 pt    | 0.00%     | 21,085       | 2.06       | 10.2     |
| 128    | Mixed     | Approx     | 5082 pt    | 1.34%     | 20,480       | 2.26       | 9.0      |
| 256    | Pure      | Approx     | 10052 pt   | 1.90%     | 12,288       | 2.17       | 5.7      |
| 256    | Heuristic | Approx     | 10054 pt   | 1.88%     | 16,384       | 2.89       | 5.7      |
| 256    | Mixed     | Approx     | 10054 pt   | 1.88%     | 16,384       | 2.83       | 5.8      |
| 512    | Pure      | Approx     | 20054 pt   | 2.29%     | 8,192        | 2.64       | 3.1      |
| 512    | Heuristic | Approx     | 20054 pt   | 2.29%     | 8,192        | 2.79       | 2.9      |
| 512    | Mixed     | Approx     | 20054 pt   | 2.29%     | 8,192        | 3.00       | 2.7      |
| 1024   | Pure      | Approx     | 40148 pt   | 2.09%     | 4,096        | 3.81       | 1.1      |
| 1024   | Heuristic | Approx     | 40148 pt   | 2.09%     | 4,096        | 4.07       | 1.0      |
| 1024   | Mixed     | Approx     | 40148 pt   | 2.09%     | 4,096        | 4.01       | 1.0      |
| 2048   | Pure      | Approx     | 80067 pt   | 2.27%     | 4,096        | 10.75      | 0.4      |
| 2048   | Heuristic | Approx     | 80067 pt   | 2.27%     | 4,096        | 14.37      | 0.3      |
| 2048   | Mixed     | Approx     | 80067 pt   | 2.27%     | 4,096        | 14.08      | 0.3      |

You can reproduce these benchmarks locally using the following command:

```bash
cargo test --test test_f_scaling --release -- --nocapture
```

### Strategies
Here are the benchmark results measured for each test using Criterion.rs:
- **`Pure`**: Pure deterministic bit-parallel branch-and-bound search.
- **`Heuristic`**: Search guided by initial graph-embedding predictions.
- **`Mixed`** (Default): Adaptive engine that selects strategy based on constraint graph density and item volume.

| Test Case | Candidate Items | Constraint Profile | Pure BnB | Heuristic BnB | Mixed (Adaptive) | Status |
| :--- | :---: | :--- | :---: | :---: | :---: | :---: |
| Test A | 14 | Standard paired dependencies | 2.38 µs | 5.68 µs | 2.59 µs (via Pure) | Proven Optimal |
| Test B | 10 | Sequential requires chain | 2.30 µs | 4.65 µs | 2.31 µs (via Pure) | Proven Infeasible |
| Test C | 7 | Low density, multi-body rules | 1.14 µs | 2.86 µs | 1.15 µs (via Pure) | Proven Optimal |
| Test D | 12 | Asymmetric high-capacity bounds | 3.17 µs | 5.66 µs | 3.18 µs (via Pure) | Proven Optimal |
| Test E | 18 | Multi-stage alternating rules | 12.68 µs | 16.44 µs | 12.70 µs (via Pure) | Proven Optimal |
| Test F | 26 | Balanced periodic constraints | 21.17 µs | 26.38 µs | 26.40 µs (via Heuristic)| Proven Optimal |
| Test G | 28 | Dense synergy clusters & honey-traps | 12.45 µs | 8.78 µs | 8.87 µs (via Heuristic)| Proven Optimal |

You can reproduce these benchmarks locally using the following command:

```bash
cargo bench --bench solver_bench
```

## License

This project is licensed under the [MIT License](LICENSE).