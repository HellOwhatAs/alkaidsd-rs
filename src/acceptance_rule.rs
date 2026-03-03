//! Acceptance rules for solution improvement decisions.
//!
//! Acceptance rules determine whether a new solution should be accepted
//! based on objective value comparison and stochastic criteria.

use crate::random::Random;

/// Trait for acceptance rules in metaheuristics.
///
/// An acceptance rule decides whether to accept a new solution based on
/// the old and new objective values.
pub trait AcceptanceRule {
    /// Determines whether to accept a new solution.
    ///
    /// # Arguments
    ///
    /// * `old_value` - Objective value of the current solution
    /// * `new_value` - Objective value of the candidate solution
    /// * `random` - Random number generator for stochastic rules
    ///
    /// # Returns
    ///
    /// `true` if the new solution should be accepted
    fn accept(&mut self, old_value: i32, new_value: i32, random: &mut Random) -> bool;
}

/// Hill climbing acceptance rule.
///
/// Only accepts strictly improving solutions (lower objective value).
#[derive(Debug, Clone, Default)]
pub struct HillClimbing;

impl AcceptanceRule for HillClimbing {
    #[inline]
    fn accept(&mut self, old_value: i32, new_value: i32, _random: &mut Random) -> bool {
        new_value < old_value
    }
}

/// Hill climbing with equal acceptance.
///
/// Accepts improving solutions and equal solutions (allows sideways moves).
#[derive(Debug, Clone, Default)]
pub struct HillClimbingWithEqual;

impl AcceptanceRule for HillClimbingWithEqual {
    #[inline]
    fn accept(&mut self, old_value: i32, new_value: i32, _random: &mut Random) -> bool {
        new_value <= old_value
    }
}

/// Late Acceptance Hill Climbing (LAHC) rule.
///
/// Maintains a history of recent solutions and accepts new solutions
/// if they improve upon the current OR upon any solution in the history.
/// This allows escaping local optima by accepting solutions that are
/// better than solutions from several iterations ago.
///
/// # References
///
/// Burke, E. K., & Bykov, Y. (2017). The late acceptance hill-climbing
/// heuristic. European Journal of Operational Research.
#[derive(Debug, Clone)]
pub struct LateAcceptanceHillClimbing {
    /// History length
    length: usize,

    /// Current position in the history
    position: usize,

    /// History of objective values
    values: Vec<i32>,
}

impl LateAcceptanceHillClimbing {
    /// Creates a new LAHC acceptance rule with the given history length.
    ///
    /// # Arguments
    ///
    /// * `length` - Number of recent solutions to remember
    pub fn new(length: usize) -> Self {
        Self {
            length,
            position: 0,
            values: vec![i32::MAX; length],
        }
    }
}

impl AcceptanceRule for LateAcceptanceHillClimbing {
    fn accept(&mut self, old_value: i32, new_value: i32, _random: &mut Random) -> bool {
        let mut accepted = false;
        let mut current_value = old_value;

        // Accept if better than current OR better than historical value
        if new_value <= old_value || new_value < self.values[self.position] {
            current_value = new_value;
            accepted = true;
        }

        // Update history if current is better than stored
        if current_value < self.values[self.position] {
            self.values[self.position] = current_value;
        }

        // Advance position in circular buffer
        self.position = (self.position + 1) % self.length;

        accepted
    }
}

/// Simulated Annealing acceptance rule.
///
/// Accepts improving solutions always, and worse solutions with a
/// probability that decreases as the "temperature" cools down.
///
/// The probability of accepting a worse solution is:
/// `P = exp((old_value - new_value) / temperature)`
///
/// After each decision, temperature is multiplied by the decay factor.
#[derive(Debug, Clone)]
pub struct SimulatedAnnealing {
    /// Current temperature
    temperature: f64,

    /// Decay factor (0 < decay < 1)
    decay: f64,
}

impl SimulatedAnnealing {
    /// Creates a new Simulated Annealing acceptance rule.
    ///
    /// # Arguments
    ///
    /// * `initial_temperature` - Starting temperature (higher = more exploration)
    /// * `decay` - Temperature decay factor (e.g., 0.9999)
    pub fn new(initial_temperature: f64, decay: f64) -> Self {
        Self {
            temperature: initial_temperature,
            decay,
        }
    }
}

impl AcceptanceRule for SimulatedAnnealing {
    fn accept(&mut self, old_value: i32, new_value: i32, random: &mut Random) -> bool {
        let accepted = new_value <= old_value
            || (random.next_float() as f64) < ((old_value - new_value) as f64 / self.temperature).exp();

        self.temperature *= self.decay;
        accepted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hill_climbing() {
        let mut rule = HillClimbing;
        let mut rng = Random::new(42);

        assert!(rule.accept(100, 90, &mut rng));  // Better
        assert!(!rule.accept(100, 100, &mut rng)); // Equal
        assert!(!rule.accept(100, 110, &mut rng)); // Worse
    }

    #[test]
    fn test_hill_climbing_with_equal() {
        let mut rule = HillClimbingWithEqual;
        let mut rng = Random::new(42);

        assert!(rule.accept(100, 90, &mut rng));   // Better
        assert!(rule.accept(100, 100, &mut rng));  // Equal
        assert!(!rule.accept(100, 110, &mut rng)); // Worse
    }

    #[test]
    fn test_simulated_annealing() {
        let mut rule = SimulatedAnnealing::new(10000.0, 0.99);
        let mut rng = Random::new(42);

        // Better always accepted
        assert!(rule.accept(100, 90, &mut rng));

        // Worse may or may not be accepted (stochastic)
        let mut accepted_count = 0;
        for _ in 0..100 {
            let mut r = SimulatedAnnealing::new(10000.0, 0.99);
            if r.accept(100, 110, &mut rng) {
                accepted_count += 1;
            }
        }
        // With high temperature, should accept some worse solutions
        assert!(accepted_count > 0);
    }
}
