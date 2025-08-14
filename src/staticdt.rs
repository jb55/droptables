use rand::Rng;

use crate::IndexSampler;

/// Generated table backed by an **index sampler** and a **static slice** of items.
///
/// - No per-table `Vec<T>` allocation.
/// - Great for enums (macro emits a `&'static [T]`).
/// - Can sample by reference **or** by value (if `T: Copy`).
#[derive(Debug, Clone, Copy)]
pub struct StaticDropTable<S: IndexSampler, T: 'static> {
    sampler: S,
    items: &'static [T],
}

impl<S: IndexSampler, T> StaticDropTable<S, T> {
    pub const fn new(sampler: S, items: &'static [T]) -> Self {
        Self { sampler, items }
    }

    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.sampler.len()
    }

    /// Borrowed sample (zero clone).
    #[inline]
    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> &'static T {
        let i = self.sampler.sample_index(rng);
        &self.items[i]
    }

    /// Owned sample (requires `T: Copy`).
    #[inline]
    pub fn sample_owned<R: Rng + ?Sized>(&self, rng: &mut R) -> T
    where
        T: Copy,
    {
        let i = self.sampler.sample_index(rng);
        self.items[i]
    }

    /// Access the backing slice.
    #[inline]
    pub const fn items(&self) -> &'static [T] {
        self.items
    }
}
