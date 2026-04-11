# 098: Medical Triage Policies

## Overview

As the colony grows and resources (or hospital beds) become scarce, the player needs control over who gets medical treatment. This spec introduces a `MedicalPolicy` resource that dictates prioritization logic within the `healing_system`.

## Dependencies

- `038` — Medical Care (Hospital building and healing system)
- `009` — Job System (for `WorkersFirst` policy)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/medical_triage_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::health::Health;
    use crate::layer1::medical::{Hospital, healing_system, MedicalPolicy};
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use crate::layer1::pop::{Pop, PopState}; // Assuming Pop/PopState exists
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::execution::WorkState; // Or however employment is tracked

    #[test]
    fn test_policy_resource_default() {
        let mut world = World::new();
        world.init_resource::<MedicalPolicy>();
        assert_eq!(*world.resource::<MedicalPolicy>(), MedicalPolicy::SaveEveryone);
    }

    #[test]
    fn test_workers_first_policy_ignores_unemployed() {
        let mut world = World::new();
        world.insert_resource(MedicalPolicy::WorkersFirst);

        let hospital = world.spawn((
            Building { building_type: BuildingType::Hospital },
            Hospital { healing_rate: 10.0 },
        )).id();

        // Employed Pop (Assigned to something else as a worker? No, pop employment is usually distinct)
        // For this test, let's assume employment is checked via having a job assignment or specific component.
        // In current codebase, jobs are assignments. But a pop is also assigned as a Patient.
        // So a pop cannot be Working AND Patient simultaneously in the Assignment slot?
        // Wait, Assignment is a component. A pop has ONE assignment.
        // If they are assigned as Patient, they are NOT assigned as Worker.
        // So we need another way to track "Employment Status".
        // Use `Skills` or `Job` component if it persists?
        // Or maybe check if they *had* a job?
        // Let's assume for this MVP that "WorkersFirst" checks if the pop has a specific `Job` component that persists,
        // OR we just add a `Job` component to employed pops.

        // Let's assume we add a marker component `Employed` for this test context,
        // or check `Pop` struct fields if they exist.
        // Checking `src/layer1/pop.rs` might be needed.
        // For now, let's define the test expectation:
        // "WorkersFirst" implies we filter based on some criteria.
        // Let's use `PopState` if it reflects employment, or just a mock component `IsWorker`.

        let worker = world.spawn((
            Health { current: 50.0, max: 100.0 },
            AssignedTo { assignment_type: AssignmentType::Patient, entity: hospital },
            // Marker for employment (Implementation detail: needs to be defined)
            crate::layer1::pop::Employed,
        )).id();

        let idler = world.spawn((
            Health { current: 50.0, max: 100.0 },
            AssignedTo { assignment_type: AssignmentType::Patient, entity: hospital },
            // No Employed component
        )).id();

        healing_system(&mut world);

        let worker_health = world.get::<Health>(worker).unwrap();
        let idler_health = world.get::<Health>(idler).unwrap();

        assert!(worker_health.current > 50.0, "Worker should be healed");
        assert!((idler_health.current - 50.0).abs() < f32::EPSILON, "Idler should NOT be healed under WorkersFirst");
    }

    #[test]
    fn test_triage_policy_prioritizes_lowest_health() {
        // This test requires a capacity limit to be meaningful,
        // OR we simulate limited healing points (e.g. Hospital has a "healing pool" per tick).
        // Let's implement a 'healing pool' concept in the Hospital for this spec,
        // or just sort the order of processing if we add a limit later.
        // For this MVP, let's say Hospital has a `max_patients_per_tick` or `total_healing_output`.
        // Let's update Hospital to have `healing_capacity: f32` (total HP it can dispense per tick).

        let mut world = World::new();
        world.insert_resource(MedicalPolicy::Triage);

        // Hospital with limited output (e.g., 10 HP total per tick)
        let hospital = world.spawn((
            Building { building_type: BuildingType::Hospital },
            Hospital { healing_rate: 10.0, max_healing_per_tick: 15.0 }, // Can heal 1.5 people fully, or split?
        )).id();

        // Critical Patient (10/100)
        let critical = world.spawn((
            Health { current: 10.0, max: 100.0 },
            AssignedTo { assignment_type: AssignmentType::Patient, entity: hospital },
        )).id();

        // Stable Patient (90/100)
        let stable = world.spawn((
            Health { current: 90.0, max: 100.0 },
            AssignedTo { assignment_type: AssignmentType::Patient, entity: hospital },
        )).id();

        // If max_healing_per_tick is 15, and rate is 10:
        // Critical needs 90, gets 10. (Cost 10). Remaining capacity 5.
        // Stable needs 10, gets 5? Or gets 0 if Critical took priority?

        // Let's assume Triage sorts by HP% ascending.

        healing_system(&mut world);

        // Critical should have received full rate (10.0) -> 20.0
        // Stable should have received partial/none?
        // If implementation consumes capacity:
        // Critical consumes 10. Capacity left 5.
        // Stable consumes 5. -> 95.0.

        let crit_health = world.get::<Health>(critical).unwrap();
        let stable_health = world.get::<Health>(stable).unwrap();

        assert_eq!(crit_health.current, 20.0);
        assert_eq!(stable_health.current, 95.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `MedicalPolicy` Resource

In `src/layer1/medical.rs`:

```rust
#[derive(Resource, Default, Debug, PartialEq, Eq, Copy, Clone)]
pub enum MedicalPolicy {
    #[default]
    SaveEveryone,
    WorkersFirst,
    Triage,
}
```

### 2. Update `Hospital` Component

Add a capacity limit to simulate scarcity (forcing triage).

```rust
#[derive(Component)]
pub struct Hospital {
    pub healing_rate: f32,
    pub max_healing_per_tick: f32, // New field
}

impl Default for Hospital {
    fn default() -> Self {
        Self {
            healing_rate: 0.5,
            max_healing_per_tick: 5.0, // Default limit (e.g. 10 patients)
        }
    }
}
```

### 3. Update `healing_system`

```rust
pub fn healing_system(world: &mut World) {
    let policy = world.get_resource::<MedicalPolicy>().copied().unwrap_or_default();

    // 1. Collect all patients per hospital
    let mut hospital_patients: HashMap<Entity, Vec<(Entity, f32, bool)>> = HashMap::new();
    // Key: Hospital Entity, Value: List of (Patient Entity, CurrentHP/MaxHP, IsEmployed)

    // ... Query logic to populate map ...
    // Note: Need to check for `Employed` component or similar for WorkersFirst.

    // 2. Process each hospital
    for (hospital_ent, mut patients) in hospital_patients {
        let hospital = world.get::<Hospital>(hospital_ent).unwrap();
        let mut capacity = hospital.max_healing_per_tick;

        // 3. Apply Policy Sort/Filter
        match policy {
            MedicalPolicy::WorkersFirst => {
                patients.retain(|(_, _, employed)| *employed);
            },
            MedicalPolicy::Triage => {
                // Sort by Health % (Ascending)
                patients.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            },
            MedicalPolicy::SaveEveryone => {
                // FIFO or random (no sort needed)
            }
        }

        // 4. Distribute Healing
        for (patient, _, _) in patients {
            if capacity <= 0.0 { break; }

            let amount = hospital.healing_rate.min(capacity);
            // Apply update...
            capacity -= amount;
        }
    }
}
```

### 4. Create `Employed` Component (if missing)

If `src/layer1/pop.rs` doesn't have an employment tracker that persists during illness, create a marker `Employed` that is added/removed when jobs are assigned.

## REFACTOR Phase: Quality & Design

- **UI Integration**: Add a UI panel to toggle `MedicalPolicy`.
- **Feedback**: Log when a patient is denied care due to policy.
- **Complexity**: `max_healing_per_tick` abstracts "beds" and "medicine" into one number. Future specs can separate these.

## Acceptance Criteria

- [ ] `MedicalPolicy` resource exists.
- [ ] `Hospital` has `max_healing_per_tick`.
- [ ] `healing_system` respects `WorkersFirst` (skips unemployed).
- [ ] `healing_system` respects `Triage` (prioritizes low health).
- [ ] Default behavior (`SaveEveryone`) works as before (but capped).
- [ ] Tests pass.

## Questions
- *Builder: Does a patient lose their job assignment? If yes, `WorkersFirst` breaks.*
  *Architect:* No, pops retain their `Job` component while hospitalized, they just have their current action overridden to `Rest`.
