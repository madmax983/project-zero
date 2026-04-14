# 1035: Modular Fauna

## 1. Overview
Evolution doesn't follow a blueprint, and alien monsters should be weird. Animals are generated procedurally from modular parts (Head, Body, Limbs, Tail), with each part conferring specific stats or behaviors. A "Wolf-Head" adds a bite attack; "Crab-Legs" add armor plating. This emergent generation leads to unpredictable threats, like a cute "Bunny-Headed Scorpion" that suddenly stings the player's best soldier with lethal venom.

## 2. Dependencies
- Layer 1 `Fauna`/`Wildlife` spawning system.
- Layer 1 `Combat` stats (Armor, Damage, Attacks).
- Layer 1 `Pathfinding` (Movement speed).
- Layer 1 `Traits` or `Genetics` system.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::combat::{CombatStats, Armor, DamageType};
    use crate::layer1::fauna::{Fauna, AnimalParts, PartType};

    #[test]
    fn test_modular_parts_construct_combat_stats() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_modular_fauna_stats_system);

        let chimera = app.world_mut().spawn((
            Fauna,
            AnimalParts {
                head: PartType::WolfHead,
                body: PartType::BearTorso,
                limbs: PartType::CrabLegs,
                tail: PartType::ScorpionTail,
            },
            CombatStats::default(),
            Armor { rating: 0 },
        )).id();

        app.update();

        let stats = app.world().get::<CombatStats>(chimera).unwrap();
        let armor = app.world().get::<Armor>(chimera).unwrap();

        // Verify stats were aggregated from the specific parts
        assert!(stats.melee_damage > 10.0, "Wolf Head should add high melee bite damage.");
        assert!(stats.damage_types.contains(&DamageType::Venom), "Scorpion Tail should add Venom damage type.");
        assert!(armor.rating > 5, "Crab Legs should provide heavy armor rating.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/modular_fauna.rs
use bevy::prelude::*;
use crate::layer1::combat::{CombatStats, Armor, DamageType};
use crate::layer1::fauna::{Fauna, AnimalParts, PartType};

pub fn evaluate_modular_fauna_stats_system(
    mut query: Query<(&AnimalParts, &mut CombatStats, &mut Armor), With<Fauna>>,
) {
    for (parts, mut stats, mut armor) in query.iter_mut() {
        // Reset base
        stats.melee_damage = 5.0;
        stats.damage_types.clear();
        armor.rating = 0;

        // Evaluate Head
        if parts.head == PartType::WolfHead {
            stats.melee_damage += 15.0;
            stats.damage_types.push(DamageType::Piercing);
        } else if parts.head == PartType::BunnyHead {
            // Cute but weak
            stats.melee_damage += 1.0;
        }

        // Evaluate Limbs
        if parts.limbs == PartType::CrabLegs {
            armor.rating += 10; // Heavy plating
        } else if parts.limbs == PartType::GazelleLegs {
            // High speed (not tracked in this test/MVP but should be)
            armor.rating += 1;
        }

        // Evaluate Tail
        if parts.tail == PartType::ScorpionTail {
            stats.damage_types.push(DamageType::Venom);
            stats.melee_damage += 10.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Data-Driven Parts:** Hardcoding `if parts.head == PartType::WolfHead` is not scalable for procedural generation. Parts should be defined in a JSON/RON asset file containing their stat modifiers (Speed, HP, Armor, Damage, Traits), which the spawner reads and aggregates dynamically.
- **Rendering:** The renderer (Ratatui/Ratzilla) needs a way to construct the sprite or ASCII character based on these parts. For a terminal UI, this might mean combining ASCII characters or using a specific color palette depending on the dominant part.
- **Ecological Niches:** The procedural spawner shouldn't just create random chaotic jumbles. It should use a seeded logic based on the biome—desert planets are more likely to generate Chitin-plated, Venom-tailed creatures.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_modular_parts_construct_combat_stats` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- `AnimalParts` struct should be generic enough to allow missing parts (e.g., `tail: Option<PartType>`).
- Make sure `CombatStats` correctly handles multiple `DamageType` values when applying damage to a Pop's armor during combat.

## 8. Questions
*Builder: add questions here if spec is unclear.*
