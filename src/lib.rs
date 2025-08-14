use rand::Rng;

mod error;
mod walker;

pub use error::ProbError;
pub use walker::AliasTable;

/// --- Generic drop table (labels + alias) ---
#[derive(Debug, Clone)]
pub struct DropTable<T> {
    alias: AliasTable,
    items: Vec<T>,
}

pub use droptables_macros::WeightedEnum;

/// Trait the macro implements for enums with #[probability(...)] on each variant.
pub trait WeightedEnum: Sized + 'static {
    /// `(variant, weight)` for every variant.
    const ENTRIES: &'static [(Self, f64)];
    /// Convenience: build a `DropTable` from the enum.
    fn droptable() -> Result<DropTable<Self>, ProbError>
    where
        Self: Copy,
    {
        DropTable::from_pairs(Self::ENTRIES.iter().copied())
    }
}

impl<T> DropTable<T> {
    /// Build from any `(item, weight)` iterator. Zero/negative weights are rejected.
    pub fn from_pairs<I>(pairs: I) -> Result<Self, ProbError>
    where
        I: IntoIterator<Item = (T, f64)>,
    {
        let mut items = Vec::new();
        let mut weights = Vec::new();
        for (t, w) in pairs {
            items.push(t);
            weights.push(w);
        }
        let alias = AliasTable::new(&weights)?;
        Ok(Self { alias, items })
    }

    /// Sample by reference (no Clone bound).
    pub fn sample<'a, R: Rng + ?Sized>(&'a self, rng: &mut R) -> &'a T {
        let idx = self.alias.sample_index(rng);
        &self.items[idx]
    }

    /// Sample by value (requires `T: Clone`).
    pub fn sample_owned<R: Rng + ?Sized>(&self, rng: &mut R) -> T
    where
        T: Clone,
    {
        self.items[self.alias.sample_index(rng)].clone()
    }

    pub fn len(&self) -> usize {
        self.alias.len()
    }
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
