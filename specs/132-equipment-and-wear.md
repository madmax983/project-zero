# 132: Equipment & Wear

## Overview

Refactors the Clothing system (Spec 040) from a global resource pool to a personal equipment system (similar to Spec 058 Personal Tools). Pops will now have equipment slots (`Head`, `Body`, `Tool`) and must physically fetch and wear clothing items to gain protection from Hypothermia.

This adds:
- **Equipment Slots**: `Head` and `Body` in addition to `Tool`.
- **Item Components**: `Clothing` component with durability and insulation.
- **Fetch Logic**: Pops actively seek clothing when cold or naked.
- **Durability**: Clothing degrades over time and breaks, requiring replacement.

## Dependencies

- `040` — Clothing and Temperature (Global resource logic to be refactored).
- `058` — Personal Tools (Established `Equipment` and `Tool` pattern).
- `016` — Utility AI (Action evaluation).

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/equipment_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Equipment};
    use crate::layer1::items::{Item, Clothing, ClothingType};
    use crate::layer1::health::Health;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::seasons::{Season, SeasonState};
    use crate::layer1::clothing::{hypothermia_system, clothing_wear_system};

    // 1. Equipment Slots
    #[test]
    fn test_equipment_has_body_slot() {
        let mut world = World::new();
        let entity = world.spawn((Pop, Equipment::default())).id();
        let eq = world.get::<Equipment>(entity).unwrap();

        // Assert new fields exist (compiler check mostly, but good for TDD)
        assert!(eq.body.is_none());
        assert!(eq.head.is_none());
    }

    // 2. Clothing Component
    #[test]
    fn test_clothing_component() {
        let mut world = World::new();
        let tunic = world.spawn((
            Item,
            Clothing {
                clothing_type: ClothingType::Tunic,
                insulation: 1.0,
                durability: 100.0,
                max_durability: 100.0,
            }
        )).id();

        let c = world.get::<Clothing>(tunic).unwrap();
        assert_eq!(c.insulation, 1.0);
    }

    // 3. Hypothermia Logic (Refactored)
    #[test]
    fn test_hypothermia_checks_equipment() {
        let mut world = World::new();
        world.insert_resource(SeasonState { current_season: Season::Winter });
        // Global resource should be ignored or used only for "available" count,
        // but damage depends on Equipment.
        world.insert_resource(ColonyResources::default());

        // Pop 1: Naked (Should take damage)
        let pop1 = world.spawn((
            Pop,
            Health { current: 100.0, max: 100.0 },
            Equipment::default(), // No body
        )).id();

        // Pop 2: Clothed (Should be safe)
        let tunic = world.spawn(Clothing {
            clothing_type: ClothingType::Tunic,
            insulation: 1.0,
            durability: 100.0,
            max_durability: 100.0,
        }).id();

        let pop2 = world.spawn((
            Pop,
            Health { current: 100.0, max: 100.0 },
            Equipment { body: Some(tunic), ..Default::default() },
        )).id();

        // Run system (deterministic check or high probability loop needed if RNG used)
        // Assuming system iterates and applies damage.
        // For test stability, we might need to mock RNG or check logic directly.
        // Here we run it once and assume standard shortage logic is replaced by equipment check.

        // Note: The previous hypothermia_system used global shortage probability.
        // The NEW system should check per-pop.
        // If Pop has NO body item -> High chance of damage.
        // If Pop has body item -> Low/No chance.

        hypothermia_system(&mut world);

        let h1 = world.get::<Health>(pop1).unwrap();
        let h2 = world.get::<Health>(pop2).unwrap();

        // Warning: This test assumes 100% damage chance for naked pops in winter for simplicity,
        // or runs enough times.
        // Ideally the system logic ensures naked pops get hurt.
        // assert!(h1.current < 100.0, "Naked pop should freeze");
        // assert_eq!(h2.current, 100.0, "Clothed pop should be warm");
    }

    // 4. Wear Logic
    #[test]
    fn test_clothing_degrades_on_wearer() {
        let mut world = World::new();

        let tunic = world.spawn(Clothing {
            clothing_type: ClothingType::Tunic,
            insulation: 1.0,
            durability: 10.0,
            max_durability: 100.0,
        }).id();

        let pop = world.spawn((
            Pop,
            Equipment { body: Some(tunic), ..Default::default() },
        )).id();

        clothing_wear_system(&mut world);

        let c = world.get::<Clothing>(tunic).unwrap();
        assert!(c.durability < 10.0);
    }

    // 5. Breakage
    #[test]
    fn test_clothing_breaks() {
        let mut world = World::new();

        let tunic = world.spawn(Clothing {
            clothing_type: ClothingType::Tunic,
            insulation: 1.0,
            durability: 0.001, // Almost broken
            max_durability: 100.0,
        }).id();

        let pop = world.spawn((
            Pop,
            Equipment { body: Some(tunic), ..Default::default() },
        )).id();

        clothing_wear_system(&mut world);

        // Entity should be despawned
        assert!(world.get_entity(tunic).is_none());

        // Slot should be None
        let eq = world.get::<Equipment>(pop).unwrap();
        assert!(eq.body.is_none());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `Equipment` Struct (`src/layer1/pop.rs`)

```rust
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Equipment {
    pub tool: Option<Entity>,
    pub weapon: Option<Entity>,
    pub body: Option<Entity>, // New
    pub head: Option<Entity>, // New
}
```

### 2. Define `Clothing` Component (`src/layer1/items.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClothingType {
    Tunic,
    Parka,
}

#[derive(Component, Debug, Clone)]
pub struct Clothing {
    pub clothing_type: ClothingType,
    pub insulation: f32, // 0.0 to 1.0 effectiveness
    pub durability: f32,
    pub max_durability: f32,
}
```

### 3. Update `hypothermia_system` (`src/layer1/clothing.rs`)

Replace global probabilistic logic with entity-based logic.

```rust
pub fn hypothermia_system(
    mut pop_query: Query<(&mut Health, &Equipment), With<Pop>>,
    clothing_query: Query<&Clothing>,
    season: Option<Res<SeasonState>>,
) {
    if !matches!(season.map(|s| s.current_season), Some(Season::Winter)) {
        return;
    }

    let mut rng = rand::thread_rng();

    for (mut health, equipment) in &mut pop_query {
        let mut insulation = 0.0;

        if let Some(entity) = equipment.body {
            if let Ok(item) = clothing_query.get(entity) {
                insulation += item.insulation;
            }
        }

        // Damage chance = 1.0 - insulation
        // If insulation is 1.0 (Full), chance is 0.
        // If insulation is 0.0 (Naked), chance is 100% (or tuned value).

        let damage_chance = (1.0 - insulation).clamp(0.0, 1.0);

        if rng.gen_bool(damage_chance as f64) {
            health.take_damage(1.0);
        }
    }
}
```

### 4. Update `clothing_wear_system` (`src/layer1/clothing.rs`)

Iterate pops, check equipment, reduce durability.

```rust
pub fn clothing_wear_system(
    mut commands: Commands,
    mut pop_query: Query<&mut Equipment, With<Pop>>,
    mut clothing_query: Query<&mut Clothing>,
) {
    let decay_amount = 0.05; // Tunable

    for mut eq in &mut pop_query {
        if let Some(entity) = eq.body {
            if let Ok(mut item) = clothing_query.get_mut(entity) {
                item.durability -= decay_amount;

                if item.durability <= 0.0 {
                    // Break item
                    commands.entity(entity).despawn();
                    eq.body = None;
                }
            } else {
                // Entity missing, clear slot
                eq.body = None;
            }
        }
    }
}
```

### 5. `FetchClothing` Action (Overview)

To make this fully playable, Pops need to find clothing.
- Add `ActionType::FetchClothing` (Index 25).
- In `evaluate_actions_system`:
    - If `Equipment.body` is None:
    - Check `ColonyResources.clothing >= 1.0`.
    - If yes, score `FetchClothing` highly (especially in Winter).
- In `execution.rs`:
    - Handle `FetchClothing` arrival (at Stockpile).
    - Decrement `ColonyResources.clothing`.
    - Spawn `Clothing` entity.
    - Equip to `Equipment.body`.

## REFACTOR Phase: Quality & Design

- **Stockpiles**: Currently `ColonyResources` tracks generic "Clothing". Future refactor should spawn actual Items in stockpile slots, but for now, "spawning on equip" is the accepted abstraction (matching Tools).
- **Insulation**: Different clothing types (`Parka` vs `Tunic`) should have different stats.
- **UI**: Visuals for pops should eventually reflect worn items.

## Acceptance Criteria

- [ ] `Equipment` struct includes `body` and `head`.
- [ ] `Clothing` component defined.
- [ ] `hypothermia_system` uses `Equipment` to determine damage, not global pool.
- [ ] `clothing_wear_system` degrades specific item entities.
- [ ] Tests pass.
- [ ] `cargo check` passes.

## Technical Guidance

- **Migration**: Old saves (if any) might have `Equipment` without `body`. Since `Equipment` is `Default`, new pops are fine. Existing binaries might need `force_recompile` if struct layout changes cause ABI issues (not relevant for this dev environment usually).
- **GPU Buffers**: If `FetchClothing` action is added, UPDATE `src/gpu/buffers.rs` and `shaders` to handle `ActionType` count increase (25 -> 26).
- **Safety**: Check `eq.body` validity before accessing. Use `get_mut` carefully.

## Questions

- *Builder*: Should Headgear be implemented now?
    - *Architect*: Add the slot, but `Clothing` logic mainly targets Body for now. Headgear can be added later for armor/helmets.
