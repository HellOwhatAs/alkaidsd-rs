//! Utility functions and types for optimization operators.
//!
//! This module provides helper structures for tracking insertion positions
//! and calculating fleet lower bounds.

use crate::delta::Delta;
use crate::instance::{Instance, Node};
use crate::random::Random;
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;

/// Tracks the best insertion position with its cost.
///
/// # Type Parameters
///
/// * `T` - The cost type (typically `i32` or `f32`)
#[derive(Debug, Clone)]
pub struct InsertionWithCost<T: Default + Copy + PartialOrd> {
    /// Predecessor node for insertion
    pub predecessor: Node,

    /// Successor node for insertion
    pub successor: Node,

    /// Route index where insertion would occur
    pub route_index: Node,

    /// Cost (delta) of this insertion
    pub cost: Delta<T>,
}

impl<T: Default + Copy + PartialOrd> Default for InsertionWithCost<T> {
    fn default() -> Self {
        Self {
            predecessor: 0,
            successor: 0,
            route_index: 0,
            cost: Delta::default(),
        }
    }
}

impl<T: Default + Copy + PartialOrd> InsertionWithCost<T> {
    /// Updates this insertion if the other one is better.
    ///
    /// # Arguments
    ///
    /// * `insertion` - The candidate insertion to compare
    /// * `random` - Random number generator for tie-breaking
    ///
    /// # Returns
    ///
    /// `true` if this insertion was updated
    #[inline]
    pub fn update(&mut self, insertion: &InsertionWithCost<T>, random: &mut Random) -> bool {
        if self.cost.update_from(&insertion.cost, random) {
            self.predecessor = insertion.predecessor;
            self.successor = insertion.successor;
            self.route_index = insertion.route_index;
            true
        } else {
            false
        }
    }
}

/// Calculates the best insertion position for a customer in a route.
///
/// # Arguments
///
/// * `solution` - The current solution
/// * `func` - Cost function (predecessor, successor, customer) -> cost
/// * `context` - Route context
/// * `route_index` - Index of the route to consider
/// * `customer` - Customer to insert
/// * `random` - Random number generator for tie-breaking
///
/// # Returns
///
/// The best insertion position with its cost
#[inline]
pub fn calc_best_insertion<T, F>(
    solution: &AlkaidSolution,
    func: F,
    context: &RouteContext,
    route_index: Node,
    customer: Node,
    random: &mut Random,
) -> InsertionWithCost<T>
where
    T: Default + Copy + PartialOrd,
    F: Fn(Node, Node, Node) -> T,
{
    let head = context.head(route_index);

    // Try insertion at the beginning (after depot)
    let head_cost = func(0, head, customer);
    let mut best_insertion = InsertionWithCost {
        predecessor: 0,
        successor: head,
        route_index,
        cost: Delta::new(head_cost, 1),
    };

    // Try insertion after each node in the route
    let mut node_index = head;
    while node_index != 0 {
        let successor = solution.successor(node_index);
        let cost = func(node_index, successor, customer);
        if best_insertion.cost.update(cost, random) {
            best_insertion.predecessor = node_index;
            best_insertion.successor = successor;
        }
        node_index = successor;
    }

    best_insertion
}

/// Calculates a lower bound on the number of vehicles needed.
///
/// This is a simple bound based on total demand divided by capacity.
///
/// # Arguments
///
/// * `instance` - The problem instance
///
/// # Returns
///
/// Minimum number of vehicles needed
#[inline]
pub fn calc_fleet_lower_bound(instance: &Instance) -> Node {
    let sum_demands: i32 = instance.demands.iter().skip(1).sum();
    ((sum_demands + instance.capacity - 1) / instance.capacity) as Node
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fleet_lower_bound() {
        let instance = Instance {
            num_customers: 3,
            capacity: 100,
            demands: vec![0, 150, 200], // Total = 350, need 4 vehicles
            distance_matrix: vec![vec![0; 3]; 3],
        };

        assert_eq!(calc_fleet_lower_bound(&instance), 4);
    }

    #[test]
    fn test_fleet_lower_bound_exact() {
        let instance = Instance {
            num_customers: 3,
            capacity: 100,
            demands: vec![0, 100, 100], // Total = 200, need exactly 2 vehicles
            distance_matrix: vec![vec![0; 3]; 3],
        };

        assert_eq!(calc_fleet_lower_bound(&instance), 2);
    }
}
