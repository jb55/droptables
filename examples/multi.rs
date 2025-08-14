use droptables::{WeightedEnum, UniformEnum};
use std::collections::HashMap;

#[derive(Copy, Eq, PartialEq, Clone, Debug, Hash, WeightedEnum)]
enum Rarity {
    #[odds = "1/1000"] Mythic,
    #[odds = "1/100"]  Legendary,
    #[odds = "20/100"] Uncommon,
    #[rest]            Common,
}

#[derive(Copy, Eq, PartialEq, Clone, Debug, Hash, UniformEnum)]
enum LegendaryLoot {
    Thunderfury,
    Sulfuras,
    Atiesh,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rarity = Rarity::droptable()?;                 // StaticDropTable<WeightedSampler, Rarity>
    let legendaries = LegendaryLoot::droptable()?;     // StaticDropTable<UniformSampler, LegendaryLoot>

    let mut rng = rand::rng();
    let mut rarity_hist: HashMap<Rarity, u64> = HashMap::default();
    let mut leg_hist: HashMap<LegendaryLoot, u64> = HashMap::default();

    for _ in 0..200_000 {
        let r = rarity.sample_owned(&mut rng);
        *rarity_hist.entry(r).or_default() += 1;

        if matches!(r, Rarity::Legendary | Rarity::Mythic) {
            *leg_hist.entry(legendaries.sample_owned(&mut rng)).or_default() += 1;
        }
    }

    println!("Rarity:");
    let mut v: Vec<_> = rarity_hist.into_iter().collect();
    v.sort_by(|a,b| b.1.cmp(&a.1));
    for (k,c) in v { println!("{c:>6} {k:?}"); }

    println!("\nLegendary Loot (only when legendary):");
    let mut v: Vec<_> = leg_hist.into_iter().collect();
    v.sort_by(|a,b| b.1.cmp(&a.1));
    for (k,c) in v { println!("{c:>6} {k:?}"); }

    Ok(())
}
