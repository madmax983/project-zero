# 451 - The Spiteful Will

## 1. Overview
**Layer:** 1
**Fantasy:** Death is not the end of a grudge. Inheritances cause societal rifts.
**Mechanic:** When a wealthy or high-status Pop dies, their belongings (high-quality tools, private stashes, housing rights) are distributed. However, Pops with the "Spiteful" trait can leave a Will that explicitly denies their rivals or gives everything to the colony pet, or requires ridiculous conditions to inherit.
**Emergence:** Your best engineer dies and leaves their masterwork welding torch to a literal space-cat because they hated the backup engineer. The backup engineer's morale tanks from the insult, and they refuse to work until the cat is "dealt with."
**Tension:** Respecting the final wishes of Pops (boosting tradition/morale) vs. confiscating the loot for the good of the colony and causing massive unrest among the deceased's friends.

## 2. Dependencies
- `Pop` component
- `PopTraits` (specifically `Spiteful`)
- `Inventory` / `PrivateStash` for holding belongings
- `Morale` system for applying modifiers
- `DeathEvent` or similar for triggering inheritance

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Trait};
    use crate::layer1::inventory::Inventory;
    use crate::layer1::morale::{Morale, MoodModifier};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<DeathEvent>();
        app.add_event::<InheritanceEvent>();
        app.add_systems(Update, process_spiteful_will_system);
        app
    }

    #[test]
    fn test_spiteful_will_triggers_on_death() {
        let mut app = setup_app();

        let dead_pop = app.world_mut().spawn((
            Pop,
            Trait::Spiteful,
            Inventory { items: vec!["Masterwork Tool".to_string()] }
        )).id();

        app.world_mut().resource_mut::<Events<DeathEvent>>().send(DeathEvent { entity: dead_pop });

        app.update();

        let events = app.world().resource::<Events<InheritanceEvent>>();
        let mut reader = events.get_cursor();
        assert!(reader.read(events).len() > 0, "Inheritance event should be triggered for spiteful pop");
    }

    #[test]
    fn test_confiscating_loot_causes_unrest() {
        let mut app = setup_app();

        let heir = app.world_mut().spawn((
            Pop,
            Morale::default()
        )).id();

        // Simulate the player overriding the will
        app.world_mut().resource_mut::<Events<OverrideWillEvent>>().send(OverrideWillEvent {
            affected_pops: vec![heir]
        });

        app.update();

        let morale = app.world().get::<Morale>(heir).unwrap();
        assert!(morale.modifiers.iter().any(|m| m.label == "Will Overridden"), "Heir should be upset by override");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pop::{Pop, Trait};
use crate::layer1::inventory::Inventory;
use crate::layer1::morale::{Morale, MoodModifier};

#[derive(Event)]
pub struct DeathEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct InheritanceEvent {
    pub deceased: Entity,
    pub items: Vec<String>,
}

#[derive(Event)]
pub struct OverrideWillEvent {
    pub affected_pops: Vec<Entity>,
}

pub fn process_spiteful_will_system(
    mut death_events: EventReader<DeathEvent>,
    mut inheritance_events: EventWriter<InheritanceEvent>,
    query: Query<(&Trait, &Inventory), With<Pop>>,
) {
    for event in death_events.read() {
        if let Ok((pop_trait, inventory)) = query.get(event.entity) {
            if *pop_trait == Trait::Spiteful && !inventory.items.is_empty() {
                inheritance_events.send(InheritanceEvent {
                    deceased: event.entity,
                    items: inventory.items.clone(),
                });
            }
        }
    }
}

pub fn process_override_will_system(
    mut override_events: EventReader<OverrideWillEvent>,
    mut query: Query<&mut Morale, With<Pop>>,
) {
    for event in override_events.read() {
        for entity in &event.affected_pops {
            if let Ok(mut morale) = query.get_mut(*entity) {
                morale.modifiers.push(MoodModifier {
                    label: "Will Overridden".to_string(),
                    value: -0.2,
                    duration: 100.0,
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Currently using `String` for items; replace with actual `Entity` references for items.
- Need to expand inheritance to handle finding valid targets (e.g. rivals, pets).
- Need a player notification so they can choose whether to override the will.
- Ensure traits are handled using a set or proper component layout instead of a single `Trait` enum if multiple traits are supported.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Death of a Pop with `Spiteful` trait correctly triggers inheritance events.
- [ ] Player overriding inheritance correctly applies negative morale to affected pops.

## 7. Technical Guidance
- Implement `OverrideWillEvent` hook to the Command Center UI so the player can actually intervene.
- Tie `InheritanceEvent` into the `Inventory` transfer system to actually move the items if the player doesn't intervene.

## 8. Questions
*Builder: add questions here if spec is unclear.*
