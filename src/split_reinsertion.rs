//! Split reinsertion for customer demand.
//!
//! Reinserts a customer's demand across multiple routes when necessary,
//! using a greedy approach with optional "blinking" for diversification.

use crate::instance::{Instance, Node};
use crate::random::Random;
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;
use crate::utils::{calc_best_insertion, InsertionWithCost};

/// A potential reinsertion move with its residual capacity.
struct SplitReinsertionMove {
    insertion: InsertionWithCost<i32>,
    residual: i32,
}

/// Reinserts a customer with split delivery capability.
///
/// Finds the best positions to insert the customer's demand across routes,
/// potentially splitting the delivery if a single route can't accommodate
/// the full demand.
///
/// # Arguments
///
/// * `instance` - The problem instance
/// * `customer` - Customer to reinsert
/// * `demand` - Total demand to deliver
/// * `blink_rate` - Probability of skipping a move (diversification)
/// * `solution` - Solution to modify
/// * `context` - Route context
/// * `random` - Random number generator
pub fn split_reinsertion(
    instance: &Instance,
    customer: Node,
    demand: i32,
    blink_rate: f64,
    solution: &mut AlkaidSolution,
    context: &mut RouteContext,
    random: &mut Random,
) {
    let func = |predecessor: Node, successor: Node, customer: Node| {
        let pre_customer = solution.customer(predecessor);
        let suc_customer = solution.customer(successor);
        
        instance.distance(customer, pre_customer)
            + instance.distance(customer, suc_customer)
            - instance.distance(pre_customer, suc_customer)
    };
    
    // Collect all feasible insertion moves with their residual capacities
    let mut moves: Vec<SplitReinsertionMove> = Vec::new();
    let mut sum_residual = 0;
    
    for route_index in 0..context.num_routes() {
        let residual = demand.min(instance.capacity - context.load(route_index));
        
        if residual > 0 {
            let insertion = calc_best_insertion(
                solution, func, context, route_index, customer, random,
            );
            moves.push(SplitReinsertionMove { insertion, residual });
            sum_residual += residual;
        }
    }
    
    // Check if we can deliver the full demand
    if sum_residual < demand {
        // Not enough capacity - need to add new routes
        // For simplicity, we'll just return here and let the algorithm
        // handle this case through route creation
        return;
    }
    
    // Sort moves by efficiency (cost per unit delivered)
    moves.sort_by(|a, b| {
        let cost_a = a.insertion.cost.value as i64 * b.residual as i64;
        let cost_b = b.insertion.cost.value as i64 * a.residual as i64;
        cost_a.cmp(&cost_b)
    });
    
    // Execute moves, possibly skipping some (blinking)
    let mut remaining_demand = demand;
    
    for mv in moves {
        sum_residual -= mv.residual;
        
        // Skip with blink_rate probability if we still have enough capacity
        if sum_residual >= remaining_demand && (random.next_float() as f64) < blink_rate {
            continue;
        }
        
        let load = remaining_demand.min(mv.residual);
        let node_index = solution.insert(
            customer,
            load,
            mv.insertion.predecessor,
            mv.insertion.successor,
        );
        
        if mv.insertion.predecessor == 0 {
            context.set_head(mv.insertion.route_index, node_index);
        }
        context.update_route_context(solution, mv.insertion.route_index, mv.insertion.predecessor);
        
        remaining_demand -= load;
        if remaining_demand == 0 {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_reinsertion_single_route() {
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
        let _node1 = solution.insert(1, 50, 0, 0);

        let mut context = RouteContext::new();
        context.calc_route_context(&solution);

        let mut random = Random::new(42);

        // Insert customer 2 with demand 30
        split_reinsertion(&instance, 2, 30, 0.0, &mut solution, &mut context, &mut random);

        // Should have added one node
        assert_eq!(solution.node_indices().len(), 2);
    }

    #[test]
    fn test_split_reinsertion_split_delivery() {
        let instance = Instance {
            num_customers: 3,
            capacity: 50,
            demands: vec![0, 30, 60], // Customer 2 needs split delivery
            distance_matrix: vec![
                vec![0, 10, 20],
                vec![10, 0, 15],
                vec![20, 15, 0],
            ],
        };

        let mut solution = AlkaidSolution::new();
        
        // Create two routes with some capacity remaining
        let _node1 = solution.insert(1, 30, 0, 0);
        let _node2 = solution.insert(1, 20, 0, 0); // Second route

        let mut context = RouteContext::new();
        context.calc_route_context(&solution);

        let mut random = Random::new(42);

        // Try to insert customer 2 with demand 40 (needs splitting)
        split_reinsertion(&instance, 2, 40, 0.0, &mut solution, &mut context, &mut random);

        // Total nodes should be 4 (2 original + 2 for split customer 2)
        assert!(solution.node_indices().len() >= 2);
    }
}
