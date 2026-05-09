# 1256: The Architecture of Regret

## Overview

The literal foundations of your empire remember the atrocities committed to build it. If a colony is built on the ruins of a conquered civilization, the new buildings absorb the "psychic resonance" of the dead. As your empire expands, these resonant buildings start generating a new unique resource: "Guilt." Guilt can be harnessed to power incredibly strong psychic weapons or shields, but causes massive, continuous unrest.

## Dependencies

- None

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use crate::layer1::architecture::ruins::{Ruin, RuinHistory};
    use crate::layer1::architecture::building::{BuildingType, MaterialType};
    use crate::layer1::GridPosition;
    use crate::layer3::guilt::{GuiltResource, PsychicResonance, process_guilt_generation_system, apply_guilt_unrest_system};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_building_on_ruins_generates_psychic_resonance() {
        let mut world = World::new();
        // Setup ruin
        world.spawn((
            Ruin {
                original_type: BuildingType::Housing,
                material: MaterialType::Stone,
            },
            GridPosition { x: 5, y: 5 },
            PsychicResonance { intensity: 10.0 }, // Emits resonance
        ));

        // When a building is placed on the ruin, it absorbs the resonance
        let new_building = world.spawn((
            crate::layer1::architecture::building::Building {
                building_type: BuildingType::Smelter,
            },
            GridPosition { x: 5, y: 5 },
        )).id();

        // This is a minimal unit test concept: we verify if the resonance from a ruin
        // influences the building. Real setup would involve the construction system adding PsychicResonance.
    }

    #[test]
    fn test_guilt_resource_generation() {
        let mut world = World::new();
        world.insert_resource(GuiltResource { amount: 0.0 });

        // Setup a resonant building
        world.spawn((
            crate::layer1::architecture::building::Building {
                building_type: BuildingType::Smelter,
            },
            PsychicResonance { intensity: 5.0 },
        ));

        process_guilt_generation_system(&mut world);

        // Assert guilt generated
        assert_eq!(world.resource::<GuiltResource>().amount, 5.0);
    }

    #[test]
    fn test_guilt_causes_unrest() {
        let mut world = World::new();
        world.insert_resource(GuiltResource { amount: 100.0 });

        let pop = world.spawn((
            crate::layer1::social::unrest::Unrest { level: 0.0, modifiers: vec![] },
        )).id();

        apply_guilt_unrest_system(&mut world);

        assert_eq!(world.get::<crate::layer1::social::unrest::Unrest>(pop).unwrap().modifiers[0].value, 10.0);
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::social::unrest::{Unrest, UnrestModifier};

#[derive(Resource, Default)]
pub struct GuiltResource {
    pub amount: f32,
}

#[derive(Component)]
pub struct PsychicResonance {
    pub intensity: f32,
}

pub fn process_guilt_generation_system(world: &mut World) {
    let mut total_resonance = 0.0;

    let mut query = world.query::<&PsychicResonance>();
    for resonance in query.iter(world) {
        total_resonance += resonance.intensity;
    }

    if let Some(mut guilt) = world.get_resource_mut::<GuiltResource>() {
        guilt.amount += total_resonance;
    }
}

pub fn apply_guilt_unrest_system(world: &mut World) {
    let guilt_amount = world.resource::<GuiltResource>().amount;
    if guilt_amount == 0.0 {
        return;
    }

    let unrest_increase = guilt_amount * 0.1; // 10% conversion to unrest

    let mut query = world.query::<&mut Unrest>();
    for mut unrest in query.iter_mut(world) {
        unrest.modifiers.push(crate::layer1::social::unrest::UnrestModifier { value: unrest_increase, duration: 10, label: "Psychic Resonance".to_string() });
    }
}
```

## REFACTOR Phase: Quality & Design

- Introduce an event listener that catches building completion over ruins to correctly attach the `PsychicResonance` component.
- The Unrest modifier needs to cap correctly and work well with other morale systems.
- Add thresholds for Guilt Resource utilization (i.e. spending Guilt to power psychic shields).
- Make sure that pops far away from the resonance don't feel the unrest equally. Distance-based calculation is needed.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified (Guilt generates, Unrest increases)

## Technical Guidance

- Create `src/layer3/guilt.rs` for the guilt layer logic.
- Integrate the Unrest component properly with the `layer1` morale structures.
- Tie the systems into the global update loop in `src/simulation.rs`.

## Questions

*Builder: add questions here if spec is unclear.*
