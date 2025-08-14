//! Walker's Alias Method for O(1) sampling from a discrete distribution.

use crate::error::ProbError;
use rand::Rng;

/// Alias table for discrete distribution sampling.
#[derive(Debug, Clone)]
pub struct AliasTable {
    prob: Vec<f64>,
    alias: Vec<usize>,
}

impl AliasTable {
    /// Construct an alias table from non-negative weights. O(n).
    pub fn new(weights: &[f64]) -> Result<Self, ProbError> {
        let n = weights.len();
        if n == 0 {
            return Err(ProbError::Empty);
        }

        let mut sum = 0.0f64;
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
        let mut scaled: Vec<f64> = weights.iter().map(|&w| w * n as f64 / sum).collect();

        let mut prob = vec![0.0f64; n];
        let mut alias = (0..n).collect::<Vec<_>>();

        let mut small = Vec::with_capacity(n);
        let mut large = Vec::with_capacity(n);

        for (i, &p) in scaled.iter().enumerate() {
            if p < 1.0 {
                small.push(i);
            } else {
                large.push(i);
            }
        }

        while let (Some(s), Some(l)) = (small.pop(), large.pop()) {
            prob[s] = scaled[s]; // in [0,1)
            alias[s] = l;

            scaled[l] = (scaled[l] + scaled[s]) - 1.0;

            if scaled[l] < 1.0 - 1e-15 {
                small.push(l);
            } else {
                large.push(l);
            }
        }

        for i in small.into_iter().chain(large.into_iter()) {
            prob[i] = 1.0;
            alias[i] = i;
        }

        Ok(Self { prob, alias })
    }

    /// Draw a single sample in O(1).
    pub fn sample_index<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let n = self.prob.len();
        let i = rng.random_range(0..n); // replaces deprecated gen_range
        let u: f64 = rng.random(); // replaces deprecated r#gen()/gen()
        if u < self.prob[i] { i } else { self.alias[i] }
    }

    /// Draw k samples, returning counts per index (useful for checks).
    #[cfg(test)]
    pub fn sample_counts<R: Rng + ?Sized>(&self, rng: &mut R, draws: usize) -> Vec<usize> {
        let mut counts = vec![0usize; self.prob.len()];
        for _ in 0..draws {
            let i = self.sample(rng);
            counts[i] += 1;
        }
        counts
    }

    pub fn len(&self) -> usize {
        self.prob.len()
    }
    pub fn is_empty(&self) -> bool {
        self.prob.is_empty()
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

        let sum_w: f64 = weights.iter().sum();
        for (i, &c) in counts.iter().enumerate() {
            let p = weights[i] / sum_w;
            let emp = c as f64 / draws as f64;
            assert!((emp - p).abs() < 0.05, "i={i} emp={emp} p={p}");
        }
    }

    #[test]
    fn degenerate_singleton() {
        let alias = AliasTable::new(&[5.0]).unwrap();
        let mut rng = rand::rng();
        for _ in 0..1000 {
            assert_eq!(alias.sample(&mut rng), 0);
        }
    }
}
