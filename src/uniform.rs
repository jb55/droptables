use crate::StaticDropTable;
use crate::error::ProbError;
use crate::sampler::UniformSampler;
use rand::Rng;

/// A compact uniform drop table: all items are equally likely.
/// Space: just the items (no alias/prob arrays).
#[derive(Debug, Clone)]
pub struct UniformTable<T> {
    items: Vec<T>,
}

impl<T> UniformTable<T> {
    /// Build from any iterator of items. Errors if empty.
    pub fn from_items<I>(items: I) -> Result<Self, ProbError>
    where
        I: IntoIterator<Item = T>,
    {
        Ok(Self {
            items: items.into_iter().collect(),
        })
    }

    /// Convenience for arrays.
    pub fn from_array<const N: usize>(items: [T; N]) -> Result<Self, ProbError>
    where
        T: Clone,
    {
        if N == 0 {
            return Err(ProbError::Empty);
        }
        Ok(Self {
            items: items.to_vec(),
        })
    }

    /// Number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Sample **by reference**.
    pub fn sample<'a, R: Rng + ?Sized>(&'a self, rng: &mut R) -> &'a T {
        let i = rng.random_range(0..self.items.len());
        &self.items[i]
    }

    /// Sample **by value** (clones).
    pub fn sample_owned<R: Rng + ?Sized>(&self, rng: &mut R) -> T
    where
        T: Clone,
    {
        self.items[rng.random_range(0..self.items.len())].clone()
    }

    /// Expose items if you need them.
    pub fn as_slice(&self) -> &[T] {
        &self.items
    }
}

/// Trait implemented by the `UniformEnum` derive macro.
///
/// Exposes the variants as a static slice and provides a zero-storage
/// uniform droptable over them.
pub trait UniformEnum: Sized + 'static {
    /// All variants in declaration order.
    const VARS: &'static [Self];

    /// Zero-alloc, zero-clone uniform table backed by `UniformSampler` and a
    /// `&'static [Self]`. Requires `Copy` so we can offer `.sample_owned()`.
    fn droptable() -> Result<StaticDropTable<UniformSampler, Self>, ProbError>
    where
        Self: Copy + 'static,
    {
        let sampler = UniformSampler::new(Self::VARS.len())?;
        Ok(StaticDropTable::new(sampler, Self::VARS))
    }

    /// If you explicitly want an owning Vec-backed table (allocates),
    /// use this. Handy if you don’t have `'static` or don’t want `Copy`.
    fn droptable_stateful() -> Result<UniformTable<Self>, ProbError>
    where
        Self: Clone,
    {
        // build from the static slice into a Vec
        UniformTable::from_items(Self::VARS.iter().cloned())
    }
}
