# 286: Great Works

## 1. Overview
Multi-stage construction projects requiring massive resources and time that provide massive benefits or win conditions upon completion.

**Layer:** 1 -> 2

**Fantasy:** Leaving a mark on the universe. Building something so massive it can be seen from orbit.

**Mechanic:** A `GreatWork` is a special building entity that processes construction in discrete "Phases". Each phase acts like a massive construction designation requiring unique resources and labor. Only when all phases are completed does the structure become operational and grant global buffs or Layer 2 capabilities.

**Emergence:** The entire colony's economy warps around "The Project." Stopping construction causes social unrest (dashed hopes).

**Tension:** Invest in current survival or future glory? All-in on the Wonder or diversified growth?

## 2. Dependencies
- **020 Construction Costs:** Needs the basic construction logic.
- **021 Utility AI Work Action:** Needs Pops to be able to execute building tasks.
- **031 Pop Morale:** For the mood buffs/debuffs associated with the project.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::construction::{ConstructionProgress, ConstructionCost};

    #[test]
    fn test_great_work_initializes_in_phase_one() {
        let mut world = World::new();
        let work_id = spawn_great_work(&mut world, "Space Elevator", vec![
            ConstructionCost { item_type: ItemType::Steel, amount: 1000 },
            ConstructionCost { item_type: ItemType::Electronics, amount: 500 }
        ]);

        let work = world.get::<GreatWork>(work_id).unwrap();
        assert_eq!(work.current_phase, 0);
        assert!(!work.is_completed());
    }

    #[test]
    fn test_great_work_advances_phase_when_progress_filled() {
        let mut world = World::new();
        let work_id = spawn_great_work(&mut world, "Planetary Shield", vec![
            ConstructionCost { item_type: ItemType::Concrete, amount: 100 }, // Phase 0
            ConstructionCost { item_type: ItemType::EnergyCore, amount: 5 } // Phase 1
        ]);

        // Manually complete phase 0
        let mut progress = world.get_mut::<ConstructionProgress>(work_id).unwrap();
        progress.current_work = progress.total_work_required;

        let mut schedule = Schedule::default();
        schedule.add_systems(process_great_work_phases);
        schedule.run(&mut world);

        let work = world.get::<GreatWork>(work_id).unwrap();
        assert_eq!(work.current_phase, 1, "Should have advanced to phase 1.");
        assert!(!work.is_completed(), "Should not be completed yet.");
    }

    #[test]
    fn test_great_work_completion_triggers_buff() {
        let mut world = World::new();
        let work_id = spawn_great_work(&mut world, "Monument", vec![
            ConstructionCost { item_type: ItemType::Stone, amount: 10 }
        ]);

        // Complete the only phase
        let mut progress = world.get_mut::<ConstructionProgress>(work_id).unwrap();
        progress.current_work = progress.total_work_required;

        let mut schedule = Schedule::default();
        schedule.add_systems(process_great_work_phases);
        schedule.run(&mut world);

        let work = world.get::<GreatWork>(work_id).unwrap();
        assert!(work.is_completed(), "Should be completed.");
        // Verify the component changed state or emitted an event
        assert!(world.get::<OperationalGreatWork>(work_id).is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::construction::{ConstructionProgress, ConstructionCost};

#[derive(Component)]
pub struct GreatWork {
    pub name: String,
    pub current_phase: usize,
    pub phase_costs: Vec<ConstructionCost>,
}

impl GreatWork {
    pub fn is_completed(&self) -> bool {
        self.current_phase >= self.phase_costs.len()
    }
}

#[derive(Component)]
pub struct OperationalGreatWork;

pub fn spawn_great_work(world: &mut World, name: &str, phase_costs: Vec<ConstructionCost>) -> Entity {
    let initial_cost = phase_costs.first().cloned().unwrap_or(ConstructionCost { item_type: crate::layer1::item::ItemType::Wood, amount: 1 });
    world.spawn((
        GreatWork {
            name: name.to_string(),
            current_phase: 0,
            phase_costs,
        },
        ConstructionProgress {
            total_work_required: initial_cost.amount as f32 * 10.0,
            current_work: 0.0,
        }
    )).id()
}

pub fn process_great_work_phases(
    mut commands: Commands,
    mut query: Query<(Entity, &mut GreatWork, &mut ConstructionProgress)>,
) {
    for (entity, mut work, mut progress) in query.iter_mut() {
        if work.is_completed() {
            continue;
        }

        if progress.current_work >= progress.total_work_required {
            work.current_phase += 1;

            if work.is_completed() {
                commands.entity(entity).insert(OperationalGreatWork);
                commands.entity(entity).remove::<ConstructionProgress>();
            } else {
                // Setup next phase
                let next_cost = &work.phase_costs[work.current_phase];
                progress.total_work_required = next_cost.amount as f32 * 10.0;
                progress.current_work = 0.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration with Builders:** Ensure `process_great_work_phases` integrates cleanly with the existing builder AI. Builders need to be able to drop off the specific `ItemType` required for the *current* phase.
- **Visuals:** Add a system that updates the visual sprite or model of the `GreatWork` entity based on its `current_phase`.
- **Morale Aura:** When completed, `OperationalGreatWork` should probably attach an `Aura` component to boost the mood of the entire colony.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for `src/layer1/construction/great_works.rs`.
- [ ] Multi-phase construction logic works correctly.

## 7. Technical Guidance
- Put the logic in `src/layer1/construction/great_works.rs`.
- Add `process_great_work_phases` to `src/layer1/systems/execution.rs` (likely running after generic construction logic).
- Do not implement the specific UI yet; focus purely on the backend state machine.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
