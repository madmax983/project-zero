use bevy_ecs::prelude::*;
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_crop_modification_increases_yield() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, apply_crop_traits);

        let crop = app
            .world_mut()
            .spawn((
                Crop {
                    base_yield: 10,
                    current_yield: 10,
                },
                GeneticallyModified {
                    traits: vec![CropTrait::NutrientDense],
                },
            ))
            .id();

        // Act
        // Apply modifications
        app.update();

        // Assert
        let c = app.world().get::<Crop>(crop).expect("Component should exist or System should run");
        assert!(c.current_yield > 10);
    }

    #[test]
    fn test_crop_mutation_triggers_event() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, process_mutations);

        // High instability
        app.world_mut().spawn((
            Crop {
                base_yield: 10,
                current_yield: 10,
            },
            GeneticallyModified {
                traits: vec![CropTrait::NutrientDense],
            },
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
