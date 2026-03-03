//! Construction heuristic for initial solution generation.
//!
//! Builds an initial feasible solution using insertion heuristics.

use crate::delta::Delta;
use crate::instance::{Instance, Node};
use crate::random::Random;
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;
use crate::utils::calc_fleet_lower_bound;

/// Candidate for insertion: (customer, demand)
type CandidateList = Vec<(Node, i32)>;

/// Insertion criteria.
#[derive(Clone, Copy)]
enum Criterion {
    Mcfic { gamma: f32 },
    Nfic,
}

/// Calculates insertion cost based on criterion.
fn calc_insertion_cost(
    instance: &Instance,
    solution: &AlkaidSolution,
    predecessor: Node,
    successor: Node,
    customer: Node,
    criterion: Criterion,
) -> f32 {
    let pre_customer = solution.customer(predecessor);
    let suc_customer = solution.customer(successor);

    match criterion {
        Criterion::Mcfic { gamma } => {
            (instance.distance(pre_customer, customer)
                + instance.distance(customer, suc_customer)
                - instance.distance(pre_customer, suc_customer)) as f32
                - 2.0 * gamma * instance.distance(0, customer) as f32
        }
        Criterion::Nfic => {
            if pre_customer == 0 {
                f32::MAX
            } else {
                instance.distance(pre_customer, customer) as f32
            }
        }
    }
}

/// Finds best insertion for a customer in a route.
fn find_best_insertion(
    instance: &Instance,
    solution: &AlkaidSolution,
    context: &RouteContext,
    route_index: Node,
    customer: Node,
    criterion: Criterion,
    random: &mut Random,
) -> (Node, Node, Delta<f32>) {
    let head = context.head(route_index);
    let head_cost = calc_insertion_cost(instance, solution, 0, head, customer, criterion);
    
    let mut best_predecessor = 0;
    let mut best_successor = head;
    let mut best_delta = Delta::new(head_cost, 1);

    let mut node_index = head;
    while node_index != 0 {
        let successor = solution.successor(node_index);
        let cost = calc_insertion_cost(instance, solution, node_index, successor, customer, criterion);
        
        if best_delta.update(cost, random) {
            best_predecessor = node_index;
            best_successor = successor;
        }
        node_index = successor;
    }

    (best_predecessor, best_successor, best_delta)
}

/// Adds a new route with a random candidate.
fn add_route(
    candidate_list: &mut CandidateList,
    random: &mut Random,
    solution: &mut AlkaidSolution,
    context: &mut RouteContext,
) -> usize {
    let position = random.next_int(0, candidate_list.len() as i32 - 1) as usize;
    let (customer, demand) = candidate_list[position];
    let node_index = solution.insert(customer, demand, 0, 0);

    candidate_list[position] = *candidate_list.last().unwrap();
    candidate_list.pop();

    context.add_route(node_index, node_index, demand);
    position
}

/// Insertion with metadata.
#[derive(Clone)]
struct InsertionInfo {
    predecessor: Node,
    successor: Node,
    route_index: Node,
    cost: Delta<f32>,
    candidate_position: Option<usize>,
}

impl Default for InsertionInfo {
    fn default() -> Self {
        Self {
            predecessor: 0,
            successor: 0,
            route_index: 0,
            cost: Delta::new(f32::MAX, -1),
            candidate_position: None,
        }
    }
}

/// Sequential insertion strategy.
fn sequential_insertion(
    instance: &Instance,
    criterion: Criterion,
    candidate_list: &mut CandidateList,
    random: &mut Random,
    solution: &mut AlkaidSolution,
    context: &mut RouteContext,
) {
    let mut is_full = vec![false; context.num_routes() as usize];

    while !candidate_list.is_empty() {
        let mut inserted = false;

        for route_index in 0..context.num_routes() {
            if is_full[route_index as usize] {
                continue;
            }

            let mut best = InsertionInfo::default();

            for (i, &(customer, demand)) in candidate_list.iter().enumerate() {
                if context.load(route_index) + demand > instance.capacity {
                    continue;
                }

                let (pred, succ, delta) = find_best_insertion(
                    instance, solution, context, route_index, customer, criterion, random,
                );

                if best.cost.update_from(&delta, random) {
                    best.predecessor = pred;
                    best.successor = succ;
                    best.route_index = route_index;
                    best.candidate_position = Some(i);
                }
            }

            if let Some(pos) = best.candidate_position {
                let (customer, demand) = candidate_list[pos];
                candidate_list[pos] = *candidate_list.last().unwrap();
                candidate_list.pop();

                let node_index = solution.insert(customer, demand, best.predecessor, best.successor);

                if best.predecessor == 0 {
                    context.set_head(route_index, node_index);
                }
                context.add_load(route_index, demand);
                inserted = true;
            } else {
                is_full[route_index as usize] = true;
            }
        }

        if !inserted {
            add_route(candidate_list, random, solution, context);
            is_full.push(false);
        }
    }
}

/// Parallel insertion strategy.
///
/// Pre-computes best insertion positions for ALL candidates across ALL routes
/// before inserting. This allows finding the globally best insertion.
fn parallel_insertion(
    instance: &Instance,
    criterion: Criterion,
    candidate_list: &mut CandidateList,
    random: &mut Random,
    solution: &mut AlkaidSolution,
    context: &mut RouteContext,
) {
    // best_insertions[candidate][route] = InsertionInfo
    let mut best_insertions: Vec<Vec<InsertionInfo>> = Vec::with_capacity(candidate_list.len());
    
    // Pre-compute best insertions for each candidate in each route
    for (candidate_idx, &(customer, _demand)) in candidate_list.iter().enumerate() {
        best_insertions.push(Vec::with_capacity(context.num_routes() as usize));
        for route_index in 0..context.num_routes() {
            let (pred, succ, delta) = find_best_insertion(
                instance, solution, context, route_index, customer, criterion, random,
            );
            best_insertions[candidate_idx].push(InsertionInfo {
                predecessor: pred,
                successor: succ,
                route_index,
                cost: delta,
                candidate_position: Some(candidate_idx),
            });
        }
    }
    
    let mut updated = vec![false; context.num_routes() as usize];
    
    while !candidate_list.is_empty() {
        let mut best = InsertionInfo::default();
        let mut best_candidate_position: Option<usize> = None;
        
        // Find globally best insertion
        for i in 0..best_insertions.len() {
            let mut j = 0;
            while j < best_insertions[i].len() {
                let route_index = best_insertions[i][j].route_index;
                let (_customer, demand) = candidate_list[i];
                
                // Check capacity constraint
                if context.load(route_index) + demand > instance.capacity {
                    // Remove this route option
                    best_insertions[i].swap_remove(j);
                    continue;
                }
                
                // Update if route has changed
                if updated[route_index as usize] {
                    let (pred, succ, delta) = find_best_insertion(
                        instance, solution, context, route_index, candidate_list[i].0, criterion, random,
                    );
                    best_insertions[i][j].predecessor = pred;
                    best_insertions[i][j].successor = succ;
                    best_insertions[i][j].cost = delta;
                }
                
                if best.cost.update_from(&best_insertions[i][j].cost, random) {
                    best = best_insertions[i][j].clone();
                    best_candidate_position = Some(i);
                }
                
                j += 1;
            }
        }
        
        if let Some(candidate_pos) = best_candidate_position {
            let (customer, demand) = candidate_list[candidate_pos];
            
            // Remove candidate from list (swap with last)
            let last_candidate_idx = candidate_list.len() - 1;
            if candidate_pos != last_candidate_idx {
                candidate_list[candidate_pos] = candidate_list[last_candidate_idx];
            }
            candidate_list.pop();
            
            // Remove candidate's insertions (swap with last)
            let last_insertion_idx = best_insertions.len() - 1;
            if candidate_pos != last_insertion_idx {
                best_insertions[candidate_pos] = best_insertions.pop().unwrap();
                // Update candidate_position for swapped candidate
                for info in &mut best_insertions[candidate_pos] {
                    info.candidate_position = Some(candidate_pos);
                }
            } else {
                best_insertions.pop();
            }
            
            // Insert customer
            let node_index = solution.insert(customer, demand, best.predecessor, best.successor);
            let route_index = best.route_index;
            
            if best.predecessor == 0 {
                context.set_head(route_index, node_index);
            }
            context.add_load(route_index, demand);
            updated[route_index as usize] = true;
        } else {
            // No valid insertion found, add new route
            let position = add_route(candidate_list, random, solution, context);
            
            // Keep best_insertions in sync with candidate_list (swap-remove)
            let last_idx = best_insertions.len() - 1;
            if position != last_idx {
                best_insertions[position] = best_insertions.pop().unwrap();
                // Update candidate_position for swapped candidate
                for info in &mut best_insertions[position] {
                    info.candidate_position = Some(position);
                }
            } else {
                best_insertions.pop();
            }
            
            // Add new route insertions for remaining candidates
            let new_route_index = context.num_routes() - 1;
            for (i, &(customer, _demand)) in candidate_list.iter().enumerate() {
                let (pred, succ, delta) = find_best_insertion(
                    instance, solution, context, new_route_index, customer, criterion, random,
                );
                best_insertions[i].push(InsertionInfo {
                    predecessor: pred,
                    successor: succ,
                    route_index: new_route_index,
                    cost: delta,
                    candidate_position: Some(i),
                });
            }
            updated.push(false);
        }
    }
}

/// Insert candidates using randomly selected strategy.
fn insert_candidates(
    instance: &Instance,
    criterion: Criterion,
    candidate_list: &mut CandidateList,
    random: &mut Random,
    solution: &mut AlkaidSolution,
    context: &mut RouteContext,
) {
    let strategy = random.next_int(0, 1);
    if strategy == 0 {
        sequential_insertion(instance, criterion, candidate_list, random, solution, context);
    } else {
        parallel_insertion(instance, criterion, candidate_list, random, solution, context);
    }
}

/// Constructs an initial solution using randomized insertion heuristics.
///
/// # Arguments
///
/// * `instance` - The problem instance
/// * `random` - Random number generator
///
/// # Returns
///
/// An initial feasible solution
pub fn construct(instance: &Instance, random: &mut Random) -> AlkaidSolution {
    let mut candidate_list: CandidateList = Vec::new();
    let num_fleets = calc_fleet_lower_bound(instance);

    for i in 1..instance.num_customers {
        let mut demand = instance.demands[i as usize];
        while demand > 0 {
            let split_demand = demand.min(instance.capacity);
            candidate_list.push((i, split_demand));
            demand -= split_demand;
        }
    }

    let mut solution = AlkaidSolution::new();
    let mut context = RouteContext::new();

    for _ in 0..num_fleets {
        if candidate_list.is_empty() {
            break;
        }
        add_route(&mut candidate_list, random, &mut solution, &mut context);
    }

    // Select criterion
    let criterion = if random.next_int(0, 1) == 0 {
        let gamma = random.next_int(0, 34) as f32 * 0.05;
        Criterion::Mcfic { gamma }
    } else {
        Criterion::Nfic
    };

    insert_candidates(instance, criterion, &mut candidate_list, random, &mut solution, &mut context);

    solution
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construct() {
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
        let solution = construct(&instance, &mut random);

        assert!(!solution.node_indices().is_empty());
        let obj = solution.calc_objective(&instance);
        assert!(obj > 0);
    }
}
