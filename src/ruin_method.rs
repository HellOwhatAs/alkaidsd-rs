//! Ruin methods for destroying parts of a solution.
//!
//! Ruin methods remove customers from the solution, creating opportunities
//! for improvement when they are reinserted.

use crate::instance::{Instance, Node};
use crate::random::Random;
use crate::route_context::RouteContext;
use crate::solution::AlkaidSolution;
use std::collections::HashSet;

/// Trait for ruin methods.
///
/// A ruin method removes customers from the solution to allow
/// exploration of different configurations during the repair phase.
pub trait RuinMethod {
    /// Removes customers from the solution.
    ///
    /// # Arguments
    ///
    /// * `instance` - The problem instance
    /// * `solution` - The solution to ruin (not modified here)
    /// * `context` - Route context
    /// * `random` - Random number generator
    ///
    /// # Returns
    ///
    /// A vector of customer indices that should be removed and reinserted
    fn ruin(
        &mut self,
        instance: &Instance,
        solution: &AlkaidSolution,
        context: &RouteContext,
        random: &mut Random,
    ) -> Vec<Node>;
}

/// Random ruin method.
///
/// Randomly selects a number of customers to remove.
#[derive(Debug, Clone)]
pub struct RandomRuin {
    /// Possible numbers of customers to perturb
    num_perturb_customers: Vec<i32>,
}

impl RandomRuin {
    /// Creates a new random ruin method.
    ///
    /// # Arguments
    ///
    /// * `num_perturb_customers` - List of possible perturbation sizes
    pub fn new(num_perturb_customers: Vec<i32>) -> Self {
        Self { num_perturb_customers }
    }
}

impl RuinMethod for RandomRuin {
    fn ruin(
        &mut self,
        instance: &Instance,
        _solution: &AlkaidSolution,
        _context: &RouteContext,
        random: &mut Random,
    ) -> Vec<Node> {
        // Select how many customers to perturb
        let num_perturb = self.num_perturb_customers
            [random.next_int(0, self.num_perturb_customers.len() as i32 - 1) as usize];

        // Create list of all customers (excluding depot)
        let mut customers: Vec<Node> = (1..instance.num_customers).collect();
        random.shuffle(&mut customers);

        // Return first num_perturb customers
        customers.truncate(num_perturb as usize);
        customers
    }
}

/// SISRs (Slack Induction by String Removals) ruin method.
///
/// Removes strings of consecutive customers from routes, centered around
/// a seed customer. This creates more structured perturbations that may
/// preserve some route structure.
///
/// # References
///
/// Christiaens, J., & Vanden Berghe, G. (2020). Slack induction by string
/// removals for vehicle routing problems.
#[derive(Debug, Clone)]
pub struct SisrsRuin {
    /// Average number of customers to remove
    average_customers: i32,

    /// Maximum string length
    max_length: i32,

    /// Probability of splitting removed strings
    split_rate: f64,

    /// Probability of preserving a node during split
    preserved_probability: f64,
}

impl SisrsRuin {
    /// Creates a new SISRs ruin method.
    ///
    /// # Arguments
    ///
    /// * `average_customers` - Target average number of customers to remove
    /// * `max_length` - Maximum length of removed strings
    /// * `split_rate` - Probability of creating gaps in removed strings
    /// * `preserved_probability` - Probability of preserving each gap position
    pub fn new(
        average_customers: i32,
        max_length: i32,
        split_rate: f64,
        preserved_probability: f64,
    ) -> Self {
        Self {
            average_customers,
            max_length,
            split_rate,
            preserved_probability,
        }
    }

    /// Gets the route head for a given node and returns position.
    fn get_route_head(solution: &AlkaidSolution, node_index: Node) -> (Node, i32) {
        let mut position = 0;
        let mut current = node_index;

        loop {
            let predecessor = solution.predecessor(current);
            if predecessor == 0 {
                return (current, position);
            }
            current = predecessor;
            position += 1;
        }
    }

    /// Extracts the route starting from a head node.
    fn get_route(solution: &AlkaidSolution, head: Node) -> Vec<Node> {
        let mut route = Vec::new();
        let mut current = head;

        while current != 0 {
            route.push(current);
            current = solution.successor(current);
        }

        route
    }
}

impl RuinMethod for SisrsRuin {
    fn ruin(
        &mut self,
        instance: &Instance,
        solution: &AlkaidSolution,
        context: &RouteContext,
        random: &mut Random,
    ) -> Vec<Node> {
        // Calculate parameters
        let average_length = (instance.num_customers - 1) as f64 / context.num_routes() as f64;
        let max_length = (self.max_length as f64).min(average_length);
        let max_strings = 4.0 * self.average_customers as f64 / (1.0 + self.max_length as f64) - 1.0;
        let num_strings = (random.next_float() as f64 * max_strings) as usize + 1;

        // Select seed customer
        let customer_seed = random.next_int(1, instance.num_customers as i32 - 1) as Node;
        let seed_distances = &instance.distance_matrix[customer_seed as usize];

        // Sort nodes by distance from seed
        let mut node_indices: Vec<Node> = solution.node_indices().to_vec();
        node_indices.sort_by(|&a, &b| {
            seed_distances[solution.customer(a) as usize]
                .cmp(&seed_distances[solution.customer(b) as usize])
        });

        // Visit routes and collect customers to remove
        let mut visited_heads = HashSet::new();
        let mut customer_indices = Vec::new();

        for node_index in node_indices {
            if visited_heads.len() >= num_strings {
                break;
            }

            let (head, position) = Self::get_route_head(solution, node_index);

            if !visited_heads.insert(head) {
                continue;
            }

            let route = Self::get_route(solution, head);
            let route_length = route.len() as i32;

            let max_ruin_length = (route_length as f64).min(max_length);
            let mut ruin_length = (random.next_float() as f64 * max_ruin_length) as i32 + 1;

            // Possibly create gaps in the string
            let mut num_preserved = 0;
            let mut preserved_start_position = -1i32;

            if ruin_length >= 2 && ruin_length < route_length && (random.next_float() as f64) < self.split_rate {
                while ruin_length < route_length {
                    if (random.next_float() as f64) < self.preserved_probability {
                        break;
                    }
                    num_preserved += 1;
                    ruin_length += 1;
                }
                preserved_start_position = random.next_int(1, ruin_length - num_preserved - 1);
            }

            // Calculate start position
            let min_start_position = 0.max(position - ruin_length + 1);
            let max_start_position = (route_length - ruin_length).min(position);
            let start_position = random.next_int(min_start_position, max_start_position);

            // Collect customers to remove (skipping preserved section)
            for j in 0..ruin_length {
                if preserved_start_position < 0
                    || j < preserved_start_position
                    || j >= preserved_start_position + num_preserved
                {
                    let idx = (start_position + j) as usize;
                    customer_indices.push(solution.customer(route[idx]));
                }
            }
        }

        // Remove duplicates and shuffle
        customer_indices.sort_unstable();
        customer_indices.dedup();
        random.shuffle(&mut customer_indices);

        customer_indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_instance() -> Instance {
        Instance {
            num_customers: 6,
            capacity: 100,
            demands: vec![0, 20, 30, 25, 15, 10],
            distance_matrix: vec![
                vec![0, 10, 20, 30, 40, 50],
                vec![10, 0, 15, 25, 35, 45],
                vec![20, 15, 0, 10, 20, 30],
                vec![30, 25, 10, 0, 10, 20],
                vec![40, 35, 20, 10, 0, 10],
                vec![50, 45, 30, 20, 10, 0],
            ],
        }
    }

    #[test]
    fn test_random_ruin() {
        let instance = test_instance();
        let solution = AlkaidSolution::new();
        let context = RouteContext::new();
        let mut random = Random::new(42);

        let mut ruin = RandomRuin::new(vec![2, 3]);
        let customers = ruin.ruin(&instance, &solution, &context, &mut random);

        assert!(customers.len() >= 2 && customers.len() <= 3);
        for c in &customers {
            assert!(*c >= 1 && *c < instance.num_customers);
        }
    }
}
