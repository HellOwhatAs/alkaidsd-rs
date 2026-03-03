//! Random number generator implementation.
//!
//! This module provides a fast, deterministic PRNG based on the xoshiro128** algorithm.
//! The implementation is designed to be compatible with the C++ version to ensure
//! reproducible results across both implementations.

/// A fast pseudo-random number generator based on xoshiro128**.
///
/// This PRNG provides high-quality randomness with a period of 2^128 - 1,
/// suitable for optimization algorithms requiring reproducible results.
///
/// # Example
///
/// ```
/// use alkaidsd::random::Random;
///
/// let mut rng = Random::new(42);
/// let value = rng.next_int(0, 100);
/// let float_val = rng.next_float();
/// ```
pub struct Random {
    /// Internal state (4 x 32-bit words)
    state: [u32; 4],
}

impl Random {
    /// Returns the internal state for debugging purposes.
    #[inline]
    pub fn state(&self) -> [u32; 4] {
        self.state
    }
}

impl Random {
    /// Constants for float conversion (2^-24)
    /// Equivalent to 0x1.0p-24 in C/C++
    const FLOAT_MULTIPLIER: f32 = 5.9604645e-8;

    /// Power of 2^32 for modulo operations
    const POW32: u64 = 1u64 << 32;

    /// Creates a new random number generator with the given seed.
    ///
    /// The seed is scrambled to initialize the internal state, ensuring
    /// that similar seeds produce different sequences.
    ///
    /// # Arguments
    ///
    /// * `seed` - Initial seed value
    #[inline]
    pub fn new(seed: u32) -> Self {
        let mut state = [0u32; 4];
        state[0] = seed;
        for i in 1..4 {
            state[i] = Self::scramble_well(state[i - 1] as u64, i as u32) as u32;
        }
        Self { state }
    }

    /// Generates a random integer in the range [a, b] (inclusive).
    ///
    /// # Arguments
    ///
    /// * `a` - Lower bound (inclusive)
    /// * `b` - Upper bound (inclusive)
    ///
    /// # Returns
    ///
    /// A random integer uniformly distributed in [a, b]
    #[inline]
    pub fn next_int(&mut self, a: i32, b: i32) -> i32 {
        let range = (b - a + 1) as u32;
        self.next_int_bounded(range) as i32 + a
    }

    /// Generates a random float in the range [0, 1).
    ///
    /// The result has approximately 24 bits of precision.
    #[inline]
    pub fn next_float(&mut self) -> f32 {
        (self.next_raw() >> 8) as f32 * Self::FLOAT_MULTIPLIER
    }

    /// Shuffles a slice in place using the Fisher-Yates algorithm.
    ///
    /// # Arguments
    ///
    /// * `slice` - The slice to shuffle
    ///
    /// # Type Parameters
    ///
    /// * `T` - Element type (any type that can be moved)
    #[inline]
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        let n = slice.len();
        for i in (1..n).rev() {
            let j = self.next_int(0, i as i32) as usize;
            slice.swap(i, j);
        }
    }

    /// Rotate left helper function.
    #[inline]
    fn rotate_left(x: u32, k: u32) -> u32 {
        x.rotate_left(k)
    }

    /// Scramble function for seed initialization.
    #[inline]
    fn scramble(n: u64, multiple: u64, shift: u32, add: u32) -> u64 {
        multiple.wrapping_mul(n ^ (n >> shift)).wrapping_add(add as u64)
    }

    /// Well-known scrambling function for seed initialization.
    #[inline]
    fn scramble_well(n: u64, add: u32) -> u64 {
        Self::scramble(n, 1812433253, 30, add)
    }

    /// Generates a raw 32-bit random value using xoshiro128**.
    #[inline]
    fn next_raw(&mut self) -> u32 {
        let result = Self::rotate_left(self.state[0].wrapping_add(self.state[3]), 7)
            .wrapping_add(self.state[0]);

        let t = self.state[1] << 9;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];
        self.state[2] ^= t;
        self.state[3] = Self::rotate_left(self.state[3], 11);

        result
    }

    /// Generates a random integer in [0, n) using rejection sampling
    /// for unbiased distribution (Lemire's method).
    #[inline]
    fn next_int_bounded(&mut self, n: u32) -> u32 {
        let mut m = (self.next_raw() as u64).wrapping_mul(n as u64);
        let mut l = m & 0xffffffff;

        if l < n as u64 {
            let t = Self::POW32 % n as u64;
            while l < t {
                m = (self.next_raw() as u64 & 0xffffffff).wrapping_mul(n as u64);
                l = m & 0xffffffff;
            }
        }
        (m >> 32) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_deterministic() {
        let mut rng1 = Random::new(42);
        let mut rng2 = Random::new(42);

        for _ in 0..100 {
            assert_eq!(rng1.next_int(0, 1000), rng2.next_int(0, 1000));
        }
    }

    #[test]
    fn test_random_range() {
        let mut rng = Random::new(12345);

        for _ in 0..1000 {
            let val = rng.next_int(10, 20);
            assert!(val >= 10 && val <= 20);
        }
    }

    #[test]
    fn test_random_float_range() {
        let mut rng = Random::new(12345);

        for _ in 0..1000 {
            let val = rng.next_float();
            assert!(val >= 0.0 && val < 1.0);
        }
    }
}
