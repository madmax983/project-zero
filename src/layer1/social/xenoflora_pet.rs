use crate::layer1::economy::resources::{ColonyResources, ResourceType};
use crate::layer1::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct XenofloraPet {
    pub resource_upkeep: f32,
    pub mood_boost: f32,
}

pub fn apply_pet_mood_boost(
    mut query: Query<(&mut Morale, &XenofloraPet)>,
    mut resources: ResMut<ColonyResources>,
) {
    for (mut morale, pet) in query.iter_mut() {
        if resources.food >= pet.resource_upkeep {
            resources.consume(ResourceType::Food, pet.resource_upkeep);
            morale.add_modifier(MoodModifier {
                label: "Pet Xenoflora".to_string(),
                value: pet.mood_boost,
                duration: 1,
            });
        } else {
            morale.add_modifier(MoodModifier {
                label: "Starving Pet".to_string(),
                value: -pet.mood_boost * 2.0, // Starving risk drops mood drastically
                duration: 1,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::resources::ColonyResources;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::Pop;
    use bevy_app::App;

    #[test]
    fn test_pet_creation() {
        let mut app = App::new();

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                },
                XenofloraPet {
                    resource_upkeep: 1.0,
                    mood_boost: 0.25,
                },
            ))
            .id();

        let pet = app.world().get::<XenofloraPet>(pop_entity).unwrap();
        assert_eq!(pet.resource_upkeep, 1.0);
        assert_eq!(pet.mood_boost, 0.25);
    }

    #[test]
    fn test_pet_mood_boost() {
        let mut app = App::new();
        // Setup initial system that applies the mood boost
        app.insert_resource(ColonyResources {
            food: 10.0,
            ..ColonyResources::zeroed()
        });
        app.add_systems(bevy_app::Update, apply_pet_mood_boost);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                },
                XenofloraPet {
                    resource_upkeep: 1.0,
                    mood_boost: 0.25,
                },
            ))
            .id();

        app.update();

        let morale = app.world().get::<Morale>(pop_entity).unwrap();
        assert_eq!(morale.modifiers.len(), 1);
        assert_eq!(morale.modifiers[0].value, 0.25);

        let resources = app.world().get_resource::<ColonyResources>().unwrap();
        assert_eq!(resources.food, 9.0);
    }

    #[test]
    fn test_pet_starvation() {
        let mut app = App::new();
        // Setup with no food
        app.insert_resource(ColonyResources {
            food: 0.0,
            ..ColonyResources::zeroed()
        });
        app.add_systems(bevy_app::Update, apply_pet_mood_boost);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                },
                XenofloraPet {
                    resource_upkeep: 1.0,
                    mood_boost: 0.25,
                },
            ))
            .id();

        app.update();

        let morale = app.world().get::<Morale>(pop_entity).unwrap();
        assert_eq!(morale.modifiers.len(), 1);
        assert_eq!(morale.modifiers[0].value, -0.50);
        assert_eq!(morale.modifiers[0].label, "Starving Pet");
    }
}
