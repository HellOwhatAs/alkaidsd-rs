# AlkaidSD-RS: Split Delivery Vehicle Routing Problem Solver (Rust)

This is a Rust port of the C++ AlkaidSD library for solving the Split Delivery Vehicle Routing Problem (SDVRP).

## Overview

The Split Delivery Vehicle Routing Problem is a variant of the classic Vehicle Routing Problem where customer demands can be split across multiple vehicles. This allows for more efficient routing when customer demands exceed vehicle capacity.

## Features

- **Zero-cost abstractions**: Leverages Rust's compile-time polymorphism and efficient memory management
- **Idiomatic Rust**: Uses traits, generics, and the ownership system
- **Complete implementation**: All major components from the C++ version are translated

## Module Structure

### Core Data Structures
- `instance` - Problem instance definition (`Instance`, `Node`)
- `solution` - Solution representation (`AlkaidSolution`)
- `route_context` - Route metadata (`RouteContext`)
- `delta` - Tie-breaking accumulator (`Delta<T>`)
- `random` - PRNG based on xoshiro128**

### Optimization Components
- `construction` - Initial solution construction heuristics
- `repair` - Route repair/consolidation
- `split_reinsertion` - Split delivery reinsertion

### Operators
- `intra_operator` - Within-route operators (`Exchange`, `OrOpt<N>`)
- `inter_operator` - Between-route operators:
  - `Relocate` - Move single node between routes
  - `Swap<N,M>` - Exchange segments between routes
  - `Cross` - Exchange tail segments
  - `SwapStar` - Exchange single nodes with best position search
  - `SdSwapStar`, `SdSwapOneOne`, `SdSwapTwoOne` - Split delivery variants

### Metaheuristics
- `acceptance_rule` - Solution acceptance rules:
  - `HillClimbing`
  - `HillClimbingWithEqual`
  - `LateAcceptanceHillClimbing`
  - `SimulatedAnnealing`
- `ruin_method` - Perturbation methods:
  - `RandomRuin`
  - `SisrsRuin` (Slack Induction by String Removals)
- `sorter` - Customer sorting strategies
- `solver` - Main ALNS solver

### Utilities
- `cache` - Caching system for move evaluations
- `distance_matrix_optimizer` - Floyd-Warshall shortest paths
- `utils` - Helper functions

## Example Usage

```rust
use alkaidsd::{Instance, AlkaidSolver, AlkaidConfig};
use alkaidsd::acceptance_rule::HillClimbing;
use alkaidsd::inter_operator::SwapStar;
use alkaidsd::intra_operator::Exchange;
use alkaidsd::ruin_method::RandomRuin;
use alkaidsd::sorter::{Sorter, SortByRandom};

fn main() {
    // Define problem instance
    let instance = Instance {
        num_customers: 5,
        capacity: 100,
        demands: vec![0, 50, 60, 40, 30],
        distance_matrix: vec![
            vec![0, 10, 20, 30, 40],
            vec![10, 0, 15, 25, 35],
            vec![20, 15, 0, 10, 20],
            vec![30, 25, 10, 0, 10],
            vec![40, 35, 20, 10, 0],
        ],
    };

    // Configure solver
    let mut sorter = Sorter::new();
    sorter.add_sort_function(Box::new(SortByRandom), 1.0);

    let mut config = AlkaidConfig {
        random_seed: 42,
        time_limit: 10.0,
        blink_rate: 0.01,
        inter_operators: vec![Box::new(SwapStar)],
        intra_operators: vec![Box::new(Exchange)],
        acceptance_rule: Box::new(|| Box::new(HillClimbing)),
        ruin_method: Box::new(RandomRuin::new(vec![1, 2, 3])),
        sorter,
        listener: None,
    };

    // Solve
    let solver = AlkaidSolver::default();
    let solution = solver.solve(&mut config, &instance);

    println!("Objective: {}", solution.calc_objective(&instance));
    println!("{}", solution);
}
```

## Building

```bash
cd alkaidsd-rs
cargo build --release
```

## Testing

```bash
cargo test
```

## Comparison with C++ Version

| Component | C++ | Rust |
|-----------|-----|------|
| Random | xoshiro128** | xoshiro128** |
| Solution | Linked-list style | Linked-list style |
| Operators | Template classes | Generic structs with traits |
| Caching | Type-erased cache map | Type-erased cache map |
| Memory | Manual (RAII) | Ownership system |
