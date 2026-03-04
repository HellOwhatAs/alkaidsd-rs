//! # AlkaidSD - Vehicle Routing Problem Solver Framework
//!
//! This crate provides a high-performance solver framework for Vehicle Routing
//! Problem (VRP) variants. The framework is designed around the [`VrpVariant`]
//! trait, enabling zero-cost support for multiple problem types including:
//!
//! - **SDVRP** (Split Delivery VRP) - Customer demands can be split across vehicles
//! - **VRPPD** (VRP with Pickup and Delivery) - Paired pickup/delivery constraints
//! - **EVRP** (Electric VRP) - Battery and charging constraints
//!
//! ## Key Components
//!
//! - [`VrpVariant`]: Core trait defining problem-variant-specific behavior
//! - [`Instance`]: Base problem data (distance matrix, capacity, demands)
//! - [`AlkaidSolution`]: Solution representation using linked-list style node storage
//! - [`AlkaidSolver`]: Main solver implementing adaptive large neighborhood search
//! - [`Sdvrp`]: Built-in SDVRP variant implementation
//!
//! ## Zero-Cost Abstraction
//!
//! The solver is generic over [`VrpVariant`], using Rust's monomorphization to
//! ensure that solving SDVRP through the generic framework produces identical
//! code to a direct implementation. Operators work with the base [`Instance`]
//! type and are completely unaware of the problem variant.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use alkaidsd::{AlkaidSolver, AlkaidConfig, Instance};
//! use alkaidsd::sdvrp::Sdvrp;
//!
//! // SDVRP (backward compatible)
//! let solver = AlkaidSolver::default();
//! let solution = solver.solve(&mut config, &instance);
//!
//! // Or explicitly using the variant
//! let solution = solver.solve_variant::<Sdvrp>(&mut config, &instance);
//! ```

pub mod acceptance_rule;
pub mod cache;
pub mod construction;
pub mod delta;
pub mod distance_matrix_optimizer;
pub mod instance;
pub mod inter_operator;
pub mod intra_operator;
pub mod problem;
pub mod random;
pub mod repair;
pub mod route_context;
pub mod ruin_method;
pub mod sdvrp;
pub mod solution;
pub mod solver;
pub mod sorter;
pub mod split_reinsertion;
pub mod utils;

// Re-export main types at crate root for convenience
pub use acceptance_rule::AcceptanceRule;
pub use instance::{Instance, Node};
pub use problem::VrpVariant;
pub use sdvrp::Sdvrp;
pub use solution::AlkaidSolution;
pub use solver::{AlkaidConfig, AlkaidSolver, Config, Listener, Solver};
