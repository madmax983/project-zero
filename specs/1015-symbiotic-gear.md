# 1015: Symbiotic Gear

## 1. Overview
Symbiotic gear represents "living" equipment (Bio-Suits, Parasite-Guns) that provides superior stats but introduces a new biological need: Hunger. If the wearer doesn't feed the gear by consuming extra food, the equipment will begin consuming the wearer's health. This introduces extreme tension in resource-scarce situations, where players might have to forcefully unequip vital gear to prevent their Pops from being eaten alive.

## 2. Dependencies
- Layer 1 `Equipment`/`Inventory` system.
- Layer 1 `Needs` system (`Hunger`).
- Layer 1 `Health`/`Damage` mechanics.
- Layer 1 `Pop` entity.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::Hunger;
    use crate::layer1::health::{Health, DamageEvent};
    use crate::layer1::inventory::Equipment;

    #[test]
    fn test_symbiotic_gear_increases_hunger_decay() {
        let mut app = App::new();
        app.add_systems(Update, symbiotic_hunger_modifier_system);

        let pop = app.world_mut().spawn((
            Hunger { value: 100.0, decay_rate: 1.0 },
            Equipment { has_symbiote: true },
        )).id();

        app.update();

        let hunger = app.world().get::<Hunger>(pop).unwrap();
        assert!(hunger.decay_rate > 1.0, "Symbiotic gear should increase the wearer's hunger decay rate.");
    }

    #[test]
    fn test_starving_symbiote_damages_wearer() {
        let mut app = App::new();
        app.add_event::<DamageEvent>();
        app.add_systems(Update, starving_symbiote_damage_system);

        let pop = app.world_mut().spawn((
            Health { current: 100.0, max: 100.0 },
            Hunger { value: 0.0, decay_rate: 2.0 }, // Pop is starving
            Equipment { has_symbiote: true },
        )).id();

        app.update();

        let damage_events = app.world().resource::<Events<DamageEvent>>();
        let mut reader = damage_events.get_reader();
        let mut found = false;
        for event in reader.read(damage_events) {
            if event.target == pop && event.source == "Symbiote" {
                found = true;
            }
        }

        assert!(found, "A starving symbiote should damage its wearer.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/symbiotic_gear.rs
use bevy::prelude::*;
use crate::layer1::needs::Hunger;
use crate::layer1::inventory::Equipment;
use crate::layer1::health::DamageEvent;

const SYMBIOTE_HUNGER_MULTIPLIER: f32 = 2.0;

pub fn symbiotic_hunger_modifier_system(
    mut query: Query<(&mut Hunger, &Equipment)>,
) {
    for (mut hunger, equip) in query.iter_mut() {
        if equip.has_symbiote {
            // MVP: Double the decay rate while worn
            // We assume a base decay is set elsewhere, here we just ensure the multiplier
            // For a robust system, this should be applied as a buff/debuff modifier
            hunger.decay_rate = 1.0 * SYMBIOTE_HUNGER_MULTIPLIER;
        } else {
            hunger.decay_rate = 1.0;
        }
    }
}

pub fn starving_symbiote_damage_system(
    query: Query<(Entity, &Hunger, &Equipment)>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    for (entity, hunger, equip) in query.iter() {
        if equip.has_symbiote && hunger.value <= 0.0 {
            damage_events.send(DamageEvent {
                target: entity,
                amount: 5.0, // Significant damage per tick when starving
                source: "Symbiote".to_string(),
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Modifier Stacking:** Hardcoding `decay_rate = 1.0 * MULTIPLIER` will break if there are other traits affecting metabolism. Need to integrate with a generic `Modifier` component attached to `Hunger`.
- **Feeding Mechanics:** The symbiote should probably have its *own* hidden hunger value, filled when the Pop eats. If the Pop's hunger is 0, the symbiote starts eating the Pop. This allows Pops to prioritize feeding their suit.
- **Unequip Penalty:** To heighten tension, forcibly removing a starving symbiote should cause immediate burst damage or apply a bleeding effect.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_symbiotic_gear_increases_hunger_decay` passes.
- [ ] Test `test_starving_symbiote_damages_wearer` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- Ensure `DamageEvent` ties into the core health system so the Pop actually takes the damage and can die.
- If the game doesn't have an `Equipment` struct that flags `has_symbiote`, use the specific item ID or trait system used for inventory slots.

## 8. Questions
*Builder: add questions here if spec is unclear.*
