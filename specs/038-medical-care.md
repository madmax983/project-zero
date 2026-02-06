# 038: Medical Care

## Overview

Colonists can get injured (from starvation, and eventually workplace accidents/combat). This spec introduces the **Hospital** building and the **SeekMedicalCare** action. Injured pops will prioritize seeking a hospital to heal.

This creates a loop: Injury -> Hospital -> Healing -> Return to Work.

## Dependencies

- `034` — Pop Health (Health component, Death system)
- `006` — Building Placement (BuildingType)
- `016` — Utility AI (ActionType)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/medical_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::health::Health;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::medical::{Hospital, healing_system};
    use crate::layer1::execution::AssignmentType;

    #[test]
    fn test_hospital_component_defaults() {
        let hospital = Hospital::default();
        assert!(hospital.healing_rate > 0.0);
    }

    #[test]
    fn test_healing_system_restores_health() {
        let mut world = World::new();

        // Spawn Hospital
        let hospital_entity = world.spawn((
            Building { building_type: BuildingType::Hospital },
            Hospital { healing_rate: 1.0 },
        )).id();

        // Spawn Injured Pop assigned to Hospital (as patient)
        let pop_entity = world.spawn((
            Health { current: 50.0, max: 100.0 },
            crate::layer1::execution::Assignment {
                assignment_type: AssignmentType::Patient,
                target: hospital_entity,
            }
        )).id();

        // Run system
        healing_system(&mut world);

        // Check health
        let health = world.get::<Health>(pop_entity).unwrap();
        assert!((health.current - 51.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_healing_stops_at_max() {
        let mut world = World::new();

        let hospital_entity = world.spawn((
            Building { building_type: BuildingType::Hospital },
            Hospital { healing_rate: 10.0 },
        )).id();

        let pop_entity = world.spawn((
            Health { current: 95.0, max: 100.0 },
            crate::layer1::execution::Assignment {
                assignment_type: AssignmentType::Patient,
                target: hospital_entity,
            }
        )).id();

        healing_system(&mut world);

        let health = world.get::<Health>(pop_entity).unwrap();
        assert!((health.current - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_assignment_type_patient_exists() {
        // This test ensures the enum variant is added
        let assignment = AssignmentType::Patient;
        // Just verify it compiles and matches
        assert!(matches!(assignment, AssignmentType::Patient));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `BuildingType`

Add `Hospital` to `src/layer1/building.rs`.

```rust
pub enum BuildingType {
    // ...
    Hospital,
}

impl BuildingType {
    pub const fn char(&self) -> char {
        match self {
            Self::Hospital => '+',
            // ...
        }
    }

    pub const fn cost(&self) -> ColonyResources {
        match self {
            Self::Hospital => ColonyResources {
                wood: 40.0,
                stone: 10.0,
                ..ColonyResources::zeroed()
            },
            // ...
        }
    }
}
```

### 2. Create `Hospital` Component

```rust
// src/layer1/medical.rs

use bevy_ecs::prelude::*;
use crate::layer1::execution::{Assignment, AssignmentType};
use crate::layer1::health::Health;

#[derive(Component)]
pub struct Hospital {
    pub healing_rate: f32,
}

impl Default for Hospital {
    fn default() -> Self {
        Self { healing_rate: 0.5 } // 0.5 HP per tick
    }
}

pub fn healing_system(world: &mut World) {
    // Iterate over pops that are Patients
    let mut query = world.query::<(&mut Health, &Assignment)>();

    // Collect updates to avoid double borrow if we needed to look up hospital stats
    // But for MVP we can assume standard rate or just check if assignment target has Hospital component?
    // Bevy query: we need to verify the target is a valid hospital to apply healing?
    // Or we assume utility AI only assigns valid targets.
    // For safety, let's look up the hospital.

    // We can't easily query nested in one pass with mutable access.
    // So we'll fetch hospital rates first if variable, or just use a constant/component lookup.

    // Alternative:
    // query <(Entity, &mut Health, &Assignment)>
    // For each, if assignment is Patient, get target, get Hospital component, apply heal.

    // Since we need random access to Hospital components, it's tricky in a single system without disjoint queries.
    // However, Hospital component is read-only here.

    // Correct Bevy 0.15 pattern:
    // let hospital_query = world.query::<&Hospital>();
    // This is hard with `world` passed mutably.

    // SIMPLIFICATION for MVP: Use a fixed healing rate constant or rely on the `Hospital` component being present on the target.
    // Let's implement the look-up properly using `world.get::<Hospital>(target)`.

    let mut updates: Vec<(Entity, f32)> = Vec::new();

    {
        let mut query = world.query::<(Entity, &Health, &Assignment)>();
        for (entity, health, assignment) in query.iter(world) {
            if assignment.assignment_type == AssignmentType::Patient {
                if let Some(hospital) = world.get::<Hospital>(assignment.target) {
                     if health.current < health.max {
                         updates.push((entity, hospital.healing_rate));
                     }
                }
            }
        }
    }

    for (entity, amount) in updates {
        if let Some(mut health) = world.get_mut::<Health>(entity) {
            health.current = (health.current + amount).min(health.max);
        }
    }
}
```

### 3. Update `AssignmentType`

In `src/layer1/execution.rs`:

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AssignmentType {
    // ...
    Patient,
}
```

### 4. Update `ActionType` and Utility AI

In `src/layer1/utility_ai.rs`:

```rust
pub enum ActionType {
    // ...
    SeekMedicalCare,
}

// In evaluate_actions:
// If health < 80% (and not starving? or priority?), score SeekMedicalCare.
// Score should increase as health decreases.
// score = (1.0 - health_pct) * 2.0; // High priority when low health
```

### 5. Add Arrival Logic

In `src/layer1/execution.rs` `arrival_handler_system`:
Handle `SeekMedicalCare` -> Find nearest `Hospital` -> Assign `AssignmentType::Patient`.

## REFACTOR Phase: Quality & Design

- **Triage**: If multiple hospitals, pick the one with space? (MVP: unlimited capacity or overlap).
- **Doctors**: Future spec. For now, hospitals heal automatically (magical med-beds).
- **Cost**: Healing could consume "Medicine" resource (future).
- **Log**: Log when a patient is fully healed.

## Acceptance Criteria

- [ ] `Hospital` component exists and `BuildingType::Hospital` is buildable.
- [ ] `AssignmentType::Patient` exists.
- [ ] `ActionType::SeekMedicalCare` is evaluated by AI.
- [ ] Injured pops go to the hospital.
- [ ] Pops in the hospital regenerate health.
- [ ] Tests pass.

## Technical Guidance

- Ensure `UtilityWeights` (utility_ai.rs) array size is updated for new `ActionType` count if it's fixed size.
- Ensure `arrival_handler_system` handles the `SeekMedicalCare` case correctly (finding a `BuildingType::Hospital`).

## Questions

*Builder: add questions here if spec is unclear.*
