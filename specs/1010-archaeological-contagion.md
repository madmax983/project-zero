# 1010: The Archaeological Contagion

## 1. Overview
Excavating deep ruins has a chance to infect Pops with "Ancient Routines." Infected Pops periodically stop their normal jobs to perform bizarre, nonsensical tasks (e.g., stacking rocks in specific patterns, chanting at the sun) that disrupt the economy. However, observing these routines slowly generates rare "Lost Tech" research points. This forces players to choose between the immense value of lost technology and the chaotic disruption to their colony's basic economy.

## 2. Dependencies
- Layer 1 `Mining` and `Excavation` discovery mechanics.
- Layer 1 `Pop` entity and traits system.
- Layer 1 `Utility AI` (Work assignment, Task priorities).
- Layer 1 `Research` or `Tech` system.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::mining::ExcavationEvent;
    use crate::layer1::pop::Pop;
    use crate::layer1::utility_ai::{ActionType, UtilityWeights};
    use crate::layer1::research::{ResearchPoints, TechCategory};

    #[test]
    fn test_excavating_ancient_ruins_infects_miner_with_routines() {
        let mut app = App::new();
        app.add_event::<ExcavationEvent>();
        app.add_systems(Update, archaeological_infection_system);

        let miner = app.world_mut().spawn(Pop).id();

        app.world_mut().resource_mut::<Events<ExcavationEvent>>().send(ExcavationEvent {
            colony: Entity::PLACEHOLDER,
            miner,
            discovery_type: "AncientRuins".to_string(),
        });

        app.update();

        assert!(app.world().get::<AncientRoutineInfection>(miner).is_some(), "Excavating Ancient Ruins should infect the miner.");
    }

    #[test]
    fn test_infected_pop_generates_lost_tech_while_performing_routine() {
        let mut app = App::new();
        app.insert_resource(ResearchPoints {
            category: TechCategory::LostTech,
            amount: 0.0,
        });
        app.add_systems(Update, ancient_routine_observation_system);

        // Spawn pop currently executing an Ancient Routine action
        app.world_mut().spawn((
            Pop,
            AncientRoutineInfection { active: true },
            ActionType::PerformAncientRoutine,
        ));

        app.update();

        let tech = app.world().resource::<ResearchPoints>();
        assert!(tech.amount > 0.0, "Pops actively performing ancient routines should generate Lost Tech points.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/archaeological_contagion.rs
use bevy::prelude::*;
use crate::layer1::mining::ExcavationEvent;
use crate::layer1::pop::Pop;
use crate::layer1::utility_ai::ActionType;
use crate::layer1::research::{ResearchPoints, TechCategory};

#[derive(Component)]
pub struct AncientRoutineInfection {
    pub active: bool,
}

pub fn archaeological_infection_system(
    mut commands: Commands,
    mut events: EventReader<ExcavationEvent>,
) {
    for event in events.read() {
        if event.discovery_type == "AncientRuins" {
            // MVP: 100% infection rate for test simplicity, should be RNG based later
            commands.entity(event.miner).insert(AncientRoutineInfection {
                active: true,
            });
        }
    }
}

pub fn ancient_routine_observation_system(
    query: Query<(&AncientRoutineInfection, &ActionType)>,
    mut research: ResMut<ResearchPoints>,
) {
    for (infection, action) in query.iter() {
        if infection.active && *action == ActionType::PerformAncientRoutine {
            // Generate tech while the routine is being executed
            if research.category == TechCategory::LostTech {
                research.amount += 0.1;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Utility AI Integration:** `ActionType::PerformAncientRoutine` needs to be scored by the Utility AI. Infected pops should occasionally have this score spike very high, overriding their normal needs/jobs.
- **Infection Spread:** The contagion shouldn't just stay on the miner; it should have a small chance to spread to other Pops who spend time near the infected individual (simulating learning the routine).
- **Observation Mechanics:** Tech shouldn't generate just because the Pop is doing the routine; it should generate faster if *other* Pops with high intelligence/researcher jobs are assigned to watch them.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_excavating_ancient_ruins_infects_miner_with_routines` passes.
- [ ] Test `test_infected_pop_generates_lost_tech_while_performing_routine` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- `ActionType::PerformAncientRoutine` must be added to the main enum of actions the AI can execute.
- Ensure `ResearchPoints` resource handles the `LostTech` category correctly without breaking standard research mechanics.

## 8. Questions
*Builder: add questions here if spec is unclear.*
