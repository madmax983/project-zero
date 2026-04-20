use bevy_ecs::prelude::*;
use crate::layer1::social::morale::{Morale, MoodModifier};

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
                duration: 1, // Basic tick duration
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use crate::layer1::entities::Pop;

    #[test]
    fn test_pet_creation() {
        let mut app = App::new();

        let pop_entity = app.world_mut().spawn((
            Pop,
            Morale { value: 50.0, modifiers: Vec::new() },
            XenofloraPet {
                resource_upkeep: 1,
                mood_boost: 25.0,
            },
        )).id();

        let pet = app.world().get::<XenofloraPet>(pop_entity).unwrap();
        assert_eq!(pet.resource_upkeep, 1);
        assert_eq!(pet.mood_boost, 25.0);
    }

    #[test]
    fn test_pet_mood_boost() {
        let mut app = App::new();
        app.add_systems(bevy_app::Update, apply_pet_mood_boost);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Morale { value: 50.0, modifiers: Vec::new() },
            XenofloraPet {
                resource_upkeep: 1,
                mood_boost: 25.0,
            },
        )).id();

        app.update();

        let morale = app.world().get::<Morale>(pop_entity).unwrap();
        assert_eq!(morale.modifiers.len(), 1);
        assert_eq!(morale.modifiers[0].value, 25.0);
    }
}
