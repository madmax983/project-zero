# Specification 458: The Monumental Ego

## 1. Overview
This feature introduces a cross-layer interaction (Layer 1 -> Layer 2) where a beloved planetary leader slowly descends into narcissism, demanding the colony bankrupt itself to build their legacy. If a Layer 1 Mayor or Governor reaches maximum "Prestige" and stays there too long, they develop the "Megalomania" trait. They issue an edict to build a colossal, mechanically useless "Vanity Megastructure" visible from Layer 2. Refusing triggers a civil war led by their loyalists. Completing the statue provides a massive diplomatic boost with superficial Layer 3 empires, but its shadow permanently lowers the morale of the slums built around its base.

## 2. Dependencies
- `054` Colony Edicts (Edict system)
- `068` Pop Factions (Civil war / loyalists mechanics)
- `152` Orbital Stations (Visibility in Layer 2)
- `329` Leader Ascension (Leader Prestige)

## 3. RED Phase: Tests First

```rust
// tests/monumental_ego_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use scale::layer1::leader::{Leader, Prestige, TraitMegalomania};
    use scale::layer1::edicts::{Edict, EdictType, ActiveEdicts};
    use scale::layer1::factions::{Faction, FactionStance};
    use scale::layer1::buildings::{Building, VanityMegastructure};
    use scale::layer1::social::Morale;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(ActiveEdicts::default());
        world
    }

    #[test]
    fn test_leader_develops_megalomania() {
        let mut world = setup_world();

        // Arrange: A leader at max prestige for a long time
        let leader_entity = world.spawn((
            Leader { is_governor: true },
            Prestige { current: 100.0, max: 100.0, time_at_max: 50.0 },
        )).id();

        // Act: Run the ego development system
        let mut schedule = Schedule::default();
        schedule.add_systems(monumental_ego_development_system);
        schedule.run(&mut world);

        // Assert: Leader now has the Megalomania trait
        assert!(world.get::<TraitMegalomania>(leader_entity).is_some());
    }

    #[test]
    fn test_megalomania_issues_vanity_edict() {
        let mut world = setup_world();

        let leader_entity = world.spawn((
            Leader { is_governor: true },
            Prestige { current: 100.0, max: 100.0, time_at_max: 50.0 },
            TraitMegalomania,
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(megalomania_edict_system);
        schedule.run(&mut world);

        let active_edicts = world.get_resource::<ActiveEdicts>().unwrap();
        assert!(active_edicts.contains(&EdictType::BuildVanityMegastructure));
    }

    #[test]
    fn test_refusing_vanity_edict_triggers_civil_war() {
        let mut world = setup_world();

        let loyalist_faction = world.spawn((
            Faction { name: "Governor Loyalists".to_string() },
            FactionStance::Neutral,
        )).id();

        // Act: trigger refusal
        world.send_event(EdictRefusedEvent { edict: EdictType::BuildVanityMegastructure });

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_vanity_edict_refusal_system);
        schedule.run(&mut world);

        // Assert: Loyalists declare civil war
        let faction_stance = world.get::<FactionStance>(loyalist_faction).unwrap();
        assert_eq!(*faction_stance, FactionStance::CivilWar);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/leader/ego.rs

use bevy_ecs::prelude::*;
use crate::layer1::leader::{Leader, Prestige, TraitMegalomania};
use crate::layer1::edicts::{EdictType, ActiveEdicts, EdictRefusedEvent};
use crate::layer1::factions::{Faction, FactionStance};

#[derive(Component)]
pub struct TraitMegalomania;

pub fn monumental_ego_development_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Leader, &Prestige), Without<TraitMegalomania>>,
) {
    for (entity, leader, prestige) in query.iter_mut() {
        if leader.is_governor && prestige.current >= prestige.max && prestige.time_at_max > 40.0 {
            commands.entity(entity).insert(TraitMegalomania);
        }
    }
}

pub fn megalomania_edict_system(
    query: Query<&TraitMegalomania, With<Leader>>,
    mut active_edicts: ResMut<ActiveEdicts>,
) {
    for _ in query.iter() {
        if !active_edicts.contains(&EdictType::BuildVanityMegastructure) {
            active_edicts.add(EdictType::BuildVanityMegastructure);
        }
    }
}

pub fn handle_vanity_edict_refusal_system(
    mut events: EventReader<EdictRefusedEvent>,
    mut factions: Query<&mut FactionStance, With<Faction>>,
) {
    for event in events.read() {
        if event.edict == EdictType::BuildVanityMegastructure {
            for mut stance in factions.iter_mut() {
                // In minimal implementation, all factions rebel. In refactor, filter by loyalists.
                *stance = FactionStance::CivilWar;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Targeted Rebellions**: Update `handle_vanity_edict_refusal_system` to correctly filter for factions with high loyalty to the specific leader rather than causing all factions to rebel.
- **Megastructure Shadow Effects**: Implement the spatial query to lower `Morale` of Pops residing within a radius of the `VanityMegastructure`.
- **Layer 3 Integration**: Connect the completion of the `VanityMegastructure` to the diplomatic event system, broadcasting the prestige boost to neighboring empires.
- **Resource Sink Scaling**: Scale the required resources (steel/fuel) for the `VanityMegastructure` based on the colony's current total wealth, ensuring it always bankrupts or severely impacts the economy.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/leader/ego.rs`.
- [ ] Governor correctly gains `TraitMegalomania` after prolonged max prestige.
- [ ] Refusal of the edict successfully triggers a civil war stance in loyal factions.

## 7. Technical Guidance
- Ensure that `Prestige::time_at_max` is properly incremented by the main leader progression system.
- The `VanityMegastructure` should be a unique building type that hooks into the standard construction system but requires astronomical resources.

## 8. Questions
*Builder: add questions here if spec is unclear.*
