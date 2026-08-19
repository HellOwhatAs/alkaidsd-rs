//! Core trait for VRP problem variants.
//!
//! This module defines the [`VrpVariant`] trait, which is the central abstraction
//! for supporting multiple VRP variants (SDVRP, VRPPD, EVRP, etc.) within a
//! single solver framework.
//!
//! # Design Philosophy
//!
//! - **Operators are variant-agnostic**: Inter-route and intra-route operators
//!   work with the base [`Instance`] type (distance matrix, capacity, demands).
//!   They don't need to know about variant-specific constraints like split delivery
//!   or pickup-delivery pairing.
//!
//! - **Variant logic is encapsulated**: Construction, repair, reinsertion, and
//!   constraint handling are defined by each variant's implementation of this trait.
//!
//! - **Zero-cost abstraction**: The solver is generic over `VrpVariant`, so Rust's
//!   monomorphization ensures that solving SDVRP through this framework produces
//!   identical code to a direct implementation.
//!
//! # Implementing a New Variant
//!
//! To add a new VRP variant:
//!
//! 1. Define your instance type (can wrap [`Instance`] with additional data)
//! 2. Implement [`VrpVariant`] for your variant struct
//! 3. Use `AlkaidSolver::solve_variant::<YourVariant>()` to solve
//!
//! ```rust,ignore
//! struct VrppdInstance {
//!     base: Instance,
//!     pairs: Vec<(Node, Node)>,  // (pickup, delivery) pairs
//! }
//!
//! struct Vrppd;
//!
//! impl VrpVariant for Vrppd {
//!     type Instance = VrppdInstance;
//!
//!     fn base_instance(instance: &VrppdInstance) -> &Instance {
//!         &instance.base
//!     }
//!
//!     // ... implement other methods with VRPPD-specific logic
//! }
//! ```

use crate::instance::{Instance, Node};
use crate::random::Random;
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;

/// Trait defining a VRP problem variant's behavior within the solver framework.
///
/// Each VRP variant (SDVRP, VRPPD, EVRP, etc.) implements this trait to
/// provide variant-specific logic for construction, repair, reinsertion,
/// and objective evaluation. The solver framework calls these methods at
/// the appropriate points, while operators work with the common base
/// [`Instance`] type.
///
/// # Associated Types
///
/// * `Instance` - The problem instance type. For simple variants this can be
///   [`Instance`] directly. For variants needing extra data (e.g., pickup-delivery
///   pairs), use a wrapper struct that contains an [`Instance`].
///
/// # Zero-Cost Guarantee
///
/// Since the solver is generic over `VrpVariant`, all trait method calls are
/// monomorphized at compile time. This means:
/// - No virtual dispatch overhead
/// - The compiler can inline all variant-specific logic
/// - SDVRP performance is identical to the non-generic implementation
pub trait VrpVariant {
    /// The problem instance type for this variant.
    ///
    /// For SDVRP, this is [`Instance`] directly.
    /// For other variants, this is typically a struct wrapping [`Instance`]
    /// with additional variant-specific data.
    type Instance;

    /// Returns a reference to the base [`Instance`] data.
    ///
    /// The base instance contains the distance matrix, capacity, and demands
    /// that operators and other framework components need. This method allows
    /// the framework to extract the common data regardless of the variant.
    ///
    /// For SDVRP where `Instance` is the instance type, this is simply
    /// the identity function.
    fn base_instance(instance: &Self::Instance) -> &Instance;

    /// Constructs an initial feasible solution.
    ///
    /// Each variant defines its own construction heuristic that respects
    /// variant-specific constraints (e.g., split delivery for SDVRP,
    /// pickup-before-delivery for VRPPD).
    fn construct(instance: &Self::Instance, random: &mut Random) -> AlkaidSolution;

    /// Repairs a route after modifications.
    ///
    /// Called after intra-route operators modify a route. The repair step
    /// ensures variant-specific invariants are maintained.
    ///
    /// - SDVRP: Consolidates multiple visits to the same customer.
    /// - VRPPD: Ensures pickup-delivery ordering constraints.
    /// - EVRP: Inserts or adjusts charging station visits.
    fn repair(
        instance: &Self::Instance,
        route_index: Node,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
    );

    /// Removes all nodes serving the given customers from the solution.
    ///
    /// Called during the ruin phase of perturbation. The implementation
    /// should handle variant-specific removal logic (e.g., for SDVRP,
    /// a customer may have multiple nodes across routes; for VRPPD,
    /// both pickup and delivery nodes must be removed together).
    fn remove_customers(
        instance: &Self::Instance,
        customers: &[Node],
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
    );

    /// Reinserts customers after ruin.
    ///
    /// Called during the repair phase of perturbation. The implementation
    /// defines how customers are placed back into routes.
    ///
    /// - SDVRP: Uses split reinsertion (demand split across routes).
    /// - CVRP: Standard greedy reinsertion.
    /// - VRPPD: Paired reinsertion (pickup and delivery together).
    fn reinsert_customers(
        instance: &Self::Instance,
        customers: &[Node],
        blink_rate: f64,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
        random: &mut Random,
    );

    /// Calculates the objective (total cost) of a solution.
    fn calc_objective(instance: &Self::Instance, solution: &AlkaidSolution) -> i32;

    /// Calculates a lower bound on the fleet size.
    ///
    /// Used to determine the maximum stagnation count for the solver.
    fn calc_fleet_lower_bound(instance: &Self::Instance) -> Node;
}
