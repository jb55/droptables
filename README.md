# 🎲 droptables

> *“Because RNGesus deserves O(1) performance.”*

`droptables` is a Rust library for building and sampling from weighted drop tables at lightning speed.
It’s perfect for games, loot systems, procedural generation, or anywhere you need **weighted random picks**—without sacrificing performance or your sanity.

---

## ✨ Features

* **⚡ O(1) Sampling** – Uses [Walker’s Alias Method](https://en.wikipedia.org/wiki/Alias_method) for constant-time draws.
* **📦 Enum Power-Up** – Derive probabilities directly from enum variants with `#[probability(...)]`.
* **🔮 Flexible Sources** – Build from enums **or** from arbitrary `(item, weight)` pairs.
* **🛡️ Error-checked** – Prevents negative weights, zero-sum disasters, and other statistical crimes.
* **🥷 No Cloning Required** – Sample by reference or by value.

---

## 🚀 Quick Start

Add to your `Cargo.toml`:


```toml
[dependencies]
droptables = "0.1"
```

---

### 🎯 Example: Loot Rarity Table

```rust
use droptables::{DropTable, WeightedEnum};
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, WeightedEnum)]
enum Rarity {
    #[probability(0.001)]
    Mythic,
    #[probability(1/100)]
    Legendary,
    #[probability(20/100)]
    Uncommon,
    #[probability(50/100)]
    Common,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let table = Rarity::droptable()?;
    let mut hist: HashMap<Rarity, u64> = HashMap::new();
    let mut rng = rand::rng();

    for _ in 0..2000 {
        *hist.entry(*table.sample(&mut rng)).or_default() += 1;
    }

    for (rarity, count) in hist {
        println!("{count:>5} {:?}", rarity);
    }

    Ok(())
}
```

---

## 🛠 How It Works

Under the hood:

1. **`WeightedEnum` macro** scans your enum variants for `#[probability(...)]` attributes.
2. Probabilities are compiled into a static `ENTRIES` array.
3. `DropTable` builds an alias table via `WalkerAlias` for O(1) sampling.
4. You call `.sample()` and get your item **fast**.

---

## 🧪 Testing Your Luck

```bash
cargo run --example rarity
```

Sample output:

```
  997 Common
  402 Uncommon
   58 Legendary
    1 Mythic
```

(Your mileage may vary, depending on the whims of RNGesus.)

---

## 🧩 When to Use This

* 🎮 Game loot systems
* 🗺 Procedural map generation
* 🎲 Random event systems
* 🦄 Gacha mechanics (don’t be evil)

---

## 📜 License

MIT — *because sharing is caring.*
