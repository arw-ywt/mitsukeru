<img src="./assets/logo.png" alt="mitsukeru logo" width="120" />

<h1 align="center">
mitsukeru
</h1>

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
<br/>
On [this page](/), where this engine has been implemented, you can try it out immediately on your own device.

- [Installation](#installation)
- [Quick Start](#quick-start)
- [What you gain](#what-you-gain)
- [When to use](#)
- [When NOT to use](#)
- [Quickstart](#)
  - [Usage in Browser](#)
- [Related Projects](#related-projects)
  - [Spaghetti vs. Mitsukeru](#spaghetti-vs-mitsukeru)
  - [The Needle in the Web](#the-needle-in-the-web)
  - [Societal Scale Optimization](#societal-scale-optimization)
- [](#)


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

### 3.Output (When Constraints Clash)

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


Want to see more examples? [Here](/), you can try out more examples for yourself on your own device.

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

## When to use

Want to see it in action first? Try the Demo on GitHub Pages.

Finding optimal combinations based on user-selected options (See Demo)
Simplifying conditional logic caused by frequent option or specification changes (See Demo)
Running under strict security or offline environments where external servers are unavailable (See Demo)

## When NOT to use

Massive industrial scheduling (use HiGHS or Google OR-Tools instead etc.)
Continuous / Floating-point optimization (use scipy.optimize etc.)

## Configuration & Options

You can customize the solver's behavior using built-in presets or fluent builder methods:

- **Presets**:
  - `Optimizer::new(&input).preset_fast()` (Default): 2.0ms timeout, optimized for real-time web/sync usage.
  - `Optimizer::new(&input).preset_accurate()`: 100ms timeout, 1,000,000 nodes for batch/high-quality requirements.
  - `Optimizer::new(&input).preset_exact()`: Time-unlimited, guarantees an exact optimal proof.
- **Builder Methods**:
  - `.with_strategy(SolverStrategy::Pure | Info | Mixed)`: Explicitly choose the search strategy.
  - `.with_time_limit_ms(50.0)`: Set a custom wall-clock time limit.
  - `.with_max_nodes(500_000)`: Set an upper bound on explored search nodes.
  - `.with_top_k(3)`: Retrieve the top-K alternative ranked solutions.

## How It Works

1. **Bit-Parallel Exact Core (`BitSet<const W>`)**:
   Core branch-and-bound operations and transitive closures are powered by fixed-size, stack-allocated bitsets (`const W` generics), eliminating heap allocations (`malloc`) in hot loops and enabling LLVM auto-vectorization.
2. **Constraint Graph & Structural Heuristics**:
   For complex problems, the engine analyzes the topology of requirements and mutual exclusions to compute structural heuristics, warm-starting the exact solver to bypass redundant branches.
3. **Infeasibility Diagnostics**:
   When constraints clash, transitive closure matrices instantly pinpoint the root causes—tracing logical chain paths and multi-dimensional bundle capacity overflows without breaking a sweat.

## Related Projects

Below, we present practical examples of how this engine is used.

### [Spaghetti vs. Mitsukeru](/)

Easily replace `if/else` statements, `select` constructs, `json-rules-engine`, or tools like HiGHS and Google OR-Tools using `mitsukeru`.


### [The Needle in the Web](/)

Build a personalized set of offers or product features that fits a visitor’s budget and technical constraints. `mitsukeru` selects a high-value combination, respects dependencies and incompatibilities, and can return alternative plans for the user to compare. Run the optimization in the browser with WebAssembly; your application supplies the candidate scores and business rules.


### [Societal Scale Optimization](/)

Help planners compare portfolios of community support actions when budgets, staff, and supplies are limited. Encode operational prerequisites and incompatible assignments as explicit rules, then rank feasible plans by a transparent impact score. `mitsukeru` can support repeatable planning at a defined regional or operational scope; people remain responsible for setting priorities and reviewing the outcomes.


## License

This project is licensed under the [MIT License](LICENSE).

