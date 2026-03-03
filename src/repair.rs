//! Repair heuristic for consolidating duplicate customer visits.
//!
//! After modifications, a route may contain multiple visits to the same
//! customer. This module merges them to reduce cost.

use crate::instance::{Instance, Node};
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;
use std::collections::HashMap;

/// Merges adjacent nodes serving the same customer.
fn merge_adjacent_same_customers(
    _instance: &Instance,
    route_index: Node,
    solution: &mut AlkaidSolution,
    context: &RouteContext,
) {
    let mut node_index = context.head(route_index);
    
    loop {
        let successor = solution.successor(node_index);
        if successor == 0 {
            break;
        }
        
        if solution.customer(node_index) == solution.customer(successor) {
            // Merge loads and remove successor
            let combined_load = solution.load(node_index) + solution.load(successor);
            solution.set_load(node_index, combined_load);
            solution.remove(successor);
        } else {
            node_index = successor;
        }
    }
}

/// Calculates the cost delta for removing a node from its position.
fn calc_removal_delta(instance: &Instance, solution: &AlkaidSolution, node_index: Node) -> i32 {
    let predecessor = solution.predecessor(node_index);
    let successor = solution.successor(node_index);
    
    instance.distance(solution.customer(predecessor), solution.customer(successor))
        - instance.distance(solution.customer(predecessor), solution.customer(node_index))
        - instance.distance(solution.customer(node_index), solution.customer(successor))
}

/// Repairs a route by consolidating multiple visits to the same customer.
///
/// For each customer visited multiple times in the route, keeps only the
/// visit that would cost the least to remove (i.e., the one in the best
/// position), and consolidates the load there.
///
/// # Arguments
///
/// * `instance` - The problem instance
/// * `route_index` - Index of the route to repair
/// * `solution` - The solution to modify
/// * `context` - Route context (will be updated)
pub fn repair(
    instance: &Instance,
    route_index: Node,
    solution: &mut AlkaidSolution,
    context: &mut RouteContext,
) {
    if context.head(route_index) == 0 {
        return;
    }
    
    // First, merge adjacent same-customer nodes
    merge_adjacent_same_customers(instance, route_index, solution, context);
    
    // Track best node for each customer
    let mut customer_node_map: HashMap<Node, Node> = HashMap::new();
    
    let mut node_index = context.head(route_index);
    
    // Set up depot successor for iteration
    solution.set_successor(0, node_index);
    
    while node_index != 0 {
        let successor = solution.successor(node_index);
        let customer = solution.customer(node_index);
        
        if let Some(&last_node_index) = customer_node_map.get(&customer) {
            // Customer already visited - decide which to keep
            let mut best_node = last_node_index;
            let mut remove_node = node_index;
            
            if calc_removal_delta(instance, solution, last_node_index)
                < calc_removal_delta(instance, solution, node_index)
            {
                std::mem::swap(&mut best_node, &mut remove_node);
                customer_node_map.insert(customer, best_node);
            }
            
            // Consolidate load and remove duplicate
            let combined_load = solution.load(best_node) + solution.load(remove_node);
            solution.set_load(best_node, combined_load);
            solution.remove(remove_node);
        } else {
            customer_node_map.insert(customer, node_index);
        }
        
        node_index = successor;
    }
    
    // Update context
    context.set_head(route_index, solution.successor(0));
    context.update_route_context(solution, route_index, 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repair_consolidates_duplicates() {
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
        
        // Create route with duplicate customer visits: 1 -> 2 -> 1
        let node1 = solution.insert(1, 25, 0, 0);
        let node2 = solution.insert(2, 30, node1, 0);
        let _node3 = solution.insert(1, 25, node2, 0);  // Duplicate

        let mut context = RouteContext::new();
        context.calc_route_context(&solution);

        // Before repair: 3 nodes
        assert_eq!(solution.node_indices().len(), 3);

        repair(&instance, 0, &mut solution, &mut context);

        // After repair: 2 nodes (duplicates merged)
        assert_eq!(solution.node_indices().len(), 2);
    }
}
