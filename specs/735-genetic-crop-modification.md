# 735 - Genetic Crop Modification

## 1. Overview
Improving on nature with unintended consequences. The lab can splice traits into crops, such as "Luminescent" for light, "Hardy" for cold resist, or "Nutrient-Dense". The risk involves unstable mutations causing "Aggressive Growth" or "Toxic Spores". The tension balances food security against bio-safety.

## 2. Dependencies
- Farming system.
- Traits and modifiers for crops.
- Event system for mutations.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_crop_modification_increases_yield() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let crop = app.world_mut().spawn((
            Crop { base_yield: 10, current_yield: 10 },
            GeneticallyModified { traits: vec![CropTrait::NutrientDense] },
        )).id();

        // Act
        // Apply modifications
        app.update();

        // Assert
        let c = app.world().get::<Crop>(crop).unwrap();
        assert!(c.current_yield > 10);
    }

    #[test]
    fn test_crop_mutation_triggers_event() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // High instability
        app.world_mut().spawn((
            Crop { base_yield: 10, current_yield: 10 },
            GeneticallyModified { traits: vec![CropTrait::NutrientDense] },
            GeneticInstability { risk_factor: 1.0 }, // Guaranteed mutation
        ));

        app.world_mut().init_resource::<Events<CropMutationEvent>>();

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<CropMutationEvent>>();
        // Event should be emitted
        assert!(!events.is_empty());
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct Crop {
    pub base_yield: u32,
    pub current_yield: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CropTrait {
    Luminescent,
    Hardy,
    NutrientDense,
}

#[derive(Component)]
pub struct GeneticallyModified {
    pub traits: Vec<CropTrait>,
}

#[derive(Component)]
pub struct GeneticInstability {
    pub risk_factor: f32,
}

#[derive(Event)]
pub struct CropMutationEvent {
    pub crop_entity: Entity,
    pub mutation_type: MutationType,
}

pub enum MutationType {
    AggressiveGrowth,
    ToxicSpores,
}

pub fn apply_crop_traits(
    mut crops: Query<(&mut Crop, &GeneticallyModified), Changed<GeneticallyModified>>,
) {
    for (mut crop, modifier) in crops.iter_mut() {
        if modifier.traits.contains(&CropTrait::NutrientDense) {
            crop.current_yield = crop.base_yield * 2;
        }
    }
}

pub fn process_mutations(
    crops: Query<(Entity, &GeneticInstability)>,
    mut events: EventWriter<CropMutationEvent>,
) {
    let mut rng = rand::thread_rng();
    for (entity, instability) in crops.iter() {
        if rng.gen::<f32>() < instability.risk_factor {
            events.send(CropMutationEvent {
                crop_entity: entity,
                mutation_type: MutationType::AggressiveGrowth, // Simplified
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Separate the "application" of stats from the core update loop to prevent re-applying multipliers.
- Inject RNG for testability.
- Provide a robust way to visualize crops that have aggressive growth or toxic spores.

## 6. Acceptance Criteria (Testable!)
- [ ] Crop traits correctly modify base crop behavior (yield, light, cold resist).
- [ ] Instability causes mutation events to trigger.
- [ ] Coverage >= 85%, 0 test failures, `clippy -D warnings` passes.

## 7. Technical Guidance
- `CropTrait` should ideally be generic, loaded from a config if the project has moved towards data-driven designs.
- Allow researchers to spend `Science` or `Biomass` to lower the `risk_factor` of modified crops before planting.

## 8. Questions
*Builder: add questions here if spec is unclear.*
