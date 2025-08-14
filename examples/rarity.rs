use droptables::{DropTable, WeightedEnum};
use std::collections::HashMap;

#[derive(Copy, Eq, PartialEq, Clone, Debug, Hash, WeightedEnum)]
enum Rarity {
    #[odds = "1/1000"]
    Mythic,
    #[odds = "1/100"]
    Legendary,
    #[odds = "20/100"]
    Uncommon,
    #[rest]
    Common,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build straight from the enum:
    let table = Rarity::droptable()?; // uses the macro-provided ENTRIES
    let mut hist: HashMap<Rarity, u64> = HashMap::default();

    // Or, if you want to mix arbitrary items with weights:
    let _custom: DropTable<&'static str> =
        DropTable::from_pairs([("sword", 1.0), ("shield", 3.0)])?;

    // Sample:
    let mut rng = rand::rng();
    for _ in 0..2000000 {
        hist.entry(*table.sample(&mut rng))
            .and_modify(|acc| {
                *acc += 1;
            })
            .or_insert(1);
    }

    let mut values: Vec<(Rarity, u64)> = hist.into_iter().collect();
    values.sort_by(|(_, ca), (_, cb)| cb.cmp(ca));

    for (rarity, count) in values {
        println!("{count: >5} {rarity:?}");
    }

    Ok(())
}
