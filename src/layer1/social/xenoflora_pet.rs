use crate::layer1::economy::resources::{ColonyResources, ResourceType};
use crate::layer1::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct XenofloraPet {
    pub resource_upkeep: f32,
    pub mood_boost: f32,
}

#[derive(Clone, Copy)]
pub struct XenofloraPetSpreadConfig {
    pub spread_chance: f32,
    pub resource_upkeep: f32,
    pub mood_boost: f32,
}

impl Default for XenofloraPetSpreadConfig {
    fn default() -> Self {
        Self {
            spread_chance: 0.05,
            resource_upkeep: 1.0,
            mood_boost: 0.25,
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn pet_viral_spread_system(
    mut commands: Commands,
    query: Query<
        (Entity, &crate::layer1::map::GridPosition),
        (With<crate::layer1::pop::Pop>, Without<XenofloraPet>),
    >,
    pet_query: Query<
        &crate::layer1::map::GridPosition,
        (With<crate::layer1::pop::Pop>, With<XenofloraPet>),
    >,
) {
    let config = XenofloraPetSpreadConfig::default();
    let mut rng = rand::thread_rng();

    for (entity, pos) in query.iter() {
        // Check if there is any pop with a pet nearby
        for pet_pos in pet_query.iter() {
            let dx = (pos.x - pet_pos.x).abs();
            let dy = (pos.y - pet_pos.y).abs();

            if dx <= 2 && dy <= 2 {
                use rand::Rng;
                if rng.gen::<f32>() < config.spread_chance {
                    commands.entity(entity).insert(XenofloraPet {
                        resource_upkeep: config.resource_upkeep,
                        mood_boost: config.mood_boost,
                    });
                    break; // Spread once per tick
                }
            }
        }
    }
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
    fn test_pet_viral_spread() {
        let mut app = App::new();
        app.add_systems(bevy_app::Update, pet_viral_spread_system);

        let _pop1_with_pet = app
            .world_mut()
            .spawn((
                Pop,
                crate::layer1::map::GridPosition { x: 10, y: 10 },
                XenofloraPet {
                    resource_upkeep: 1.0,
                    mood_boost: 0.25,
                },
            ))
            .id();

        let pop2_without_pet = app
            .world_mut()
            .spawn((Pop, crate::layer1::map::GridPosition { x: 11, y: 11 }))
            .id();

        let pop3_too_far = app
            .world_mut()
            .spawn((Pop, crate::layer1::map::GridPosition { x: 20, y: 20 }))
            .id();

        // Run multiple times to almost guarantee spread due to 5% chance
        for _ in 0..1000 {
            app.update();
        }

        assert!(
            app.world().get::<XenofloraPet>(pop2_without_pet).is_some(),
            "Nearby pop should have caught the pet craze"
        );
        assert!(
            app.world().get::<XenofloraPet>(pop3_too_far).is_none(),
            "Far away pop should not catch the pet craze"
        );
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
