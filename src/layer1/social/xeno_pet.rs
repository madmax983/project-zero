use crate::layer1::economy::resources::{ColonyResources, ResourceType};
use crate::layer1::social::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct XenoPetOwner;

#[derive(Component)]
pub struct XenoPet {
    pub hunger: f32,
    pub reproduction_progress: f32,
}

#[derive(Component)]
pub struct ProtectedPet;

pub fn apply_xeno_pet_morale(mut query: Query<&mut Morale, With<XenoPetOwner>>) {
    for mut morale in query.iter_mut() {
        let label = "Xeno-Pet".to_string();

        // Remove old modifier if exists so we don't leak memory
        morale.modifiers.retain(|m| m.label != label);

        morale.add_modifier(MoodModifier {
            label,
            value: 10.0, // Large morale boost for the pet
            duration: 2, // Lasts a bit so it won't decay immediately
        });
    }
}

pub fn xeno_pet_reproduction_system(
    mut commands: Commands,
    mut resources: ResMut<ColonyResources>,
    mut query: Query<&mut XenoPet>,
) {
    for mut pet in query.iter_mut() {
        if pet.reproduction_progress >= 90.0 && resources.try_consume(ResourceType::Food, 10.0) {
            pet.reproduction_progress = 0.0;
            // Spawn new pet that is also protected from butchering
            commands.spawn((
                XenoPet {
                    hunger: 0.0,
                    reproduction_progress: 0.0,
                },
                ProtectedPet,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::resources::ColonyResources;
    use crate::layer1::social::morale::Morale;

    #[test]
    fn test_xeno_pet_increases_morale() {
        let mut world = World::new();
        let entity = world
            .spawn(Morale {
                value: 0.5,
                modifiers: vec![],
            })
            .id();

        world.entity_mut(entity).insert(XenoPetOwner);

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_xeno_pet_morale);

        // Include the actual cache update system to test the real value
        use crate::layer1::needs::Needs;
        use crate::layer1::social::morale::update_morale_cache_system;

        world.entity_mut(entity).insert(Needs::default());

        schedule.add_systems(update_morale_cache_system.after(apply_xeno_pet_morale));

        schedule.run(&mut world);

        let morale = world.get::<Morale>(entity).unwrap();
        assert!(
            morale.value > 0.5,
            "Morale should increase when owning a Xeno-Pet"
        );
        assert_eq!(morale.modifiers[0].label, "Xeno-Pet");
    }

    #[test]
    fn test_xeno_pet_reproduction_consumes_food() {
        let mut world = World::new();
        let mut resources = ColonyResources::zeroed();
        resources.food = 100.0;
        world.insert_resource(resources);
        world.spawn(XenoPet {
            hunger: 0.0,
            reproduction_progress: 90.0,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(xeno_pet_reproduction_system);
        schedule.run(&mut world);

        let resources = world.get_resource::<ColonyResources>().unwrap();
        assert!(
            resources.food < 100.0,
            "Food should be consumed during reproduction"
        );

        let mut query = world.query::<&XenoPet>();
        let count = query.iter(&world).count();
        assert_eq!(count, 2, "A new Xeno-Pet should have been spawned");

        let mut protected_query = world.query::<&ProtectedPet>();
        let protected_count = protected_query.iter(&world).count();
        assert_eq!(
            protected_count, 1,
            "The newly spawned pet should be protected"
        );
    }
}
