use droptables::{UniformEnum, WeightedEnum};
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, WeightedEnum)]
enum UniqueRoll {
    // Exact OSRS behaviour:
    //  - 1/128 to hit the Bandos armour table; choose piece uniformly (=> each is 1/384 overall)
    //  - 1/508 Bandos hilt
    //  - 1/256 any godsword shard; choose shard uniformly (=> 1/768 each)
    //  - else NotUnique
    #[odds = "1/128"]
    BandosArmor,
    #[odds = "1/508"]
    BandosHilt,
    #[odds = "1/256"]
    GodswordShard,
    #[rest]
    NotUnique, // proceed to RDT access check
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, UniformEnum)]
enum BandosArmorItem {
    Chestplate, // each ends up 1/384 overall
    Tassets,
    Boots,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, UniformEnum)]
enum GodswordShardItem {
    Shard1, // each 1/768 overall
    Shard2,
    Shard3,
}

// RDT access gate on NotUnique: 8/127 chance to roll RDT, else roll main table.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, WeightedEnum)]
enum RdtAccess {
    #[odds = "8/127"]
    Hit,
    #[rest]
    Miss,
}

// Tiny illustrative RDT (not comprehensive).
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

// “Main” table when you miss uniques and fail the RDT gate.
// The wiki shows categories, quantities, and that RDT access is 8/127; exact per-item weights aren’t public,
// so we pick plausible relative weights that feel right in simulation.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, WeightedEnum)]
enum CommonMainItem {
    // coin piles are very common; bias them highest
    #[odds = "240/1000"]
    Coins19500to21000,

    // staple supplies
    #[odds = "96/1000"]
    SuperRestore4x3,
    #[odds = "72/1000"]
    MagicLogs15to20, // noted
    #[odds = "60/1000"]
    NatureRunes60to70,

    // herb-y stuff
    #[odds = "36/1000"]
    SnapdragonSeed,
    #[odds = "30/1000"]
    GrimySnapdragonx3, // noted

    // ores (noted)
    #[odds = "36/1000"]
    AdamantiteOre15to20Noted,
    #[odds = "48/1000"]
    Coal115to120Noted,

    #[odds = "120/1000"]
    RunePlatelegsOrSkirt, // alchable filler
    #[odds = "90/1000"]
    RuneKiteshield, // alchable filler
    #[odds = "72/1000"]
    LawRunes20to30, // common runes
    #[odds = "50/1000"]
    DeathRunes30to40, // common runes
    #[odds = "50/1000"]
    HerbMixLowTier, // assorted herbs/seeds

    // anything else we’re not modelling explicitly
    #[rest]
    Misc,
}

// Tertiaries (independent of everything else).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, WeightedEnum)]
enum TertiaryPet {
    #[odds = "1/5000"]
    PetGeneralGraardor,
    #[rest]
    Nothing,
}

// Graardor gives Elite clues at 1/250.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, WeightedEnum)]
enum TertiaryClue {
    #[odds = "1/250"]
    EliteClue,
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

        // ===== Always drop =====
        *hist.entry("BigBones".into()).or_default() += 1;
        if show_drop {
            println!("  Always: BigBones");
        }

        // ===== Primary flow: Unique ➜ (if miss) RDT gate ➜ (if miss) Common =====
        match unique.sample_owned(&mut rng) {
            UniqueRoll::BandosArmor => {
                let piece = bandos_armor.sample_owned(&mut rng);
                *hist.entry(format!("{piece:?}")).or_default() += 1;
                if show_drop {
                    println!("  Unique: Bandos {piece:?}");
                }
            }
            UniqueRoll::BandosHilt => {
                *hist.entry("BandosHilt".into()).or_default() += 1;
                if show_drop {
                    println!("  Unique: BandosHilt");
                }
            }
            UniqueRoll::GodswordShard => {
                let which = shard_piece.sample_owned(&mut rng);
                *hist.entry(format!("{which:?}")).or_default() += 1;
                if show_drop {
                    println!("  Unique: {which:?}");
                }
            }
            UniqueRoll::NotUnique => match rdt_gate.sample_owned(&mut rng) {
                RdtAccess::Hit => {
                    let r = rdt.sample_owned(&mut rng);
                    *hist.entry(format!("{r:?}")).or_default() += 1;
                    if show_drop {
                        println!("  RDT: {r:?}");
                    }
                }
                RdtAccess::Miss => {
                    let c = common_main.sample_owned(&mut rng);
                    *hist.entry(format!("{c:?}")).or_default() += 1;
                    if show_drop {
                        println!("  Common: {c:?}");
                    }
                }
            },
        }

        // ===== Independent tertiaries =====
        if let TertiaryPet::PetGeneralGraardor = pet.sample_owned(&mut rng) {
            *hist.entry("PetGeneralGraardor".into()).or_default() += 1;
            if show_drop {
                println!("  Tertiary: PetGeneralGraardor");
            }
        }
        if let TertiaryClue::EliteClue = clue.sample_owned(&mut rng) {
            *hist.entry("EliteClue".into()).or_default() += 1;
            if show_drop {
                println!("  Tertiary: EliteClue");
            }
        }
        if let TertiaryLongBone::LongBone = lbone.sample_owned(&mut rng) {
            *hist.entry("LongBone".into()).or_default() += 1;
            if show_drop {
                println!("  Tertiary: LongBone");
            }
        }
        if let TertiaryCurvedBone::CurvedBone = cbone.sample_owned(&mut rng) {
            *hist.entry("CurvedBone".into()).or_default() += 1;
            if show_drop {
                println!("  Tertiary: CurvedBone");
            }
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
