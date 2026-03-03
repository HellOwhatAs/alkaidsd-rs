//! # AlkaidSD - Split Delivery Vehicle Routing Problem Solver
//!
//! This crate provides a high-performance solver for the Split Delivery Vehicle
//! Routing Problem (SDVRP). The SDVRP is a variant of the classic Vehicle Routing
//! Problem where customer demands can be split across multiple vehicles.
//!
//! ## Key Components
//!
//! - [`Instance`]: Problem instance definition with customers, demands, and distances
//! - [`AlkaidSolution`]: Solution representation using linked-list style node storage
//! - [`AlkaidSolver`]: Main solver implementing adaptive large neighborhood search
//! - Various operators for solution improvement (inter-route and intra-route)
//!
//! ## Zero-Cost Abstractions
//!
//! This implementation leverages Rust's zero-cost abstractions:
//! - Generic traits for operators and rules enable compile-time polymorphism
//! - `#[inline]` hints for hot paths
//! - Stack allocation where possible
//! - Efficient iterator patterns

pub mod acceptance_rule;
pub mod cache;
pub mod construction;
pub mod delta;
pub mod distance_matrix_optimizer;
pub mod instance;
pub mod inter_operator;
pub mod intra_operator;
pub mod random;
pub mod repair;
pub mod route_context;
pub mod ruin_method;
pub mod solution;
pub mod solver;
pub mod sorter;
pub mod split_reinsertion;
pub mod utils;

// Re-export main types at crate root for convenience
pub use acceptance_rule::AcceptanceRule;
pub use instance::{Instance, Node};
pub use solution::AlkaidSolution;
pub use solver::{AlkaidConfig, AlkaidSolver, Config, Listener, Solver};
