# 496: The Bureau of Regrets

## 1. Overview
A faction dedicated solely to undoing the mistakes of the past, even if it destroys the present. If your colony has a high "Atrocity" score (from starving pops, executions, etc.), the "Penitent Faction" forms. They demand the dismantling of the structures or technologies that caused the atrocities (e.g., destroying the very hydroponics bays that saved the colony but required forced labor). If ignored, they initiate "Reparation Strikes."

## 2. Dependencies
- `068` Pop Factions (Implemented)
- `233` Public Grievances (Implemented)
- `010` Chronicle System (Implemented)

## 3. RED Phase: Tests First
```rust
// tests/bureau_of_regrets_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use scale::layer1::social::factions::{FactionId, FactionMember, Factions, FactionDemand};
    use scale::layer1::pop::Pop;
    use scale::layer1::psychology::needs::Needs;

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<AtrocityScore>();
        world.init_resource::<Factions>();
        // Minimal setup
        world
    }

    #[test]
    fn test_penitent_faction_forms_on_high_atrocity() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            FactionMember { faction_id: Some(FactionId::Unaligned) },
            Needs { ..Default::default() }, // Using Needs instead of standalone Morale
        )).id();

        let mut atrocity = world.resource_mut::<AtrocityScore>();
        atrocity.score = 100.0; // High score

        let mut schedule = Schedule::default();
        schedule.add_systems(check_penitent_faction_formation_system);
        schedule.run(&mut world);

        let faction = world.get::<FactionMember>(pop).unwrap();
        assert_eq!(faction.faction_id, Some(FactionId::Penitent)); // Pop converted
    }

    #[test]
    fn test_penitent_faction_strikes_if_ignored() {
        let mut world = setup_world();

        let mut factions = world.resource_mut::<Factions>();
        factions.map.insert(FactionId::Penitent, scale::layer1::social::factions::FactionData {
            active_demand: Some(FactionDemand {
                remaining_time: 1.0,
                policy: None,
                description: "Reparations".to_string(),
            }),
            state: scale::layer1::social::factions::FactionState::Unhappy,
            ..Default::default()
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(scale::layer1::social::factions::update_faction_strikes_system);
        schedule.run(&mut world);

        let factions = world.resource::<Factions>();
        let penitent_data = factions.get(FactionId::Penitent).unwrap();
        assert_eq!(penitent_data.state, scale::layer1::social::factions::FactionState::Striking); // Strike initiated
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/factions/bureau_of_regrets.rs
use bevy_ecs::prelude::*;
use crate::layer1::social::factions::{FactionId, FactionMember, Factions, FactionDemand, FactionState};

#[derive(Resource, Default)]
pub struct AtrocityScore {
    pub score: f32,
}

pub fn check_penitent_faction_formation_system(
    atrocity_score: Res<AtrocityScore>,
    mut pops: Query<&mut FactionMember, With<crate::layer1::entities::pop::Pop>>,
    mut factions: ResMut<Factions>,
) {
    if atrocity_score.score >= 100.0 {
        // Convert some random pops
        for mut faction in pops.iter_mut() {
            if faction.faction_id == Some(FactionId::Unaligned) && rand::random::<f32>() < 0.1 {
                faction.faction_id = Some(FactionId::Penitent);
            }
        }

        // Ensure the Penitent faction data exists and has a demand
        if let Some(data) = factions.map.get_mut(&FactionId::Penitent) {
            if data.active_demand.is_none() {
                data.active_demand = Some(FactionDemand {
                    policy: None,
                    remaining_time: 2000.0,
                    description: "Reparations".to_string(),
                });
                data.state = FactionState::Unhappy;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Atrocity Tracking**: Hook into executions, starvation events, or `PenalState` to continuously accumulate `AtrocityScore`.
- **Dismantling Logic**: The Penitent Faction should specifically demand the destruction of specific buildings (e.g., Prisons, Hydroponics bays built during famine). Satisfying the demand lowers their unrest and slowly dissolves the faction.
- **Work Interruption**: Ensure that a striking Pop refuses to take normal jobs from the `Utility AI System` until their demands are met or they are violently suppressed.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/factions/bureau_of_regrets.rs`.
- [ ] High AtrocityScore spawns the Penitent faction.
- [ ] Ignored demands lead to work strikes.

## 7. Technical Guidance
- Modify `AtrocityScore` to be a global resource if it isn't already part of the `ChronicleSystem`.
- Use the `Utility AI Buffer` to zero out work scores for striking Pops.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
- **Architectural Contradictions:** `AtrocityScore` does not exist in `ChronicleSystem` (or anywhere). `Faction::Colony` does not exist in `FactionId` enum. `Morale` is not a standalone component (`needs.rs` uses calculated morale). `ReparationDemands` logic is unclear regarding how it relates to `FactionDemand` in the current `factions.rs` which has an `active_demand` field. `process_reparation_strikes_system` tests fail to account for `Utility AI Buffer` details. Moving to next task.


*Architect:* The RED and GREEN phases have been rewritten to introduce `AtrocityScore` as a `Resource`, utilize `FactionId::Unaligned` in place of the non-existent `Faction::Colony`, properly use `Needs` instead of `Morale`, and hook into the existing `Factions` and `FactionDemand` structures rather than creating a bespoke component.