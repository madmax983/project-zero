use bevy::prelude::*;
use crate::layer1::entities::pop::Pop;
use crate::layer1::social::morale::{MoodModifier, Morale};
use crate::layer1::economy::resources::ColonyResources;
use rand::Rng;

#[derive(Component)]
pub struct XenofloraPet {
    pub resource_upkeep: u32,
    pub mood_boost: f32,
}

pub fn apply_pet_mood_boost(mut query: Query<(&mut Morale, &XenofloraPet)>) {
    for (mut morale, pet) in query.iter_mut() {
        if !morale.modifiers.iter().any(|m| m.label == "Xenoflora Pet") {
            morale.add_modifier(MoodModifier {
                label: "Xenoflora Pet".to_string(),
                value: pet.mood_boost,
                duration: 1, // Renewed every tick as long as they have the pet
            });
        }
    }
}

pub fn pet_upkeep_system(
    mut query: Query<(&XenofloraPet, &mut Morale)>,
    mut resources: ResMut<ColonyResources>,
) {
    for (pet, mut morale) in query.iter_mut() {
        let upkeep = pet.resource_upkeep as f32;
        if resources.food >= upkeep {
            resources.food -= upkeep;
        } else {
            morale.value = (morale.value - 0.5).clamp(0.0, 1.0);
        }
    }
}

pub fn viral_pet_spread_system(
    mut commands: Commands,
    pets_query: Query<&XenofloraPet>,
    no_pets_query: Query<Entity, (With<Pop>, Without<XenofloraPet>)>,
) {
    if pets_query.is_empty() {
        return; // No pets to spread
    }

    let mut rng = rand::thread_rng();

    for entity in no_pets_query.iter() {
        // 5% chance to adopt a pet if someone in the colony has one
        if rng.gen::<f32>() < 0.05 {
            commands.entity(entity).insert(XenofloraPet {
                resource_upkeep: 1,
                mood_boost: 0.25,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pet_creation() {
        let mut app = App::new();

        let pop_entity = app.world_mut().spawn((
            Pop,
            Morale { value: 0.5, ..Default::default() },
            XenofloraPet {
                resource_upkeep: 1,
                mood_boost: 0.25,
            },
        )).id();

        let pet = app.world().get::<XenofloraPet>(pop_entity).unwrap();
        assert_eq!(pet.resource_upkeep, 1);
        assert_eq!(pet.mood_boost, 0.25);
    }

    #[test]
    fn test_pet_mood_boost() {
        let mut app = App::new();
        app.add_systems(Update, apply_pet_mood_boost);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Morale { value: 0.5, ..Default::default() },
            XenofloraPet {
                resource_upkeep: 1,
                mood_boost: 0.25,
            },
        )).id();

        app.update();

        let morale = app.world().get::<Morale>(pop_entity).unwrap();
        assert_eq!(morale.modifiers.len(), 1);
        assert_eq!(morale.modifiers[0].label, "Xenoflora Pet");
        assert_eq!(morale.modifiers[0].value, 0.25);
    }

    #[test]
    fn test_pet_upkeep_success() {
        let mut app = App::new();
        app.insert_resource(ColonyResources {
            food: 10.0,
            ..Default::default()
        });
        app.add_systems(Update, pet_upkeep_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Morale { value: 0.8, ..Default::default() },
            XenofloraPet {
                resource_upkeep: 2,
                mood_boost: 0.25,
            },
        )).id();

        app.update();

        let resources = app.world().get_resource::<ColonyResources>().unwrap();
        assert_eq!(resources.food, 8.0);

        let morale = app.world().get::<Morale>(pop_entity).unwrap();
        assert_eq!(morale.value, 0.8);
    }

    #[test]
    fn test_pet_upkeep_failure() {
        let mut app = App::new();
        app.insert_resource(ColonyResources {
            food: 1.0,
            ..Default::default()
        });
        app.add_systems(Update, pet_upkeep_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Morale { value: 0.8, ..Default::default() },
            XenofloraPet {
                resource_upkeep: 2,
                mood_boost: 0.25,
            },
        )).id();

        app.update();

        let resources = app.world().get_resource::<ColonyResources>().unwrap();
        assert_eq!(resources.food, 1.0); // Not enough food, so nothing deducted

        let morale = app.world().get::<Morale>(pop_entity).unwrap();
        assert_eq!(morale.value, 0.3); // Morale penalized due to starvation
    }

    #[test]
    fn test_viral_pet_spread() {
        let mut app = App::new();
        app.add_systems(Update, viral_pet_spread_system);

        // Spawn a pop WITH a pet
        app.world_mut().spawn((
            Pop,
            XenofloraPet {
                resource_upkeep: 1,
                mood_boost: 0.25,
            },
        ));

        // Spawn many pops WITHOUT a pet to increase the chance that at least one adopts
        let mut entities = Vec::new();
        for _ in 0..100 {
            entities.push(app.world_mut().spawn(Pop).id());
        }

        // Run the system
        app.update();

        // Check if any of the initially petless pops got a pet
        let mut adoptions = 0;
        for e in entities {
            if app.world().get::<XenofloraPet>(e).is_some() {
                adoptions += 1;
            }
        }

        assert!(adoptions > 0, "Some pops should adopt a pet");
    }

    #[test]
    fn test_no_viral_spread_without_pet() {
        let mut app = App::new();
        app.add_systems(Update, viral_pet_spread_system);

        // Spawn a pop WITHOUT a pet
        let pop = app.world_mut().spawn(Pop).id();

        // Run the system
        app.update();

        // Check that the pop still has no pet
        assert!(app.world().get::<XenofloraPet>(pop).is_none());
    }
}
