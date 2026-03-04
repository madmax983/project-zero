# 284: Generational Hoarders

## 1. Overview
Grandparents refuse to throw away scrap, eventually clogging the colony's logistics.

**Layer:** 1

**Fantasy:** The colony's supply of useful scrap and obsolete tools mysteriously disappears, only to be found meticulously organized in the bedrooms of retirees who refuse to give up their "keepsakes." This forces the player into a difficult choice between respecting the elders and repossessing critical resources during emergencies.

**Mechanic:** Older Pops (based on Age or `Generation` traits) have a chance to develop a `Hoarder` trait. Pops with this trait will periodically claim random low-value items (scrap, waste, obsolete tools) from the colony's usable inventory and store them in a `Hoard` component attached to their quarters/bed. These items are removed from the global `ColonyResources`, but provide a small mood buff to the Hoarder.

**Emergence:** When a critical life support failure requires scrap metal for jury-rigging, the colony's entire supply might be locked away. The player must choose between letting the colony suffocate or issuing an edict to forcibly evict/confiscate the elders' hoards, causing a massive morale drop for the `Old Guard` faction.

**Tension:** Respecting elder privileges for stability vs. repossessing critical resources in emergencies.

## 2. Dependencies
- **062 Pop Lifecycle:** Needs `Age` or `Generation` tracking.
- **022 Resource Stockpiles:** Needs inventory management system.
- **103 Private Stashes:** Built upon the existing stash logic but tied to the `Hoarder` trait.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Age;
    use crate::layer1::inventory::{ItemType, Inventory};
    use crate::layer1::needs::Morale;

    #[test]
    fn test_elder_develops_hoarder_trait() {
        let mut world = World::new();
        // Setup an elder pop
        let elder = world.spawn((Age { cycles: 65 }, Traits::default())).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(check_for_hoarder_trait_system);
        schedule.run(&mut world);

        let traits = world.get::<Traits>(elder).unwrap();
        assert!(traits.has(Trait::Hoarder), "Elder should have a chance to develop the Hoarder trait");
    }

    #[test]
    fn test_hoarder_claims_scrap() {
        let mut world = World::new();
        let hoarder = world.spawn((Trait::Hoarder, Hoard::default(), Morale::default())).id();
        let stockpile = world.spawn(Inventory::with_items(ItemType::Scrap, 10)).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(hoarder_collection_system);
        schedule.run(&mut world);

        let hoard = world.get::<Hoard>(hoarder).unwrap();
        let stockpile_inv = world.get::<Inventory>(stockpile).unwrap();

        assert!(hoard.items.contains(&ItemType::Scrap), "Hoarder should have collected scrap");
        assert_eq!(stockpile_inv.count(ItemType::Scrap), 9, "Scrap should be removed from stockpile");
    }

    #[test]
    fn test_hoard_grants_morale_buff() {
        let mut world = World::new();
        let mut hoard = Hoard::default();
        hoard.items.push(ItemType::Scrap);

        let hoarder = world.spawn((Trait::Hoarder, hoard, Morale::default())).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_hoard_morale_buff_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(hoarder).unwrap();
        assert!(morale.current > 50.0, "Hoarder should receive a morale buff from their hoard");
    }

    #[test]
    fn test_confiscate_hoard_lowers_morale() {
        let mut world = World::new();
        let mut hoard = Hoard::default();
        hoard.items.push(ItemType::Scrap);

        let hoarder = world.spawn((Trait::Hoarder, hoard, Morale { current: 80.0, ..default() })).id();

        // Trigger confiscation
        world.send_event(ConfiscateHoardEvent { target: hoarder });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_confiscation_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(hoarder).unwrap();
        assert!(morale.current < 80.0, "Confiscating the hoard should heavily penalize morale");
        let empty_hoard = world.get::<Hoard>(hoarder).unwrap();
        assert!(empty_hoard.items.is_empty(), "Hoard should be empty after confiscation");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::pop::{Age, Traits, Trait};
use crate::layer1::inventory::{ItemType, Inventory};
use crate::layer1::needs::Morale;

#[derive(Component, Default)]
pub struct Hoard {
    pub items: Vec<ItemType>,
}

#[derive(Event)]
pub struct ConfiscateHoardEvent {
    pub target: Entity,
}

pub fn check_for_hoarder_trait_system(mut query: Query<(&Age, &mut Traits)>) {
    for (age, mut traits) in query.iter_mut() {
        // Simple threshold for MVP
        if age.cycles >= 60 && !traits.has(Trait::Hoarder) {
            // Simplified chance: always adds for this green phase
            traits.add(Trait::Hoarder);
        }
    }
}

pub fn hoarder_collection_system(
    mut hoarders: Query<(&Trait, &mut Hoard)>,
    mut stockpiles: Query<&mut Inventory>,
) {
    for (traits, mut hoard) in hoarders.iter_mut() {
        if traits.has(Trait::Hoarder) {
            for mut inv in stockpiles.iter_mut() {
                if inv.count(ItemType::Scrap) > 0 {
                    inv.remove(ItemType::Scrap, 1);
                    hoard.items.push(ItemType::Scrap);
                    break; // Only take one item per tick
                }
            }
        }
    }
}

pub fn apply_hoard_morale_buff_system(mut query: Query<(&Hoard, &mut Morale)>) {
    for (hoard, mut morale) in query.iter_mut() {
        if !hoard.items.is_empty() {
            morale.current += 5.0; // Flat buff for having items
        }
    }
}

pub fn process_confiscation_system(
    mut events: EventReader<ConfiscateHoardEvent>,
    mut query: Query<(&mut Hoard, &mut Morale)>,
) {
    for ev in events.read() {
        if let Ok((mut hoard, mut morale)) = query.get_mut(ev.target) {
            if !hoard.items.is_empty() {
                morale.current -= 30.0; // Heavy penalty
                hoard.items.clear();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration with Utility AI:** The act of "claiming" an item should be an action scored by the Utility AI (`ActionType::Stash`) rather than an instant teleportation in the system. The Hoarder trait should modify the weight of this action.
- **Specific Item Types:** Hoarders shouldn't just grab generic "Scrap"; they should target items flagged as `Obsolete` or `LowValue`.
- **Edict Integration:** The `ConfiscateHoardEvent` should be triggered by a global `Edict` or policy, not just a standalone event.
- **Chronicle Hook:** Add a lore template for `STASH_FOUND` or `HOARD_CONFISCATED` to inform the player when they upset the elders.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new `hoarder.rs` module.
- [ ] Elders successfully acquire the `Hoarder` trait.
- [ ] Items are removed from public stockpiles and placed in private `Hoard` components.
- [ ] Confiscation correctly penalizes morale and empties the hoard.

## 7. Technical Guidance
- Add `Trait::Hoarder` to the `Trait` enum in `src/layer1/traits.rs`.
- Create a new module `src/layer1/social/hoarder.rs` for the systems.
- Register the systems in `src/layer1/systems/social.rs`.
- When integrating with Utility AI, make sure the Hoarder trait heavily biases the Pop towards the existing `ActionType::Stash` (from Spec 103) but specifically targeting low-value items.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
