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
    use scale::layer1::factions::{Faction, FactionMember};
    use scale::layer1::pop::{Pop, Morale};
    use scale::shared::chronicle::AtrocityScore;

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<AtrocityScore>();
        // Minimal setup
        world
    }

    #[test]
    fn test_penitent_faction_forms_on_high_atrocity() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            FactionMember { faction_id: Faction::Colony },
            Morale { value: 0.5, ..Default::default() },
        )).id();

        let mut atrocity = world.resource_mut::<AtrocityScore>();
        atrocity.score = 100.0; // High score

        let mut schedule = Schedule::default();
        schedule.add_systems(check_penitent_faction_formation_system);
        schedule.run(&mut world);

        let faction = world.get::<FactionMember>(pop).unwrap();
        assert_eq!(faction.faction_id, Faction::Penitent); // Pop converted
    }

    #[test]
    fn test_penitent_faction_strikes_if_ignored() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            FactionMember { faction_id: Faction::Penitent },
            ReparationDemands { ignored_ticks: 100 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_reparation_strikes_system);
        schedule.run(&mut world);

        let demands = world.get::<ReparationDemands>(pop).unwrap();
        assert!(demands.is_striking); // Strike initiated
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/factions/bureau_of_regrets.rs
use bevy_ecs::prelude::*;
use crate::layer1::factions::{Faction, FactionMember};
use crate::shared::chronicle::AtrocityScore;

#[derive(Component)]
pub struct ReparationDemands {
    pub ignored_ticks: u32,
    pub is_striking: bool,
}

pub fn check_penitent_faction_formation_system(
    atrocity_score: Res<AtrocityScore>,
    mut pops: Query<(Entity, &mut FactionMember), With<Pop>>,
) {
    if atrocity_score.score >= 100.0 {
        // Convert some random pops or unhappy pops
        for (entity, mut faction) in pops.iter_mut() {
            if faction.faction_id == Faction::Colony && rand::random::<f32>() < 0.1 {
                faction.faction_id = Faction::Penitent;
                commands.entity(entity).insert(ReparationDemands {
                    ignored_ticks: 0,
                    is_striking: false,
                });
            }
        }
    }
}

pub fn process_reparation_strikes_system(
    mut demands: Query<&mut ReparationDemands>,
) {
    for mut demand in demands.iter_mut() {
        demand.ignored_ticks += 1;
        if demand.ignored_ticks >= 100 {
            demand.is_striking = true;
            // Additional logic to halt work or sabotage
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
