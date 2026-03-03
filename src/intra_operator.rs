//! Intra-route operators for improving individual routes.
//!
//! These operators modify nodes within a single route to reduce cost.

use crate::delta::Delta;
use crate::instance::{Instance, Node};
use crate::random::Random;
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;

/// Trait for intra-route operators.
///
/// Intra-route operators improve a single route by rearranging its nodes.
pub trait IntraOperator {
    /// Attempts to improve the given route.
    ///
    /// # Arguments
    ///
    /// * `instance` - The problem instance
    /// * `route_index` - Index of the route to improve
    /// * `solution` - The solution to modify
    /// * `context` - Route context
    /// * `random` - Random number generator
    ///
    /// # Returns
    ///
    /// `true` if an improving move was found and applied
    fn apply(
        &self,
        instance: &Instance,
        route_index: Node,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
        random: &mut Random,
    ) -> bool;
}

/// Exchange operator.
///
/// Swaps the positions of two non-adjacent nodes within a route.
#[derive(Debug, Clone, Default)]
pub struct Exchange;

/// Move data for Exchange operator.
struct ExchangeMove {
    node_a: Node,
    node_b: Node,
}

impl Exchange {
    /// Evaluates an exchange move between two nodes.
    fn evaluate_inner(
        instance: &Instance,
        solution: &AlkaidSolution,
        node_a: Node,
        node_b: Node,
        best_move: &mut Option<ExchangeMove>,
        best_delta: &mut Delta<i32>,
        random: &mut Random,
    ) {
        let predecessor_a = solution.predecessor(node_a);
        let successor_a = solution.successor(node_a);
        let predecessor_b = solution.predecessor(node_b);
        let successor_b = solution.successor(node_b);

        let delta = instance.distance(solution.customer(predecessor_a), solution.customer(node_b))
            + instance.distance(solution.customer(node_b), solution.customer(successor_a))
            + instance.distance(solution.customer(predecessor_b), solution.customer(node_a))
            + instance.distance(solution.customer(node_a), solution.customer(successor_b))
            - instance.distance(solution.customer(predecessor_a), solution.customer(node_a))
            - instance.distance(solution.customer(node_a), solution.customer(successor_a))
            - instance.distance(solution.customer(predecessor_b), solution.customer(node_b))
            - instance.distance(solution.customer(node_b), solution.customer(successor_b));

        if best_delta.update(delta, random) {
            *best_move = Some(ExchangeMove { node_a, node_b });
        }
    }

    /// Applies an exchange move.
    fn do_exchange(
        mv: &ExchangeMove,
        route_index: Node,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
    ) {
        let predecessor_a = solution.predecessor(mv.node_a);
        let successor_a = solution.successor(mv.node_a);
        let predecessor_b = solution.predecessor(mv.node_b);
        let successor_b = solution.successor(mv.node_b);

        // Swap positions
        solution.link(predecessor_a, mv.node_b);
        solution.link(mv.node_b, successor_a);
        solution.link(predecessor_b, mv.node_a);
        solution.link(mv.node_a, successor_b);

        // Update route head if necessary
        if predecessor_a == 0 {
            context.set_head(route_index, mv.node_b);
        }

        context.update_route_context(solution, route_index, predecessor_a);
    }
}

impl IntraOperator for Exchange {
    fn apply(
        &self,
        instance: &Instance,
        route_index: Node,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
        random: &mut Random,
    ) -> bool {
        let mut best_move: Option<ExchangeMove> = None;
        let mut best_delta = Delta::default();

        let mut node_a = context.head(route_index);
        while node_a != 0 {
            let mut node_b = solution.successor(node_a);
            if node_b != 0 {
                // Skip adjacent node
                node_b = solution.successor(node_b);
                while node_b != 0 {
                    Self::evaluate_inner(
                        instance, solution, node_a, node_b,
                        &mut best_move, &mut best_delta, random,
                    );
                    node_b = solution.successor(node_b);
                }
            }
            node_a = solution.successor(node_a);
        }

        if best_delta.value < 0 {
            if let Some(mv) = best_move {
                Self::do_exchange(&mv, route_index, solution, context);
                return true;
            }
        }

        false
    }
}

/// Or-Opt operator.
///
/// Moves a sequence of consecutive nodes to a new position.
/// The generic parameter `NUM` specifies the sequence length (1, 2, or 3).
#[derive(Debug, Clone, Default)]
pub struct OrOpt<const NUM: usize>;

/// Move data for Or-Opt operator.
struct OrOptMove {
    reversed: bool,
    head: Node,
    tail: Node,
    predecessor: Node,
    successor: Node,
}

impl<const NUM: usize> OrOpt<NUM> {
    /// Evaluates an Or-Opt move.
    #[allow(clippy::too_many_arguments)]
    fn evaluate_inner(
        instance: &Instance,
        solution: &AlkaidSolution,
        head: Node,
        tail: Node,
        predecessor: Node,
        successor: Node,
        best_move: &mut Option<OrOptMove>,
        best_delta: &mut Delta<i32>,
        random: &mut Random,
    ) {
        let predecessor_head = solution.predecessor(head);
        let successor_tail = solution.successor(tail);

        let mut delta = instance.distance(solution.customer(predecessor_head), solution.customer(successor_tail))
            - instance.distance(solution.customer(predecessor_head), solution.customer(head))
            - instance.distance(solution.customer(tail), solution.customer(successor_tail))
            - instance.distance(solution.customer(predecessor), solution.customer(successor));

        let mut reversed = false;

        let insertion_delta = instance.distance(solution.customer(predecessor), solution.customer(head))
            + instance.distance(solution.customer(successor), solution.customer(tail));

        let mut best_insertion_delta = insertion_delta;

        if NUM > 1 {
            let reversed_delta = instance.distance(solution.customer(predecessor), solution.customer(tail))
                + instance.distance(solution.customer(successor), solution.customer(head));

            if reversed_delta < insertion_delta {
                best_insertion_delta = reversed_delta;
                reversed = true;
            }
        }

        delta += best_insertion_delta;

        if best_delta.update(delta, random) {
            *best_move = Some(OrOptMove {
                reversed,
                head,
                tail,
                predecessor,
                successor,
            });
        }
    }

    /// Applies an Or-Opt move.
    fn do_or_opt(
        mv: &OrOptMove,
        route_index: Node,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
    ) {
        let predecessor_head = solution.predecessor(mv.head);
        let successor_tail = solution.successor(mv.tail);

        // Save current head for restoration
        solution.set_successor(0, context.head(route_index));

        // Remove segment from current position
        solution.link(predecessor_head, successor_tail);

        // Insert segment at new position
        if !mv.reversed {
            solution.link(mv.predecessor, mv.head);
            solution.link(mv.tail, mv.successor);
        } else {
            solution.reversed_link(mv.head, mv.tail, mv.predecessor, mv.successor);
        }

        // Update route head
        context.set_head(route_index, solution.successor(0));
    }
}

impl<const NUM: usize> IntraOperator for OrOpt<NUM> {
    fn apply(
        &self,
        instance: &Instance,
        route_index: Node,
        solution: &mut AlkaidSolution,
        context: &mut RouteContext,
        random: &mut Random,
    ) -> bool {
        let mut best_move: Option<OrOptMove> = None;
        let mut best_delta = Delta::default();

        let mut head = context.head(route_index);
        let mut tail = head;

        // Move tail to create segment of length NUM
        for _ in 1..NUM {
            if tail == 0 {
                return false;
            }
            tail = solution.successor(tail);
        }

        while tail != 0 {
            // Try all positions after the segment
            let mut predecessor = solution.successor(tail);
            while predecessor != 0 {
                let successor = solution.successor(predecessor);
                Self::evaluate_inner(
                    instance, solution, head, tail,
                    predecessor, successor,
                    &mut best_move, &mut best_delta, random,
                );
                predecessor = successor;
            }

            // Try all positions before the segment
            let mut successor = solution.predecessor(head);
            while successor != 0 {
                let predecessor = solution.predecessor(successor);
                Self::evaluate_inner(
                    instance, solution, head, tail,
                    predecessor, successor,
                    &mut best_move, &mut best_delta, random,
                );
                successor = predecessor;
            }

            head = solution.successor(head);
            tail = solution.successor(tail);
        }

        if best_delta.value < 0 {
            if let Some(mv) = best_move {
                Self::do_or_opt(&mv, route_index, solution, context);
                context.update_route_context(solution, route_index, 0);
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Instance, AlkaidSolution, RouteContext) {
        let instance = Instance {
            num_customers: 5,
            capacity: 200,
            demands: vec![0, 20, 30, 25, 15],
            distance_matrix: vec![
                vec![0, 10, 20, 30, 40],
                vec![10, 0, 5, 25, 35],
                vec![20, 5, 0, 10, 20],
                vec![30, 25, 10, 0, 10],
                vec![40, 35, 20, 10, 0],
            ],
        };

        let mut solution = AlkaidSolution::new();
        let node1 = solution.insert(1, 20, 0, 0);
        let node2 = solution.insert(2, 30, node1, 0);
        let node3 = solution.insert(3, 25, node2, 0);
        solution.insert(4, 15, node3, 0);

        let mut context = RouteContext::new();
        context.calc_route_context(&solution);

        (instance, solution, context)
    }

    #[test]
    fn test_exchange_operator() {
        let (instance, mut solution, mut context) = setup();
        let mut random = Random::new(42);

        let exchange = Exchange;
        let initial_cost = solution.calc_objective(&instance);

        // Apply exchange multiple times
        for _ in 0..5 {
            exchange.apply(&instance, 0, &mut solution, &mut context, &mut random);
        }

        let final_cost = solution.calc_objective(&instance);
        assert!(final_cost <= initial_cost);
    }

    #[test]
    fn test_or_opt_1() {
        let (instance, mut solution, mut context) = setup();
        let mut random = Random::new(42);

        let or_opt = OrOpt::<1>;
        let initial_cost = solution.calc_objective(&instance);

        // Apply or-opt multiple times
        for _ in 0..5 {
            or_opt.apply(&instance, 0, &mut solution, &mut context, &mut random);
        }

        let final_cost = solution.calc_objective(&instance);
        assert!(final_cost <= initial_cost);
    }
}
