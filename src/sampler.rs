use rand::Rng;

use crate::{IndexSampler, error::ProbError, walker::WeightedSampler};

/// Uniform index sampler: picks an index in `0..n` with equal probability.
#[derive(Debug, Clone, Copy)]
pub struct UniformSampler {
    n: usize,
}

impl UniformSampler {
    pub fn new(n: usize) -> Result<Self, ProbError> {
        if n == 0 {
            return Err(ProbError::Empty);
        }
        Ok(Self { n })
    }
}

impl IndexSampler for UniformSampler {
    #[inline]
    fn len(&self) -> usize {
        self.n
    }
    #[inline]
    fn sample_index<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        rng.random_range(0..self.n)
    }
}

/// `WeightedSampler` is the weighted sampler; wire it into the trait.
impl IndexSampler for WeightedSampler {
    #[inline]
    fn len(&self) -> usize {
        // call the inherent method explicitly to avoid trait-recursion
        WeightedSampler::len(self)
    }
    #[inline]
    fn sample_index<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        // call the inherent method explicitly to avoid trait-recursion
        WeightedSampler::sample_index(self, rng)
    }
}
