//! Delta accumulator for tie-breaking in optimization.
//!
//! The Delta type tracks the best value found and a counter for how many times
//! that value was encountered. This enables random tie-breaking among equally
//! good solutions, improving solution diversity.

use crate::random::Random;

/// A value accumulator with tie-breaking support.
///
/// When multiple moves have the same improvement value (delta), this structure
/// enables fair random selection among them by counting occurrences and using
/// reservoir sampling.
///
/// # Type Parameters
///
/// * `T` - The value type (typically `i32` or `f32`)
///
/// # Example
///
/// ```
/// use alkaidsd::delta::Delta;
/// use alkaidsd::random::Random;
///
/// let mut delta = Delta::<i32>::default();
/// let mut rng = Random::new(42);
///
/// // First update with negative value (improvement) wins
/// assert!(delta.update(-10, &mut rng));
///
/// // Better (lower) value wins
/// assert!(delta.update(-15, &mut rng));
///
/// // Equal values are randomly selected
/// let updated = delta.update(-15, &mut rng);
/// // updated may be true or false depending on RNG
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Delta<T> {
    /// The current best value.
    pub value: T,

    /// Counter for tie-breaking. -1 means uninitialized.
    /// When counter > 0, it tracks how many times we've seen this value.
    pub counter: i32,
}

impl<T: Default> Default for Delta<T> {
    /// Creates a default Delta with default value and counter = -1 (uninitialized).
    #[inline]
    fn default() -> Self {
        Self {
            value: T::default(),
            counter: -1,
        }
    }
}

impl<T: Copy> Delta<T> {
    /// Creates a new Delta with the given value and counter.
    ///
    /// # Arguments
    ///
    /// * `value` - The initial value
    /// * `counter` - The initial counter (usually 1 for first observation)
    #[inline]
    pub fn new(value: T, counter: i32) -> Self {
        Self { value, counter }
    }
}

impl<T: PartialOrd + Copy> Delta<T> {
    /// Updates the delta with a new value, implementing reservoir sampling for ties.
    ///
    /// # Arguments
    ///
    /// * `new_value` - The new value to compare
    /// * `random` - Random number generator for tie-breaking
    ///
    /// # Returns
    ///
    /// `true` if this new value was selected (either better or won the tie-break)
    #[inline]
    pub fn update(&mut self, new_value: T, random: &mut Random) -> bool {
        if new_value < self.value {
            // New value is strictly better
            self.value = new_value;
            self.counter = 1;
            true
        } else if new_value == self.value && self.counter != -1 {
            // Tie: use reservoir sampling (1/n probability of selecting new)
            self.counter += 1;
            random.next_int(1, self.counter) == 1
        } else {
            false
        }
    }

    /// Updates this delta from another delta, merging counters for ties.
    ///
    /// This is useful when combining results from multiple independent searches.
    ///
    /// # Arguments
    ///
    /// * `delta` - The other delta to merge
    /// * `random` - Random number generator for tie-breaking
    ///
    /// # Returns
    ///
    /// `true` if the other delta's value was selected
    #[inline]
    pub fn update_from(&mut self, delta: &Delta<T>, random: &mut Random) -> bool {
        if delta.value < self.value {
            // Other delta is strictly better
            self.value = delta.value;
            self.counter = delta.counter;
            true
        } else if delta.value == self.value && self.counter != -1 {
            // Merge counters and select proportionally
            self.counter += delta.counter;
            random.next_int(1, self.counter) <= delta.counter
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_first_update_negative() {
        let mut delta = Delta::<i32>::default();
        let mut rng = Random::new(42);

        // First update with negative value should succeed (improvement)
        assert!(delta.update(-10, &mut rng));
        assert_eq!(delta.value, -10);
        assert_eq!(delta.counter, 1);
    }

    #[test]
    fn test_delta_first_update_positive_rejected() {
        let mut delta = Delta::<i32>::default();
        let mut rng = Random::new(42);

        // Rejected because 100 is not < 0 (the default initial value)
        assert!(!delta.update(100, &mut rng));
        assert_eq!(delta.value, 0); // unchanged
        assert_eq!(delta.counter, -1); // still uninitialized
    }

    #[test]
    fn test_delta_better_value() {
        let mut delta = Delta::new(100, 1);
        let mut rng = Random::new(42);

        // Better (lower) value should always win
        assert!(delta.update(50, &mut rng));
        assert_eq!(delta.value, 50);
        assert_eq!(delta.counter, 1);
    }

    #[test]
    fn test_delta_worse_value() {
        let mut delta = Delta::new(50, 1);
        let mut rng = Random::new(42);

        // Worse (higher) value should never win
        assert!(!delta.update(100, &mut rng));
        assert_eq!(delta.value, 50);
        assert_eq!(delta.counter, 1);
    }
}
