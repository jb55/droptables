use droptables::{DropTable, StaticDropTable, WeightedEnum, WeightedSampler};
use rand::Rng;
use std::collections::HashMap;
use std::error::Error;

struct TierByRarity {
    common: DropTable<StatTier>,
    uncommon: DropTable<StatTier>,
    rare: DropTable<StatTier>,
    legendary: DropTable<StatTier>,
    mythic: DropTable<StatTier>,
}

impl TierByRarity {
    fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            common: DropTable::from_pairs([
                (StatTier::T1, 80.0 / 100.0),
                (StatTier::T2, 18.0 / 100.0),
                (StatTier::T3, 2.0 / 100.0),
            ])?,
            uncommon: DropTable::from_pairs([
                (StatTier::T1, 60.0 / 100.0),
                (StatTier::T2, 32.0 / 100.0),
                (StatTier::T3, 8.0 / 100.0),
            ])?,
            rare: DropTable::from_pairs([
                (StatTier::T1, 30.0 / 100.0),
                (StatTier::T2, 40.0 / 100.0),
                (StatTier::T3, 24.0 / 100.0),
                (StatTier::T4, 6.0 / 100.0),
            ])?,
            legendary: DropTable::from_pairs([
                (StatTier::T2, 20.0 / 100.0),
                (StatTier::T3, 35.0 / 100.0),
                (StatTier::T4, 30.0 / 100.0),
                (StatTier::T5, 15.0 / 100.0),
            ])?,
            mythic: DropTable::from_pairs([
                (StatTier::T3, 15.0 / 100.0),
                (StatTier::T4, 45.0 / 100.0),
                (StatTier::T5, 25.0 / 100.0),
                (StatTier::T6, 15.0 / 100.0),
            ])?,
        })
    }
    fn sample<R: rand::Rng>(&self, rng: &mut R, r: Rarity) -> StatTier {
        match r {
            Rarity::Common => self.common.sample_owned(rng),
            Rarity::Uncommon => self.uncommon.sample_owned(rng),
            Rarity::Rare => self.rare.sample_owned(rng),
            Rarity::Legendary => self.legendary.sample_owned(rng),
            Rarity::Mythic => self.mythic.sample_owned(rng),
        }
    }
}

fn rarity_slot_bonus(r: Rarity) -> u8 {
    match r {
        Rarity::Legendary => 1,
        Rarity::Mythic => 2,
        _ => 0,
    }
}

fn maybe_promote_gem_slot_quality<R: Rng>(
    rng: &mut R,
    base: GemSlotQuality,
    item_rarity: Rarity,
) -> GemSlotQuality {
    let (p_one, p_two) = match item_rarity {
        Rarity::Common => (0.00, 0.00),
        Rarity::Uncommon => (0.05, 0.00),
        Rarity::Rare => (0.12, 0.00),
        Rarity::Legendary => (0.30, 0.05),
        Rarity::Mythic => (0.45, 0.15),
    };
    let mut hit = |p: f32| rng.random::<f32>() < p;

    let step_up = |r: GemSlotQuality| match r {
        GemSlotQuality::Rusted => GemSlotQuality::Standard,
        GemSlotQuality::Standard => GemSlotQuality::Charged,
        GemSlotQuality::Charged => GemSlotQuality::SuperCharged,
        GemSlotQuality::SuperCharged => GemSlotQuality::SoulAttuned,
        GemSlotQuality::SoulAttuned => GemSlotQuality::SoulAttuned,
    };

    let mut out = base;
    if hit(p_one) {
        out = step_up(out);
        if hit(p_two) {
            out = step_up(out);
        }
    }

    // legendary/mythics don't have rusted gem slots
    if (item_rarity == Rarity::Legendary || item_rarity == Rarity::Mythic)
        && out == GemSlotQuality::Rusted
    {
        out = GemSlotQuality::Standard
    }

    out
}

// -------------------- enums --------------------

#[derive(Copy, Eq, PartialEq, Clone, Debug, Hash, Default, WeightedEnum)]
enum Rarity {
    #[rest]
    #[default]
    Common,
    #[odds = "1/10"]
    Uncommon,
    #[odds = "1/100"]
    Rare,
    #[odds = "1/100000"]
    Legendary,
    #[odds = "1/1000000"]
    Mythic,
}

#[derive(Copy, Eq, PartialEq, Clone, Debug, Hash, Default, WeightedEnum)]
enum GemSlotQuality {
    #[default]
    #[rest]
    Rusted,
    #[odds = "50/100"]
    Standard,
    #[odds = "10/100"]
    Charged,
    #[odds = "5/100"]
    SuperCharged,
    #[odds = "1/1000"]
    SoulAttuned,
}

#[derive(Copy, Eq, PartialEq, Clone, Debug, Hash, WeightedEnum)]
enum GemSlots {
    #[rest]
    Zero,
    #[odds = "1200/4096"]
    One,
    #[odds = "600/4096"]
    Two,
    #[odds = "200/4096"]
    Three,
    #[odds = "40/4096"]
    Four,
    #[odds = "10/4096"]
    Five,
    // 1–6 gems with aggressive rarity drop-off.
    #[odds = "3/4096"]
    Six,
}

impl GemSlots {
    fn as_index(self) -> u8 {
        self as u8
    }
}

#[derive(Copy, Eq, PartialEq, Clone, Debug, Hash, WeightedEnum)]
enum StatSlots {
    #[rest]
    One = 1,
    #[odds = "1200/4096"]
    Two,
    #[odds = "600/4096"]
    Three,
    #[odds = "200/4096"]
    Four,
    #[odds = "40/4096"]
    Five,
    #[odds = "10/4096"]
    Six,
}

impl StatSlots {
    #[inline]
    fn as_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Copy, Eq, PartialEq, Clone, Default, Debug, Hash, WeightedEnum)]
enum StatType {
    // Make some offensive spikes rarer.
    #[odds = "1/40"]
    CritChance,
    #[odds = "1/60"]
    CritDamage,
    #[odds = "1/50"]
    AttackSpeed,
    #[odds = "1/50"]
    ElementalDamage,
    #[odds = "2/50"]
    ArmorPen,
    // Core/defensive more common.
    #[odds = "5/50"]
    Strength,
    #[odds = "5/50"]
    Dexterity,
    #[odds = "5/50"]
    Intelligence,
    #[odds = "5/50"]
    Vitality,
    #[odds = "7/50"]
    LifeOnHit,
    #[odds = "8/50"]
    Armor,
    #[rest]
    #[default]
    AllRes,
}

#[derive(Copy, Eq, Default, PartialEq, Clone, Debug, Hash, WeightedEnum)]
enum StatTier {
    #[odds = "1/5000"]
    T6,
    #[odds = "1/1000"]
    T5,
    #[odds = "1/200"]
    T4,
    #[odds = "1/40"]
    T3,
    #[odds = "1/8"]
    T2,
    #[rest]
    #[default]
    T1,
}

// -------------------- data structs --------------------

#[derive(Clone, Debug, Default)]
struct Gem {
    quality: GemSlotQuality,
}

#[derive(Clone, Debug, Default)]
struct StatRoll {
    kind: StatType,
    tier: StatTier,
    value: i32,
}

#[derive(Clone, Debug, Default)]
struct Item {
    rarity: Rarity,
    gem_slots: u8,
    stat_slots: u8,
    gem_storage: [Gem; 7],       // 0-6,
    stat_storage: [StatRoll; 6], // 1-6,
}

impl Item {
    fn gems(&self) -> &[Gem] {
        &self.gem_storage[..self.gem_slots as usize]
    }

    fn gems_mut(&mut self) -> &mut [Gem] {
        &mut self.gem_storage
    }

    fn stats(&self) -> &[StatRoll] {
        &self.stat_storage[..self.stat_slots as usize]
    }
}

// All droptables built once, passed by reference.
struct Tables {
    rarity: StaticDropTable<WeightedSampler, Rarity>,
    gem_slots: StaticDropTable<WeightedSampler, GemSlots>,
    gem_slot_quality: StaticDropTable<WeightedSampler, GemSlotQuality>,
    stat_slots: StaticDropTable<WeightedSampler, StatSlots>,
    stat_type: StaticDropTable<WeightedSampler, StatType>,
    tier_by_rarity: TierByRarity,
}
impl Tables {
    fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            rarity: Rarity::droptable()?,
            gem_slots: GemSlots::droptable()?,
            gem_slot_quality: GemSlotQuality::droptable()?,
            stat_slots: StatSlots::droptable()?,
            stat_type: StatType::droptable()?,
            tier_by_rarity: TierByRarity::new()?,
        })
    }
}

// -------------------- helpers --------------------

fn sample_unique_stat_types<R: Rng>(
    rng: &mut R,
    n: u8,
    tables: &Tables,
    item: &mut Item,
    item_rarity: Rarity,
) -> u8 {
    let mut seen: u64 = 0;
    let mut attempts = 0;
    let mut count = 0_u8;

    while count < n && attempts < n * 50 {
        attempts += 1;
        let stat_type = tables.stat_type.sample_owned(rng);
        let stat_bit = 1 << (stat_type as u8);
        if (seen & stat_bit) != 0 {
            continue;
        }
        seen = seen | stat_bit;
        item.stat_storage[count as usize] = roll_stat(rng, stat_type, item_rarity, tables);
        count += 1;
    }

    item.stat_slots = count;

    count
}

fn stat_value_range(kind: StatType, tier: StatTier) -> (i32, i32) {
    let (base_min, base_max) = match kind {
        StatType::Strength | StatType::Dexterity | StatType::Intelligence => (3, 12),
        StatType::Vitality => (4, 14),
        StatType::Armor => (10, 40),
        StatType::AllRes => (3, 10),
        StatType::LifeOnHit => (5, 25),
        StatType::ArmorPen => (2, 8),
        StatType::ElementalDamage => (2, 6),
        StatType::AttackSpeed => (1, 4),
        StatType::CritChance => (1, 3),
        StatType::CritDamage => (5, 15),
    };

    let (mul_min, mul_max) = match tier {
        StatTier::T1 => (1.0, 1.2),
        StatTier::T2 => (1.2, 1.6),
        StatTier::T3 => (1.6, 2.3),
        StatTier::T4 => (2.3, 3.5),
        StatTier::T5 => (3.5, 5.5),
        StatTier::T6 => (5.5, 9.0),
    };

    let min = ((base_min as f32) * mul_min).round() as i32;
    let max = ((base_max as f32) * mul_max).round() as i32;
    (min.max(1), max.max(min + 1))
}

// CHANGE: accept item_rarity & tables, pick tier from TierTableByRarity
fn roll_stat<R: Rng>(
    rng: &mut R,
    kind: StatType,
    item_rarity: Rarity,
    tables: &Tables,
) -> StatRoll {
    let tier = tables.tier_by_rarity.sample(rng, item_rarity); // <— conditioned
    let (min, max) = stat_value_range(kind, tier);

    let u: f32 = rng.random::<f32>();
    let gamma = 0.6;
    let scaled = (u.powf(gamma) * (max - min) as f32).round() as i32;
    let value = (min + scaled).clamp(min, max);

    StatRoll { kind, tier, value }
}

fn roll_item<R: Rng>(rng: &mut R, t: &Tables, item: &mut Item) {
    let item_rarity = t.rarity.sample_owned(rng);
    let base_gem_slots = t.gem_slots.sample_owned(rng).as_index();
    let base_stat_slots = t.stat_slots.sample_owned(rng).as_u8();
    // Stat slots and unique stat kinds
    let bonus = rarity_slot_bonus(item_rarity);
    let gem_slots = (base_gem_slots.saturating_add(bonus)).min(6);
    let stat_slots = (base_stat_slots.saturating_add(bonus)).min(6).max(1);

    item.rarity = item_rarity;
    item.gem_slots = gem_slots;
    item.stat_slots = stat_slots;

    // NEW: gem rarity promotion per-gem based on item rarity
    for g in item.gems_mut().iter_mut().take(gem_slots as usize) {
        let gem_slot_quality = t.gem_slot_quality.sample_owned(rng);
        *g = Gem {
            quality: maybe_promote_gem_slot_quality(rng, gem_slot_quality, item_rarity),
        };
    }

    // Stat slots and unique stat kinds
    sample_unique_stat_types(rng, stat_slots, &t, item, item_rarity);
}

// -------------------- demo --------------------

fn print_item(item: &Item) {
    println!("== {:?} item ==", item.rarity);
    if !item.gems().is_empty() {
        print!("Gem Slots: ");
        for (i, g) in item.gems().iter().enumerate() {
            if i != 0 {
                print!(", ");
            }
            print!("{:?}", g.quality)
        }
        println!();
    }
    println!("Stats: ");
    for s in item.stats() {
        println!("  {:?} {:?} = {}", s.kind, s.tier, s.value);
    }
    println!();
}

fn main() -> Result<(), Box<dyn Error>> {
    let tables = Tables::new()?; // build once
    let mut rng = rand::rng();
    let mut item = Item::default();

    // generate a few showcase items
    for _ in 0..5 {
        roll_item(&mut rng, &tables, &mut item);
        print_item(&item);
    }

    // quick rarity sanity check to ensure odds feel right
    let mut hist: HashMap<Rarity, u64> = HashMap::new();

    let amt = 300_000;
    for i in 0..amt {
        roll_item(&mut rng, &tables, &mut item);
        if item.rarity == Rarity::Mythic || item.rarity == Rarity::Legendary {
            println!("{:?} drop {i}/{amt}", item.rarity);
            print_item(&item);
        }

        *hist.entry(item.rarity).or_insert(0) += 1;
    }
    println!("Rarity histogram (300k rolls):");
    let mut v: Vec<_> = hist.into_iter().collect();
    v.sort_by_key(|&(_, c)| c);
    v.reverse();
    for (r, c) in v {
        println!("{:>8} {:?}", c, r);
    }

    Ok(())
}
