use droptables::{UniformEnum, WeightedEnum};
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, WeightedEnum)]
enum UniqueRoll {
    // Approximate OSRS-ish odds (illustrative)
    #[odds = "1/128"]
    BandosArmor,       // (choose piece uniformly)
    #[odds = "1/508"]
    BandosHilt,
    #[odds = "1/256"]
    GodswordShard,     // (choose shard uniformly)
    #[rest]
    NotUnique,         // proceed to RDT access check
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, UniformEnum)]
enum BandosArmorItem {
    Chestplate,
    Tassets,
    Boots,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, UniformEnum)]
enum GodswordShardItem {
    Shard1,
    Shard2,
    Shard3,
}

// RDT access gate: only rolled if UniqueRoll::NotUnique.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, WeightedEnum)]
enum RdtAccess {
    #[odds = "8/127"]
    Hit,
    #[rest]
    Miss,
}

// Keep the RDT tiny for illustration.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, WeightedEnum)]
enum RareDropTableItem {
    #[odds = "1/100"]
    LoopHalfKey,
    #[odds = "1/106"]
    ToothHalfKey,
    #[odds = "1/406"]
    RuniteBar,
    #[odds = "1/677"]
    Rune2hSword,
    #[odds = "1/677"]
    RuneBattleaxe,
    #[rest]
    RuneStack,
}

// “Main” common table: the thing you get if you fail uniques and RDT gate.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, WeightedEnum)]
enum CommonMainItem {
    #[odds = "20/100"]
    Coins20500to21000,
    #[odds = "10/100"]
    SuperRestore4x3,
    #[odds = "8/100"]
    MagicLogs15to20,
    #[odds = "4/100"]
    SnapdragonSeed,
    #[odds = "3/100"]
    GrimySnapdragonx3,
    #[rest]
    Misc,
}

// The guaranteed “extra” table (a second, separate roll every kill).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, WeightedEnum)]
enum ExtraItem {
    #[odds = "25/100"]
    BonesNoted6to8,
    #[odds = "20/100"]
    LawRunes20to30,
    #[odds = "10/100"]
    NatureRunes25to35,
    #[odds = "5/100"]
    Coins4000to6000,
    #[rest]
    SuppliesMisc,
}

// Tertiaries (independent of everything else).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, WeightedEnum)]
enum TertiaryPet {
    #[odds = "1/5000"]
    PetGeneralGraardor,
    #[rest]
    Nothing,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, WeightedEnum)]
enum TertiaryClue {
    #[odds = "1/100"] // hard clue per your text
    HardClue,
    #[rest]
    Nothing,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, WeightedEnum)]
enum TertiaryLongBone {
    #[odds = "1/400"]
    LongBone,
    #[rest]
    Nothing,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, WeightedEnum)]
enum TertiaryCurvedBone {
    #[odds = "1/5000"]
    CurvedBone,
    #[rest]
    Nothing,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build tables
    let unique = UniqueRoll::droptable()?;
    let bandos_armor = BandosArmorItem::droptable()?; // uniform
    let shard_piece = GodswordShardItem::droptable()?; // uniform
    let rdt_gate = RdtAccess::droptable()?; // weighted yes/no
    let rdt = RareDropTableItem::droptable()?; // weighted
    let common_main = CommonMainItem::droptable()?; // weighted
    let extra = ExtraItem::droptable()?; // weighted

    // Tertiaries
    let pet = TertiaryPet::droptable()?;
    let clue = TertiaryClue::droptable()?;
    let lbone = TertiaryLongBone::droptable()?;
    let cbone = TertiaryCurvedBone::droptable()?;

    // Tallies
    let mut hist: HashMap<String, u64> = HashMap::new();

    let mut rng = rand::rng();

    for i in 0..200_000 {
        let show_drop = i < 3;
        if show_drop {
            println!("Kill {} — drops:", i + 1);
        }

        // ===== Primary flow: Unique ➜ (if miss) RDT gate ➜ (if miss) Common =====
        match unique.sample_owned(&mut rng) {
            UniqueRoll::BandosArmor => {
                let piece = bandos_armor.sample_owned(&mut rng);
                *hist.entry(format!("{piece:?}")).or_default() += 1;
                if show_drop { println!("  Unique: Bandos {piece:?}"); }
            }
            UniqueRoll::BandosHilt => {
                *hist.entry("BandosHilt".into()).or_default() += 1;
                if show_drop { println!("  Unique: BandosHilt"); }
            }
            UniqueRoll::GodswordShard => {
                let which = shard_piece.sample_owned(&mut rng);
                *hist.entry(format!("{which:?}")).or_default() += 1;
                if show_drop { println!("  Unique: {which:?}"); }
            }
            UniqueRoll::NotUnique => {
                match rdt_gate.sample_owned(&mut rng) {
                    RdtAccess::Hit => {
                        let r = rdt.sample_owned(&mut rng);
                        *hist.entry(format!("{r:?}")).or_default() += 1;
                        if show_drop { println!("  RDT: {r:?}"); }
                    }
                    RdtAccess::Miss => {
                        let c = common_main.sample_owned(&mut rng);
                        *hist.entry(format!("{c:?}")).or_default() += 1;
                        if show_drop { println!("  Common: {c:?}"); }
                    }
                }
            }
        }

        // ===== Guaranteed “extra” roll =====
        let e = extra.sample_owned(&mut rng);
        *hist.entry(format!("{e:?}")).or_default() += 1;
        if show_drop { println!("  Extra: {e:?}"); }

        // ===== Independent tertiaries =====
        if let TertiaryPet::PetGeneralGraardor = pet.sample_owned(&mut rng) {
            *hist.entry("PetGeneralGraardor".into()).or_default() += 1;
            if show_drop { println!("  Tertiary: PetGeneralGraardor"); }
        }
        if let TertiaryClue::HardClue = clue.sample_owned(&mut rng) {
            *hist.entry("HardClue".into()).or_default() += 1;
            if show_drop { println!("  Tertiary: HardClue"); }
        }
        if let TertiaryLongBone::LongBone = lbone.sample_owned(&mut rng) {
            *hist.entry("LongBone".into()).or_default() += 1;
            if show_drop { println!("  Tertiary: LongBone"); }
        }
        if let TertiaryCurvedBone::CurvedBone = cbone.sample_owned(&mut rng) {
            *hist.entry("CurvedBone".into()).or_default() += 1;
            if show_drop { println!("  Tertiary: CurvedBone"); }
        }

        if show_drop {
            println!();
        }
    }

    // Pretty print totals (highest first)
    let mut items: Vec<(String, u64)> = hist.into_iter().collect();
    items.sort_by(|a, b| b.1.cmp(&a.1));

    println!("General Graardor (Bandos) — simulated drops:");
    for (item, count) in items {
        println!("{count:>7}  {item}");
    }

    Ok(())
}
