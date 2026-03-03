//! Sorter module for ordering customers during perturbation.
//!
//! Different sorting strategies can affect solution quality by
//! influencing the order in which customers are reinserted.

use crate::instance::{Instance, Node};
use crate::random::Random;

/// Trait for sorting operators.
///
/// Sort operators define different strategies for ordering customers
/// during the repair phase of the algorithm.
pub trait SortOperator {
    /// Sorts the given vector of customers in place.
    ///
    /// # Arguments
    ///
    /// * `instance` - The problem instance
    /// * `customers` - Mutable slice of customer indices to sort
    /// * `random` - Random number generator
    fn sort(&self, instance: &Instance, customers: &mut [Node], random: &mut Random);
}

/// A weighted collection of sort operators.
///
/// Randomly selects a sort operator based on assigned weights.
#[derive(Default)]
pub struct Sorter {
    /// Sum of all weights
    sum_weights: f64,

    /// Sort operators with their weights
    sort_functions: Vec<(Box<dyn SortOperator>, f64)>,
}

impl Sorter {
    /// Creates a new empty sorter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a sort operator with the specified weight.
    ///
    /// # Arguments
    ///
    /// * `sort_function` - The sort operator
    /// * `weight` - Selection weight (higher = more likely to be chosen)
    pub fn add_sort_function(&mut self, sort_function: Box<dyn SortOperator>, weight: f64) {
        self.sum_weights += weight;
        self.sort_functions.push((sort_function, weight));
    }

    /// Sorts customers using a randomly selected operator.
    ///
    /// Selection is weighted by the assigned weights.
    ///
    /// # Arguments
    ///
    /// * `instance` - The problem instance
    /// * `customers` - Mutable slice of customer indices to sort
    /// * `random` - Random number generator
    pub fn sort(&self, instance: &Instance, customers: &mut [Node], random: &mut Random) {
        let mut r = random.next_float() as f64 * self.sum_weights;

        for (sort_function, weight) in &self.sort_functions {
            r -= weight;
            if r < 0.0 {
                sort_function.sort(instance, customers, random);
                return;
            }
        }
    }
}

/// Randomly shuffles customers.
#[derive(Debug, Clone, Default)]
pub struct SortByRandom;

impl SortOperator for SortByRandom {
    fn sort(&self, _instance: &Instance, customers: &mut [Node], random: &mut Random) {
        random.shuffle(customers);
    }
}

/// Sorts customers by demand in descending order.
///
/// Customers with higher demands are inserted first.
#[derive(Debug, Clone, Default)]
pub struct SortByDemand;

impl SortOperator for SortByDemand {
    fn sort(&self, instance: &Instance, customers: &mut [Node], _random: &mut Random) {
        customers.sort_by(|&a, &b| {
            instance.demands[b as usize].cmp(&instance.demands[a as usize])
        });
    }
}

/// Sorts customers by distance from depot in descending order.
///
/// Customers farther from the depot are inserted first.
#[derive(Debug, Clone, Default)]
pub struct SortByFar;

impl SortOperator for SortByFar {
    fn sort(&self, instance: &Instance, customers: &mut [Node], _random: &mut Random) {
        customers.sort_by(|&a, &b| {
            instance.distance_matrix[0][b as usize].cmp(&instance.distance_matrix[0][a as usize])
        });
    }
}

/// Sorts customers by distance from depot in ascending order.
///
/// Customers closer to the depot are inserted first.
#[derive(Debug, Clone, Default)]
pub struct SortByClose;

impl SortOperator for SortByClose {
    fn sort(&self, instance: &Instance, customers: &mut [Node], _random: &mut Random) {
        customers.sort_by(|&a, &b| {
            instance.distance_matrix[0][a as usize].cmp(&instance.distance_matrix[0][b as usize])
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_instance() -> Instance {
        Instance {
            num_customers: 4,
            capacity: 100,
            demands: vec![0, 30, 50, 20],
            distance_matrix: vec![
                vec![0, 10, 20, 30],
                vec![10, 0, 15, 25],
                vec![20, 15, 0, 10],
                vec![30, 25, 10, 0],
            ],
        }
    }

    #[test]
    fn test_sort_by_demand() {
        let instance = test_instance();
        let mut customers = vec![1, 2, 3];
        let mut rng = Random::new(42);

        SortByDemand.sort(&instance, &mut customers, &mut rng);
        assert_eq!(customers, vec![2, 1, 3]); // 50 > 30 > 20
    }

    #[test]
    fn test_sort_by_far() {
        let instance = test_instance();
        let mut customers = vec![1, 2, 3];
        let mut rng = Random::new(42);

        SortByFar.sort(&instance, &mut customers, &mut rng);
        assert_eq!(customers, vec![3, 2, 1]); // 30 > 20 > 10 from depot
    }

    #[test]
    fn test_sort_by_close() {
        let instance = test_instance();
        let mut customers = vec![1, 2, 3];
        let mut rng = Random::new(42);

        SortByClose.sort(&instance, &mut customers, &mut rng);
        assert_eq!(customers, vec![1, 2, 3]); // 10 < 20 < 30 from depot
    }
}
