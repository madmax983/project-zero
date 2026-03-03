# 151: Cybernetic Augmentation

## Overview

"We can rebuild him. We have the technology."

This spec introduces the ability to augment Pops with cybernetic prosthetics. These are items crafted or found that can be surgically installed into a Pop. Augmentations provide significant bonuses to physical stats (Work Speed, Movement Speed, Combat) but come with social penalties and potential maintenance costs.

This feature lays the groundwork for transhumanism, "The Ship of Theseus" scenarios, and specialized castes of workers.

## Dependencies

- `003` — Pop Entity (Speed, Skills)
- `038` — Medical Care (Hospital structure)
- `030` — Tool Economy (Item system)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/cybernetics_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Speed, Skills};
    use crate::layer1::items::{Item, ItemType};
    use crate::layer1::cybernetics::{Prosthetic, Augmentations, ProstheticType, surgery_system};
    use crate::layer1::medical::Hospital;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::actions::{AssignedTo, AssignmentType};

    #[test]
    fn test_prosthetic_component_defaults() {
        let prosthetic = Prosthetic {
            prosthetic_type: ProstheticType::BionicArm,
            efficiency_bonus: 0.2,
            social_penalty: 0.1,
            power_consumption: 1.0,
        };
        assert_eq!(prosthetic.prosthetic_type, ProstheticType::BionicArm);
    }

    #[test]
    fn test_augmentations_component_exists() {
        let mut world = World::new();
        let entity = world.spawn((Pop, Augmentations::default())).id();
        let augs = world.get::<Augmentations>(entity).unwrap();
        assert!(augs.installed.is_empty());
    }

    #[test]
    fn test_surgery_installs_prosthetic() {
        let mut world = World::new();

        // Spawn Hospital
        let hospital = world.spawn((
            Building { building_type: BuildingType::Hospital },
            Hospital::default(),
            GridPosition { x: 0, y: 0 },
            // Inventory component? For MVP, let's assume the item is "available"
            // or the pop is holding it.
            // Let's spawn the item entity first.
        )).id();

        // Spawn Prosthetic Item
        let bionic_arm = world.spawn((
            Item,
            Prosthetic {
                prosthetic_type: ProstheticType::BionicArm,
                efficiency_bonus: 0.5,
                social_penalty: 0.1,
                power_consumption: 0.0,
            }
        )).id();

        // Spawn Pop assigned to Surgery with the item as the "target_item" context?
        // Or simply the hospital has a "SurgeryQueue" component.
        // For MVP: Let's assume the Pop is carrying the item (conceptually) or it's linked via Assignment.

        // Let's extend Assignment or use a new component for SurgeryContext.

        let pop = world.spawn((
            Pop,
            Augmentations::default(),
            AssignedTo {
                assignment_type: AssignmentType::Surgery,
                entity: hospital,
            },
            // The item to install
            crate::layer1::cybernetics::PendingSurgery {
                item: bionic_arm,
                progress: 0.0,
                duration: 10.0,
            }
        )).id();

        // Run surgery system multiple times to complete progress
        for _ in 0..11 {
            surgery_system(&mut world);
        }

        // Verify installation
        let augs = world.get::<Augmentations>(pop).unwrap();
        assert!(augs.installed.contains(&bionic_arm));

        // Verify PendingSurgery component is removed
        assert!(world.get::<crate::layer1::cybernetics::PendingSurgery>(pop).is_none());
    }

    #[test]
    fn test_augmentation_affects_stats() {
        // This requires a system that recalculates stats based on augmentations.
        // e.g. `recalculate_stats_system` or hooking into existing systems.
        // Let's test a helper function `calculate_efficiency_bonus`.

        let mut world = World::new();

        let bionic_arm = world.spawn(Prosthetic {
            prosthetic_type: ProstheticType::BionicArm,
            efficiency_bonus: 0.5,
            social_penalty: 0.1,
            power_consumption: 0.0,
        }).id();

        let pop = world.spawn((
            Pop,
            Augmentations {
                installed: vec![bionic_arm],
            },
            Speed::default(),
        )).id();

        // Helper function to get total bonus
        let bonus = crate::layer1::cybernetics::get_efficiency_bonus(&world, pop);
        assert!((bonus - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_augmentation_social_penalty() {
        let mut world = World::new();
        let bionic_face = world.spawn(Prosthetic {
            prosthetic_type: ProstheticType::SyntheticSkin, // Or something obvious
            efficiency_bonus: 0.0,
            social_penalty: 0.2,
            power_consumption: 0.0,
        }).id();

        let pop = world.spawn((
            Pop,
            Augmentations { installed: vec![bionic_face] },
        )).id();

        let penalty = crate::layer1::cybernetics::get_social_penalty(&world, pop);
        assert!((penalty - 0.2).abs() < f32::EPSILON);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `Prosthetic` and `Augmentations`

Create `src/layer1/cybernetics.rs`.

```rust
use bevy_ecs::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProstheticType {
    BionicArm,
    BionicLeg,
    BionicEye,
    SyntheticSkin,
    NeuralInterface,
}

#[derive(Component, Debug, Clone)]
pub struct Prosthetic {
    pub prosthetic_type: ProstheticType,
    /// Multiplier for work speed (e.g. 0.2 = +20%)
    pub efficiency_bonus: f32,
    /// Malus for social interactions (e.g. 0.1 = -10% relationship gain)
    pub social_penalty: f32,
    /// Power consumed per tick (future use)
    pub power_consumption: f32,
}

#[derive(Component, Debug, Default, Clone)]
pub struct Augmentations {
    /// List of installed prosthetic entities.
    pub installed: Vec<Entity>,
}

#[derive(Component, Debug, Clone)]
pub struct PendingSurgery {
    pub item: Entity,
    pub progress: f32,
    pub duration: f32,
}
```

### 2. Implement `surgery_system`

```rust
use crate::layer1::actions::{AssignedTo, AssignmentType};

pub fn surgery_system(world: &mut World) {
    let mut completed_surgeries = Vec::new();

    // Query for pops undergoing surgery
    let mut query = world.query::<(Entity, &mut PendingSurgery, &AssignedTo)>();

    // Check assignments
    for (entity, mut surgery, assignment) in query.iter_mut(world) {
        if assignment.assignment_type == AssignmentType::Surgery {
            // Check if at hospital (assignment.entity)?
            // For MVP, assume assignment implies presence or adjacent.

            // Advance progress (1.0 per tick? Or skill based?)
            surgery.progress += 1.0;

            if surgery.progress >= surgery.duration {
                completed_surgeries.push((entity, surgery.item));
            }
        }
    }

    // Apply results
    for (pop_entity, item_entity) in completed_surgeries {
        // Move item to installed list
        if let Some(mut augs) = world.get_mut::<Augmentations>(pop_entity) {
            augs.installed.push(item_entity);
        }

        // Remove PendingSurgery component
        world.entity_mut(pop_entity).remove::<PendingSurgery>();

        // Change assignment back to Idle or Recovery?
        // world.entity_mut(pop_entity).remove::<AssignedTo>(); // Or set to Idle
    }
}
```

### 3. Implement Helper Functions

```rust
pub fn get_efficiency_bonus(world: &World, pop: Entity) -> f32 {
    let mut total = 0.0;
    if let Some(augs) = world.get::<Augmentations>(pop) {
        for &item in &augs.installed {
            if let Some(prosthetic) = world.get::<Prosthetic>(item) {
                total += prosthetic.efficiency_bonus;
            }
        }
    }
    total
}

pub fn get_social_penalty(world: &World, pop: Entity) -> f32 {
    let mut total = 0.0;
    if let Some(augs) = world.get::<Augmentations>(pop) {
        for &item in &augs.installed {
            if let Some(prosthetic) = world.get::<Prosthetic>(item) {
                total += prosthetic.social_penalty;
            }
        }
    }
    total
}
```

### 4. Integration Points

- Update `ActionType` (utility_ai.rs) to include `Surgery` (passive action like `SeekMedicalCare`).
- Update `AssignmentType` (execution.rs) to include `Surgery`.
- In `work_execution_system` (or wherever speed is used), query `Augmentations` and add `get_efficiency_bonus` to the base speed.

## REFACTOR Phase: Quality & Design

- **Power Integration**: Cyborgs should consume Power if `power_consumption > 0`. This replaces or supplements Food needs.
- **Recovery**: Surgery should leave the Pop with low Health or a "Recovering" trait/state requiring bed rest.
- **Item Consumption**: Ensure the item is removed from any previous inventory (Stockpile) when surgery starts.
- **UI**: Display installed augmentations in the Pop Inspector.

## Acceptance Criteria

- [ ] `Prosthetic` items can be spawned.
- [ ] `Augmentations` component tracks installed items.
- [ ] `surgery_system` successfully moves an item from `PendingSurgery` to `Augmentations`.
- [ ] Stat helper functions return correct summed values.
- [ ] Integration with `AssignmentType::Surgery`.

## Technical Guidance

- Use `PendingSurgery` as a transient component to track the "casting time" of the operation.
- Don't destroy the Item entity; keep it alive but "owned" by the Pop's `Augmentations` component. This allows un-installing later or tracking specific item history ("Heirloom Items").

## Questions

*Builder: Should surgery require a Doctor pop to be present?*
*Architect: Yes, applying augmentations requires a pop with the Medical skill to perform the action.*
*Architect: Yes, applying augmentations requires a pop with the Medical skill to perform the action.*
*Architect: For MVP, no. The Hospital building provides the "service". Future updates can require a Doctor job.*
