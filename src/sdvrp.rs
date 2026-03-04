//! SDVRP (Split Delivery Vehicle Routing Problem) variant implementation.
//!
//! This module implements the [`VrpVariant`] trait for the SDVRP, where
//! customer demands can be split across multiple vehicles.
//!
//! # Usage
//!
//! ```rust,ignore
//! use alkaidsd::{AlkaidSolver, AlkaidConfig, Instance};
//! use alkaidsd::sdvrp::Sdvrp;
//!
//! let solver = AlkaidSolver::default();
//! let solution = solver.solve_variant::<Sdvrp>(&mut config, &instance);
//! ```
//!
//! For convenience, `AlkaidSolver::solve()` is equivalent to
//! `AlkaidSolver::solve_variant::<Sdvrp>()`.

use crate::construction::construct;
use crate::instance::{Instance, Node};
use crate::problem::VrpVariant;
use crate::random::Random;
use crate::repair::repair;
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;
use crate::split_reinsertion::split_reinsertion;
use crate::utils::calc_fleet_lower_bound;

/// The Split Delivery Vehicle Routing Problem variant.
///
/// In SDVRP, customer demands can be split across multiple vehicles,
/// allowing more efficient use of vehicle capacity.
pub struct Sdvrp;

impl VrpVariant for Sdvrp {
    type Instance = Instance;

    #[inline]
    fn base_instance(instance: &Instance) -> &Instance {
        instance
    }

    fn construct(instance: &Instance, random: &mut Random) -> AlkaidSolution {
        construct(instance, random)
    }

    fn repair(
        instance: &Instance,
        route_index: Node,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
    ) {
        repair(instance, route_index, solution, context);
    }

    fn remove_customers(
        _instance: &Instance,
        customers: &[Node],
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
    ) {
        for &customer in customers {
            for route_index in 0..context.num_routes() {
                let mut node_index = context.head(route_index);
                while node_index != 0 {
                    let successor = solution.successor(node_index);
                    if solution.customer(node_index) == customer {
                        let predecessor = solution.predecessor(node_index);
                        solution.remove(node_index);
                        if predecessor == 0 {
                            context.set_head(route_index, successor);
                        }
                        context.update_route_context(solution, route_index, predecessor);
                    }
                    node_index = successor;
                }
            }
        }
    }

    fn reinsert_customers(
        instance: &Instance,
        customers: &[Node],
        blink_rate: f64,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
        random: &mut Random,
    ) {
        for &customer in customers {
            split_reinsertion(
                instance,
                customer,
                instance.demands[customer as usize],
                blink_rate,
                solution,
                context,
                random,
            );
        }
    }

    fn calc_objective(instance: &Instance, solution: &AlkaidSolution) -> i32 {
        solution.calc_objective(instance)
    }

    fn calc_fleet_lower_bound(instance: &Instance) -> Node {
        calc_fleet_lower_bound(instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdvrp_base_instance() {
        let instance = Instance {
            num_customers: 3,
            capacity: 100,
            demands: vec![0, 50, 30],
            distance_matrix: vec![
                vec![0, 10, 20],
                vec![10, 0, 15],
                vec![20, 15, 0],
            ],
        };

        let base = Sdvrp::base_instance(&instance);
        assert_eq!(base.num_customers, 3);
        assert_eq!(base.capacity, 100);
    }

    #[test]
    fn test_sdvrp_construct() {
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

        let mut random = Random::new(42);
        let solution = Sdvrp::construct(&instance, &mut random);
        assert!(!solution.node_indices().is_empty());
        let obj = Sdvrp::calc_objective(&instance, &solution);
        assert!(obj > 0);
    }

    #[test]
    fn test_sdvrp_remove_and_reinsert() {
        let instance = Instance {
            num_customers: 3,
            capacity: 100,
            demands: vec![0, 50, 30],
            distance_matrix: vec![
                vec![0, 10, 20],
                vec![10, 0, 15],
                vec![20, 15, 0],
            ],
        };

        let mut solution = AlkaidSolution::new();
        let node1 = solution.insert(1, 50, 0, 0);
        solution.insert(2, 30, node1, 0);

        let mut context = RouteContext::new();
        context.calc_route_context(&solution);

        let original_obj = Sdvrp::calc_objective(&instance, &solution);
        assert!(original_obj > 0);

        // Remove customer 2
        Sdvrp::remove_customers(&instance, &[2], &mut solution, &mut context);

        // Reinsert customer 2
        let mut random = Random::new(42);
        Sdvrp::reinsert_customers(
            &instance,
            &[2],
            0.0,
            &mut solution,
            &mut context,
            &mut random,
        );

        let new_obj = Sdvrp::calc_objective(&instance, &solution);
        assert!(new_obj > 0);
    }
}
