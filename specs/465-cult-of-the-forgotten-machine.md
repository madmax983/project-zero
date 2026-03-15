# 465 - Cult of the Forgotten Machine

## 1. Overview
When highly complex machinery (such as advanced Layer 1 power cores or production facilities) goes unmaintained for an extended period, it doesn't simply break down. Instead, it develops unique "Quirks." Over time, if these Quirks are not addressed, a faction of Pops may begin to revere the machine, believing the Quirks are divine mandates. This "Cult of the Forgotten Machine" will actively sabotage repair efforts, hoard resources to appease the machine, and can cause significant unrest if suppressed.

This feature introduces `MachineQuirk` to aging, unmaintained buildings, a `Cultist` component for affected Pops, and an event structure to track the formation and actions of the cult, utilizing the `CIVILIZATION_FALL` or related narrative structures if the cult triggers broader collapse, or generating distinct lore events.

## 2. Dependencies
- **Maintenance Debt (Spec 112):** Provides the baseline for tracking unmaintained buildings.
- **Pop Factions (Spec 068):** Provides the foundation for group behaviors and unrest.
- **Job System (Spec 009):** Needed for repair job generation and subsequent sabotage.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    // Assume necessary component definitions exist
    #[derive(Component, Default)]
    struct MaintenanceDebt { amount: f32 }

    #[derive(Component)]
    struct MachineQuirk;

    #[derive(Component)]
    struct Cultist {
        target_machine: Entity,
    }

    #[derive(Component)]
    struct RepairJob {
        target: Entity,
    }

    #[derive(Event)]
    struct SabotageEvent {
        saboteur: Entity,
        job: Entity,
    }

    fn spawn_test_machine(world: &mut World, debt: f32) -> Entity {
        world.spawn(MaintenanceDebt { amount: debt }).id()
    }

    fn spawn_test_pop(world: &mut World) -> Entity {
        world.spawn_empty().id()
    }

    #[test]
    fn test_machine_develops_quirk_from_high_debt() {
        let mut world = World::new();
        let machine = spawn_test_machine(&mut world, 150.0); // High debt

        // Run the system that generates quirks
        // world.run_system_once(generate_machine_quirks_system).unwrap();

        // Assert the machine now has a quirk
        // assert!(world.entity(machine).contains::<MachineQuirk>());
    }

    #[test]
    fn test_pops_become_cultists_near_quirky_machine() {
        let mut world = World::new();
        let machine = spawn_test_machine(&mut world, 150.0);
        world.entity_mut(machine).insert(MachineQuirk);

        let pop = spawn_test_pop(&mut world);
        // Add logic to position pop near machine if spatial queries are used

        // Run cult formation system
        // world.run_system_once(form_machine_cult_system).unwrap();

        // Assert the pop is now a cultist targeting the machine
        // assert!(world.entity(pop).contains::<Cultist>());
        // assert_eq!(world.get::<Cultist>(pop).unwrap().target_machine, machine);
    }

    #[test]
    fn test_cultists_sabotage_repair_jobs() {
        let mut world = World::new();
        let machine = spawn_test_machine(&mut world, 150.0);
        world.entity_mut(machine).insert(MachineQuirk);

        let cultist = spawn_test_pop(&mut world);
        world.entity_mut(cultist).insert(Cultist { target_machine: machine });

        let repair_job = world.spawn(RepairJob { target: machine }).id();

        // Run sabotage system
        // world.run_system_once(cult_sabotage_system).unwrap();

        // Assert the repair job was sabotaged (e.g., despawned or marked invalid)
        // assert!(world.get_entity(repair_job).is_err() || world.get::<RepairJob>(repair_job).is_none());
        // Alternatively, check for SabotageEvent emission
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Basic component structures
use bevy_ecs::prelude::*;

#[derive(Component, Default)]
pub struct MaintenanceDebt {
    pub amount: f32,
}

#[derive(Component)]
pub struct MachineQuirk;

#[derive(Component)]
pub struct Cultist {
    pub target_machine: Entity,
}

#[derive(Component)]
pub struct RepairJob {
    pub target: Entity,
}

#[derive(Event)]
pub struct SabotageEvent {
    pub saboteur: Entity,
    pub job: Entity,
}

// Minimal systems to make tests pass

pub fn generate_machine_quirks_system(
    mut commands: Commands,
    query: Query<(Entity, &MaintenanceDebt), Without<MachineQuirk>>,
) {
    for (entity, debt) in query.iter() {
        if debt.amount > 100.0 {
            commands.entity(entity).insert(MachineQuirk);
        }
    }
}

pub fn form_machine_cult_system(
    mut commands: Commands,
    quirky_machines: Query<Entity, With<MachineQuirk>>,
    pops: Query<Entity, Without<Cultist>>,
) {
    // Highly simplified: just grab the first machine and convert all pops
    if let Some(machine) = quirky_machines.iter().next() {
        for pop in pops.iter() {
            commands.entity(pop).insert(Cultist { target_machine: machine });
        }
    }
}

pub fn cult_sabotage_system(
    mut commands: Commands,
    cultists: Query<(Entity, &Cultist)>,
    repair_jobs: Query<(Entity, &RepairJob)>,
    mut sabotage_events: EventWriter<SabotageEvent>,
) {
    for (saboteur, cultist) in cultists.iter() {
        for (job_entity, job) in repair_jobs.iter() {
            if job.target == cultist.target_machine {
                // Sabotage!
                commands.entity(job_entity).despawn();
                sabotage_events.send(SabotageEvent {
                    saboteur,
                    job: job_entity,
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Spatial/Radius Checking:** The `form_machine_cult_system` currently converts everyone globally. This must be refactored to only affect Pops within a certain radius or those who interact with the machine.
- **Probability/Resistance:** Not all Pops should instantly convert. Introduce a resistance check based on Morale, Traits (e.g., "Rational", "Superstitious"), or Education.
- **Sabotage Nuance:** Instead of instantly despawning repair jobs globally, cultists should need to physically move to the job site or interact with the assigned worker to commit sabotage.
- **Resource Hoarding:** Add logic for Cultists to take resources from stockpiles and "offer" them to the machine, effectively destroying or locking the resources.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code module.
- [ ] Machines with high `MaintenanceDebt` correctly acquire `MachineQuirk`.
- [ ] Pops in proximity/interacting with quirky machines have a chance to gain `Cultist`.
- [ ] `Cultist` Pops actively attempt to disrupt `RepairJob` entities targeting their revered machine.

## 7. Technical Guidance
- Integrate with existing Pathfinding and Job Execution systems to make sabotage a physical action rather than a global instantaneous event.
- Use `lore/TEMPLATES.md` to trigger chronicle events when a cult forms or successfully sabotages a critical repair.
- Ensure the `MachineQuirk` component alters the machine's behavior (e.g., fluctuating power output, strange noises like "the hum" or "the song" referenced in `lore/LEXICON.md`) to justify the cult's belief.

## 8. Questions
*Builder: add questions here if spec is unclear.*
