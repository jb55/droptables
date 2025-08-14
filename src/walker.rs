//! Walker's Alias Method for O(1) sampling from a discrete distribution.
//!
//! The alias table stores, for each bucket `i`, a primary probability `prob[i]`
//! and a fallback index `alias[i]`. A single uniform draw over buckets, plus
//! a single uniform draw in `[0,1)`, yields an index in O(1).
//!
//! ## Construction
//! Given non-negative weights `w`, we scale them so that the **average** is 1.
//! Buckets with `p < 1` are matched with buckets with `p >= 1` until all are
//! resolved; remaining buckets get probability 1 and alias to themselves.
//!
//! ## Properties
//! * **Build**: O(n)
//! * **Sample**: O(1)
//! * **Space**: ~`(f32 + usize) * n`
//!
//! See [`AliasTable::new`] for input validation.

use crate::error::ProbError;
use rand::Rng;

/// Alias table for discrete distribution sampling.
///
/// Construct with [`AliasTable::new`], then draw using
/// [`AliasTable::sample_index`].
#[derive(Debug, Clone)]
pub struct AliasTable {
    probs: Vec<Bucket>,
}

#[repr(C)]
#[derive(Default, Debug, Clone)]
struct Bucket {
    prob: f32,  // f32 is typically plenty here
    alias: u32, // if n <= u32::MAX
}

impl AliasTable {
    /// Construct an alias table from non-negative weights. **O(n)**.
    ///
    /// # Errors
    /// * [`ProbError::Empty`] if `weights` is empty
    /// * [`ProbError::Negative`] if any weight is negative (includes `-0.0`)
    /// * [`ProbError::ZeroSum`] if the sum is zero or not finite (`NaN`/∞)
    ///
    /// # Notes
    /// * Inputs are normalized internally; original scale doesn’t matter.
    /// * We apply a small tolerance (`1e-15`) to avoid numerical flip-flops.
    pub fn new(weights: &[f32]) -> Result<Self, ProbError> {
        let n = weights.len();
        if n == 0 {
            return Err(ProbError::Empty);
        }

        let mut sum = 0.0f32;
        for (i, &w) in weights.iter().enumerate() {
            if w.is_sign_negative() {
                return Err(ProbError::Negative { index: i, value: w });
            }
            sum += w;
        }
        if !sum.is_finite() || sum == 0.0 {
            return Err(ProbError::ZeroSum);
        }

        // Scale so average is 1.
        let mut scaled: Vec<f32> = weights.iter().map(|&w| w * n as f32 / sum).collect();

        let mut probs = Vec::with_capacity(n);
        for i in 0..n {
            probs.push(Bucket {
                prob: 0.0,
                alias: i as u32,
            });
        }

        let mut small: Vec<u32> = Vec::with_capacity(n);
        let mut large: Vec<u32> = Vec::with_capacity(n);

        for (i, &p) in scaled.iter().enumerate() {
            if p < 1.0 {
                small.push(i as u32);
            } else {
                large.push(i as u32);
            }
        }

        while let (Some(s), Some(l)) = (small.pop(), large.pop()) {
            probs[s as usize].prob = scaled[s as usize]; // in [0,1)
            probs[s as usize].alias = l;

            scaled[l as usize] = (scaled[l as usize] + scaled[s as usize]) - 1.0;

            if scaled[l as usize] < 1.0 - 1e-15 {
                small.push(l);
            } else {
                large.push(l);
            }
        }

        for i in small.into_iter().chain(large.into_iter()) {
            probs[i as usize].prob = 1.0;
            probs[i as usize].alias = i;
        }

        Ok(Self { probs })
    }

    /// Draw a single sample **index** in O(1).
    ///
    /// # Examples
    /// ```rust,ignore
    /// use rand::Rng;
    /// # use droptables::AliasTable;
    /// let alias = AliasTable::new(&[1.0, 2.0, 3.0]).unwrap();
    /// let mut rng = rand::rng();
    /// let i = alias.sample_index(&mut rng);
    /// assert!(i < 3);
    /// ```
    pub fn sample_index<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let n = self.probs.len();
        let i = rng.random_range(0..n); // replaces deprecated gen_range
        let u: f32 = rng.random(); // replaces deprecated r#gen()/gen()
        if u < self.probs[i].prob {
            i
        } else {
            self.probs[i].alias as usize
        }
    }

    /// Draw k samples, returning counts per index (useful for checks).
    #[cfg(test)]
    pub fn sample_counts<R: Rng + ?Sized>(&self, rng: &mut R, draws: usize) -> Vec<usize> {
        let mut counts = vec![0usize; self.probs.len()];
        for _ in 0..draws {
            let i = self.sample_index(rng);
            counts[i] += 1;
        }
        counts
    }

    /// Number of categories in the table.
    pub fn len(&self) -> usize {
        self.probs.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.probs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng, rngs::StdRng};

    #[test]
    fn rejects_bad_inputs() {
        assert!(matches!(AliasTable::new(&[]), Err(ProbError::Empty)));
        assert!(matches!(
            AliasTable::new(&[0.0, 0.0]),
            Err(ProbError::ZeroSum)
        ));
        assert!(matches!(
            AliasTable::new(&[-0.1, 0.2]),
            Err(ProbError::Negative { .. })
        ));
    }

    #[test]
    fn roughly_matches_distribution() {
        let weights = [1.0, 2.0, 3.0, 4.0];
        let alias = AliasTable::new(&weights).unwrap();

        let mut rng = StdRng::seed_from_u64(42);
        let draws = 2_000_0usize; // keep test light; raise locally if you like
        let counts = alias.sample_counts(&mut rng, draws);

        let sum_w: f32 = weights.iter().sum();
        for (i, &c) in counts.iter().enumerate() {
            let p = weights[i] / sum_w;
            let emp = c as f32 / draws as f32;
            assert!((emp - p).abs() < 0.05, "i={i} emp={emp} p={p}");
        }
    }

    #[test]
    fn degenerate_singleton() {
        let alias = AliasTable::new(&[5.0]).unwrap();
        let mut rng = rand::rng();
        for _ in 0..1000 {
            assert_eq!(alias.sample_index(&mut rng), 0);
        }
    }
}
