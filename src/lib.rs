//! # droptables
//!
//! Weighted random selection that’s *fast* and ergonomic.
//!
//! This crate wraps a compact implementation of
//! [Walker’s Alias Method](https://en.wikipedia.org/wiki/Alias_method)
//! to provide O(1) sampling from a fixed discrete distribution.
//!
//! There are two primary ways to use it:
//!
//! 1. **Ad-hoc pairs** with [`DropTable::from_pairs`]
//! 2. **Compile-time enums** with the [`WeightedEnum`] derive macro (from the
//!    companion `droptables_macros` crate), which turns an enum into a drop table.
//!
//! ## Quick start (pairs)
//!
//! ```rust,ignore
//! use rand::Rng;
//! use droptables::DropTable;
//!
//! # fn main() {
//! let table = DropTable::from_pairs([
//!     ("common", 60.0),
//!     ("uncommon", 30.0),
//!     ("rare", 9.0),
//!     ("legendary", 1.0),
//! ]).unwrap();
//!
//! let mut rng = rand::rng();
//! let tier = table.sample(&mut rng); // &str
//! println!("you got: {tier}");
//! # }
//! ```
//!
//! ## Quick start (enum + macro)
//!
//! ```rust,ignore
//! use droptables::{WeightedEnum, DropTable};
//! use droptables_macros::WeightedEnum;
//! use rand::Rng;
//!
//! #[derive(Copy, Clone, Debug, WeightedEnum)]
//! enum Loot {
//!     #[probability(60.0)] Common,
//!     #[probability(30.0)] Uncommon,
//!     #[probability(9.0)]  Rare,
//!     #[probability(1.0)]  Legendary,
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let table: DropTable<Loot> = Loot::droptable()?;
//! let mut rng = rand::rng();
//! let item = table.sample(&mut rng);       // &Loot
//! let owned = table.sample_owned(&mut rng); // Loot (cloned)
//! # Ok(()) }
//! ```
//!
//! ## Performance
//! * **Build**: O(n) to construct an alias table from weights.
//! * **Sample**: O(1) per draw (2 random numbers, 1 branch).
//! * **Space**: 2 vectors of length `n` (f32 + usize).
//!
//! ## Gotchas
//! * Weights must be **non-negative** and not all zero; `NaN`/∞ are rejected.
//! * This is for *fixed* distributions. If you mutate weights often, rebuild the table.
//!
//! ## Testing & validation
//! The crate includes light tests that check input validation and that empirical
//! frequencies roughly match the specified distribution.
//!
//! ---
//!
//! `rand` integration uses the modern `Rng::random()` / `random_range()` APIs

mod error;
mod sampler;
mod staticdt;
mod uniform;
mod walker;

/// A minimal interface for “index samplers”.
/// Implemented by `WeightedSampler` (weighted) and `UniformSampler` (equal odds).
#[allow(clippy::len_without_is_empty)]
pub trait IndexSampler {
    fn len(&self) -> usize;
    fn sample_index<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> usize;
}

pub use error::ProbError;
pub use sampler::UniformSampler;
pub use staticdt::StaticDropTable;
pub use uniform::{UniformEnum, UniformTable};
pub use walker::WeightedSampler;

use rand::Rng;

/// A generic “drop table”: associates items with weights and samples them
/// using an internal [`WeightedSampler`].
///
/// Build it from any iterator of `(item, weight)` where `weight >= 0`.
#[derive(Debug, Clone)]
pub struct DropTable<T> {
    alias: WeightedSampler,
    items: Vec<T>,
}

pub use droptables_macros::UniformEnum;
/// Derive macro imported from `droptables_macros`.
/// See the crate-level example for usage.
pub use droptables_macros::WeightedEnum;

/// Trait implemented by the `WeightedEnum` derive macro.
///
/// Each variant and its weight is exposed via [`WeightedEnum::ENTRIES`],
/// which enables building a ready-to-sample [`DropTable`].
pub trait WeightedEnum: Sized + 'static {
    /// All `(variant, weight)` pairs for the enum.
    const ENTRIES: &'static [(Self, f32)];

    /// Convenience constructor that builds a [`DropTable`] from the enum entries.
    ///
    /// # Errors
    /// See [`WeightedSampler::new`] and [`ProbError`]: zero length, negative weight,
    /// non-finite or zero total weight will error.
    fn droptable() -> Result<DropTable<Self>, ProbError>
    where
        Self: Copy,
    {
        DropTable::from_pairs(Self::ENTRIES.iter().copied())
    }
}

impl<T> DropTable<T> {
    /// Build from any `(item, weight)` iterator.
    ///
    /// # Errors
    /// * [`ProbError::Empty`] if there are no items.
    /// * [`ProbError::Negative`] if any weight is negative.
    /// * [`ProbError::ZeroSum`] if the sum of weights is zero or not finite.
    ///
    /// # Complexity
    /// O(n) time / O(n) space.
    pub fn from_pairs<I>(pairs: I) -> Result<Self, ProbError>
    where
        I: IntoIterator<Item = (T, f32)>,
    {
        let mut items = Vec::new();
        let mut weights = Vec::new();
        for (t, w) in pairs {
            items.push(t);
            weights.push(w);
        }
        let alias = WeightedSampler::new(&weights)?;
        Ok(Self { alias, items })
    }

    /// Sample an item **by reference** (no `Clone` bound).
    ///
    /// # Panics
    /// Never panics for a well-constructed table.
    ///
    /// # Examples
    /// ```rust,ignore
    /// # use droptables::DropTable;
    /// # let table = DropTable::from_pairs([("a", 1.0), ("b", 3.0)]).unwrap();
    /// let mut rng = rand::rng();
    /// let s = table.sample(&mut rng); // &str
    /// ```
    pub fn sample<'a, R: Rng + ?Sized>(&'a self, rng: &mut R) -> &'a T {
        let idx = self.alias.sample_index(rng);
        &self.items[idx]
    }

    /// Sample an item **by value** (clones the chosen element).
    ///
    /// Prefer [`sample`](Self::sample) if you don’t need ownership.
    pub fn sample_owned<R: Rng + ?Sized>(&self, rng: &mut R) -> T
    where
        T: Clone,
    {
        self.items[self.alias.sample_index(rng)].clone()
    }

    /// Number of items in the table.
    pub fn len(&self) -> usize {
        self.alias.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.alias.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_pairs() {
        let dt = DropTable::from_pairs([("a", 1.0), ("b", 3.0)]).unwrap();
        let mut rng = rand::rng();
        let _ = dt.sample(&mut rng);
    }
}
