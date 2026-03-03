//! Star-based insertion caching for SWAP* and related operators.
//!
//! Maintains best insertion positions for each customer in each route,
//! enabling fast lookup during optimization.

use crate::cache::Cache;
use crate::delta::Delta;
use crate::instance::{Instance, Node};
use crate::random::Random;
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;
use std::any::Any;

/// A single insertion position with its delta.
#[derive(Debug, Clone, Default)]
pub struct Insertion {
    /// Delta (cost change) for this insertion
    pub delta: Delta<i32>,

    /// Predecessor node for insertion
    pub predecessor: Node,

    /// Successor node for insertion
    pub successor: Node,
}

/// Stores the best N insertion positions for a customer.
///
/// Keeping multiple positions allows finding the best position
/// that doesn't conflict with other moves.
#[derive(Debug, Clone)]
pub struct BestInsertion<const N: usize> {
    insertions: [Insertion; N],
}

impl<const N: usize> Default for BestInsertion<N> {
    fn default() -> Self {
        Self {
            insertions: std::array::from_fn(|_| Insertion {
                delta: Delta::new(i32::MAX, -1),
                predecessor: 0,
                successor: 0,
            }),
        }
    }
}

impl<const N: usize> BestInsertion<N> {
    /// Resets all insertions to maximum cost.
    pub fn reset(&mut self) {
        for insertion in &mut self.insertions {
            insertion.delta = Delta::new(i32::MAX, -1);
        }
    }

    /// Adds a potential insertion position.
    ///
    /// Maintains sorted order by delta value, keeping only the best N.
    pub fn add(&mut self, delta: i32, predecessor: Node, successor: Node, random: &mut Random) {
        for i in 0..N {
            if self.insertions[i].delta.value == i32::MAX {
                // Empty slot, insert here
                self.insertions[i] = Insertion {
                    delta: Delta::new(delta, 1),
                    predecessor,
                    successor,
                };
                return;
            } else if delta < self.insertions[i].delta.value {
                // Better than this position, shift others down
                for j in (i + 1..N).rev() {
                    self.insertions[j] = self.insertions[j - 1].clone();
                }
                self.insertions[i] = Insertion {
                    delta: Delta::new(delta, 1),
                    predecessor,
                    successor,
                };
                return;
            } else if delta == self.insertions[i].delta.value && self.insertions[i].delta.counter != -1 {
                // Equal value - use reservoir sampling
                if random.next_int(1, self.insertions[i].delta.counter + 1) == 1 {
                    // Selected: shift down and insert at position i
                    for j in (i + 1..N).rev() {
                        self.insertions[j] = self.insertions[j - 1].clone();
                    }
                    self.insertions[i].delta.counter += 1;
                    self.insertions[i].predecessor = predecessor;
                    self.insertions[i].successor = successor;
                    break;
                } else {
                    // Not selected: increment counter and continue to next position
                    self.insertions[i].delta.counter += 1;
                }
            }
        }
    }

    /// Returns the best insertion.
    pub fn find_best(&self) -> &Insertion {
        &self.insertions[0]
    }

    /// Returns the best insertion that doesn't involve a specific node.
    ///
    /// Used when the node being inserted would conflict with the cached position.
    pub fn find_best_without_node(&self, node_index: Node) -> Option<&Insertion> {
        self.insertions.iter().find(|insertion| {
            insertion.delta.counter > 0
                && insertion.predecessor != node_index
                && insertion.successor != node_index
        })
    }
}

/// Cache for star-based insertion positions.
///
/// Stores the best 3 insertion positions for each customer in each route.
#[derive(Default)]
pub struct StarCaches {
    /// caches[route][customer] = BestInsertion
    caches: Vec<Vec<BestInsertion<3>>>,

    /// Saved routes for cache invalidation detection
    routes: Vec<Vec<Node>>,
}

impl StarCaches {
    /// Preprocesses insertion positions for a route.
    ///
    /// Computes the best insertion positions for all customers in the given route.
    pub fn preprocess(
        &mut self,
        instance: &Instance,
        solution: &AlkaidSolution,
        context: &RouteContext,
        route: Node,
        random: &mut Random,
    ) {
        let route_idx = route as usize;

        // Ensure caches are large enough
        if self.caches.len() <= route_idx {
            self.caches.resize(route_idx + 1, Vec::new());
        }

        // Skip if already computed
        if !self.caches[route_idx].is_empty() {
            return;
        }

        // Initialize insertions for all customers
        self.caches[route_idx].resize(instance.num_customers as usize, BestInsertion::default());
        for customer in 1..instance.num_customers {
            self.caches[route_idx][customer as usize].reset();
        }

        // Compute insertion costs for each position
        let mut predecessor = 0;
        let mut successor = context.head(route);
        let route_cache = &mut self.caches[route_idx];

        loop {
            let pred_customer = solution.customer(predecessor);
            let succ_customer = solution.customer(successor);
            let pred_distances = unsafe { instance.distance_matrix.get_unchecked(pred_customer as usize) };
            let succ_distances = unsafe { instance.distance_matrix.get_unchecked(succ_customer as usize) };
            let distance = instance.distance(pred_customer, succ_customer);

            for customer in 1..instance.num_customers {
                let delta = unsafe { *pred_distances.get_unchecked(customer as usize) }
                    + unsafe { *succ_distances.get_unchecked(customer as usize) }
                    - distance;
                route_cache[customer as usize].add(delta, predecessor, successor, random);
            }

            if successor == 0 {
                break;
            }
            predecessor = successor;
            successor = solution.successor(successor);
        }
    }

    /// Gets the best insertions for a customer in a route.
    pub fn get(&self, route_index: Node, customer: Node) -> &BestInsertion<3> {
        &self.caches[route_index as usize][customer as usize]
    }
}

impl Cache for StarCaches {
    fn reset(&mut self, solution: &AlkaidSolution, context: &RouteContext) {
        self.caches.resize(context.num_routes() as usize, Vec::new());

        for route_index in 0..self.caches.len().min(self.routes.len()) as Node {
            let mut same_route = false;

            if (route_index as usize) < context.num_routes() as usize {
                same_route = true;
                let mut head = context.head(route_index);

                for &node in &self.routes[route_index as usize] {
                    if head != node {
                        same_route = false;
                        break;
                    }
                    head = solution.successor(head);
                }

                if head != 0 {
                    same_route = false;
                }
            }

            if !same_route {
                self.caches[route_index as usize].clear();
            }
        }
    }

    fn add_route(&mut self, route_index: Node) {
        if self.caches.len() <= route_index as usize {
            self.caches.resize(route_index as usize + 1, Vec::new());
        }
    }

    fn remove_route(&mut self, route_index: Node) {
        if (route_index as usize) < self.caches.len() {
            self.caches[route_index as usize].clear();
        }
    }

    fn move_route(&mut self, dest_route_index: Node, src_route_index: Node) {
        if dest_route_index as usize >= self.caches.len() {
            self.caches.resize(dest_route_index as usize + 1, Vec::new());
        }
        // Use take and replace pattern to avoid double borrow
        let src = std::mem::take(&mut self.caches[src_route_index as usize]);
        self.caches[dest_route_index as usize] = src;
    }

    fn save(&mut self, solution: &AlkaidSolution, context: &RouteContext) {
        self.routes.resize(context.num_routes() as usize, Vec::new());

        for route_index in 0..self.routes.len() as Node {
            let route = &mut self.routes[route_index as usize];
            route.clear();

            let mut node = context.head(route_index);
            while node != 0 {
                route.push(node);
                node = solution.successor(node);
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
