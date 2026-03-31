# 765: Diplomatic Wards

## Overview

Hostage diplomacy comes to SCALE. You can now host the children of rival faction leaders as "Wards" (Students/Guests) in your colony. If they are kept happy and educated, relations with their home faction improve. However, if they die or are mistreated, it instantly triggers war. This introduces a tension between the leverage of hosting important wards and the massive liability of keeping them safe.

## Dependencies

- `003` — Population Basics (for `Pop`)
- `031` — Pop Morale (for `Morale`)
- `047` — Pop Relationships
- `068` — Pop Factions (for tracking home factions)
- `146` — Command Center & System Visibility (for triggering wars on Layer 3)
- `764` — The Embassy Sector (for Layer 1 / Layer 3 diplomatic connections)

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Morale;
    use crate::layer3::diplomacy::{DiplomaticStanding, FactionId};

    #[test]
    fn test_diplomatic_ward_happiness_improves_relations() {
        let mut app = App::new();

        let home_faction = FactionId(1);
        app.world_mut().insert_resource(DiplomaticStanding {
            faction_relations: vec![(home_faction, 0.0)].into_iter().collect(),
        });

        // Spawn a ward with high morale
        let ward = app.world_mut().spawn((
            Pop,
            DiplomaticWard { home_faction },
            Morale { current: 90.0, ..Default::default() },
        )).id();

        app.add_systems(Update, process_diplomatic_wards_system);

        // Run system to process ward morale
        app.update();

        // Relations should improve
        let standing = app.world().get_resource::<DiplomaticStanding>().unwrap();
        assert!(standing.faction_relations.get(&home_faction).unwrap() > &0.0);
    }

    #[test]
    fn test_diplomatic_ward_death_triggers_war() {
        let mut app = App::new();

        let home_faction = FactionId(1);
        app.world_mut().insert_resource(DiplomaticStanding {
            faction_relations: vec![(home_faction, 50.0)].into_iter().collect(),
        });

        app.add_event::<WarDeclaredEvent>();

        // Spawn a ward and then kill them
        let ward = app.world_mut().spawn((
            Pop,
            DiplomaticWard { home_faction },
            // Represent dead or dying pop (e.g. low health/dead component)
            Dead,
        )).id();

        app.add_systems(Update, process_ward_deaths_system);

        // Run system
        app.update();

        // War event should be emitted
        let war_events = app.world().resource::<Events<WarDeclaredEvent>>();
        assert_eq!(war_events.len(), 1);

        let mut reader = war_events.get_cursor();
        let event = reader.read(war_events).next().unwrap();
        assert_eq!(event.target_faction, home_faction);

        // Relations should tank
        let standing = app.world().get_resource::<DiplomaticStanding>().unwrap();
        assert_eq!(*standing.faction_relations.get(&home_faction).unwrap(), -100.0);
    }

    #[test]
    fn test_diplomatic_ward_mistreatment_tanks_relations() {
        let mut app = App::new();

        let home_faction = FactionId(1);
        app.world_mut().insert_resource(DiplomaticStanding {
            faction_relations: vec![(home_faction, 50.0)].into_iter().collect(),
        });

        // Spawn a ward with terrible morale
        let ward = app.world_mut().spawn((
            Pop,
            DiplomaticWard { home_faction },
            Morale { current: 10.0, ..Default::default() },
        )).id();

        app.add_systems(Update, process_diplomatic_wards_system);

        app.update();

        // Relations should degrade
        let standing = app.world().get_resource::<DiplomaticStanding>().unwrap();
        assert!(*standing.faction_relations.get(&home_faction).unwrap() < 50.0);
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::needs::Morale;
use crate::layer1::pop::Dead;
use crate::layer3::diplomacy::{DiplomaticStanding, FactionId};

#[derive(Component)]
pub struct DiplomaticWard {
    pub home_faction: FactionId,
}

#[derive(Event)]
pub struct WarDeclaredEvent {
    pub target_faction: FactionId,
}

pub fn process_diplomatic_wards_system(
    ward_query: Query<(&DiplomaticWard, &Morale)>,
    mut standing: ResMut<DiplomaticStanding>,
) {
    for (ward, morale) in ward_query.iter() {
        if let Some(relation) = standing.faction_relations.get_mut(&ward.home_faction) {
            if morale.current >= 80.0 {
                // High morale improves relations slowly
                *relation = (*relation + 0.5).clamp(-100.0, 100.0);
            } else if morale.current <= 20.0 {
                // Low morale damages relations
                *relation = (*relation - 1.0).clamp(-100.0, 100.0);
            }
        }
    }
}

pub fn process_ward_deaths_system(
    dead_wards: Query<&DiplomaticWard, With<Dead>>,
    mut standing: ResMut<DiplomaticStanding>,
    mut war_events: EventWriter<WarDeclaredEvent>,
) {
    for ward in dead_wards.iter() {
        if let Some(relation) = standing.faction_relations.get_mut(&ward.home_faction) {
            *relation = -100.0; // Instant bottom relations
        }
        war_events.send(WarDeclaredEvent {
            target_faction: ward.home_faction,
        });
    }
}
```

## REFACTOR Phase: Quality & Design

- Ensure the standing struct safely initializes non-existent factions if a ward appears before explicit Layer 3 contact, or strictly enforce dependencies where wards are only generated from known factions.
- Tie ward arrivals to diplomatic events on Layer 3 (e.g. an "Offer Ward" event from `layer3/diplomacy.rs`).
- Connect `WarDeclaredEvent` to the chronicle system so the player gets a narrative notification when a ward dies ("The Warlord's son has perished. War is upon us.").
- Tweak the rates of relation improvement/degradation based on time delta (`Time<Virtual>`) rather than ticking blindly per frame to prevent hyper-fast relation shifts.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for the new module.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Wards with high morale improve relations over time.
- [ ] Wards with low morale damage relations over time.
- [ ] Dead wards instantly tank relations to -100 and emit a `WarDeclaredEvent`.

## Technical Guidance

- Place the `DiplomaticWard` component in a new file `src/layer1/diplomacy/wards.rs` or add to `src/layer1/social/` depending on architectural preference.
- Make sure to register the `process_diplomatic_wards_system` and `process_ward_deaths_system` in the appropriate system sets (e.g., `Layer1SystemSet::Observation` or `Layer1SystemSet::Update`).
- Add the `WarDeclaredEvent` to `src/layer3/diplomacy.rs` or `src/layer3/events/` if it doesn't already exist.

## Questions

*Builder: add questions here if spec is unclear.*
