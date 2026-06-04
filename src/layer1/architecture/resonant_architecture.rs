use crate::layer1::entities::pop::Pop;
use bevy::prelude::Transform;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Room {
    pub material: BuildingMaterial,
}

#[derive(PartialEq, Clone, Debug, Default)]
pub enum BuildingMaterial {
    #[default]
    Standard,
    MindStone,
    IronPlating,
}

#[derive(Component)]
pub struct RoomBoundary {
    pub radius: f32,
}

#[derive(Component)]
pub struct PopTraits {
    pub research_speed: f32,
    pub stress_gain: f32,
}

pub fn apply_resonant_architecture_system(
    room_query: Query<(&Room, &RoomBoundary, &Transform)>,
    mut pop_query: Query<(&mut PopTraits, &Transform), With<Pop>>,
) {
    for (mut traits, pop_transform) in pop_query.iter_mut() {
        traits.research_speed = 1.0;
        traits.stress_gain = 1.0;

        for (room, boundary, room_transform) in room_query.iter() {
            let dist = room_transform
                .translation
                .distance(pop_transform.translation);
            if dist <= boundary.radius {
                match room.material {
                    BuildingMaterial::MindStone => {
                        traits.research_speed *= 2.0;
                        traits.stress_gain *= 2.0;
                    }
                    BuildingMaterial::IronPlating => {}
                    BuildingMaterial::Standard => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Transform;
    use bevy_app::{App, Update};

    #[test]
    fn test_resonant_material_amplifies_trait() {
        let mut app = App::new();
        app.add_systems(Update, apply_resonant_architecture_system);

        let _room = app
            .world_mut()
            .spawn((
                Room {
                    material: BuildingMaterial::MindStone,
                },
                RoomBoundary { radius: 5.0 },
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Transform::from_xyz(1.0, 0.0, 0.0),
                PopTraits {
                    research_speed: 1.0,
                    stress_gain: 1.0,
                },
            ))
            .id();

        app.update();

        let traits = app.world().get::<PopTraits>(pop).unwrap();
        assert_eq!(traits.research_speed, 2.0);
        assert_eq!(traits.stress_gain, 2.0);
    }

    #[test]
    fn test_pop_outside_room_not_affected() {
        let mut app = App::new();
        app.add_systems(Update, apply_resonant_architecture_system);

        let _room = app
            .world_mut()
            .spawn((
                Room {
                    material: BuildingMaterial::MindStone,
                },
                RoomBoundary { radius: 5.0 },
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Transform::from_xyz(10.0, 0.0, 0.0),
                PopTraits {
                    research_speed: 1.0,
                    stress_gain: 1.0,
                },
            ))
            .id();

        app.update();

        let traits = app.world().get::<PopTraits>(pop).unwrap();
        assert_eq!(traits.research_speed, 1.0);
        assert_eq!(traits.stress_gain, 1.0);
    }

    #[test]
    fn test_iron_plating_does_nothing() {
        let mut app = App::new();
        app.add_systems(Update, apply_resonant_architecture_system);

        let _room = app
            .world_mut()
            .spawn((
                Room {
                    material: BuildingMaterial::IronPlating,
                },
                RoomBoundary { radius: 5.0 },
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Transform::from_xyz(1.0, 0.0, 0.0),
                PopTraits {
                    research_speed: 1.0,
                    stress_gain: 1.0,
                },
            ))
            .id();

        app.update();

        let traits = app.world().get::<PopTraits>(pop).unwrap();
        assert_eq!(traits.research_speed, 1.0);
        assert_eq!(traits.stress_gain, 1.0);
    }

    #[test]
    fn test_standard_material_does_nothing() {
        let mut app = App::new();
        app.add_systems(Update, apply_resonant_architecture_system);

        let _room = app
            .world_mut()
            .spawn((
                Room {
                    material: BuildingMaterial::Standard,
                },
                RoomBoundary { radius: 5.0 },
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Transform::from_xyz(1.0, 0.0, 0.0),
                PopTraits {
                    research_speed: 1.0,
                    stress_gain: 1.0,
                },
            ))
            .id();

        app.update();

        let traits = app.world().get::<PopTraits>(pop).unwrap();
        assert_eq!(traits.research_speed, 1.0);
        assert_eq!(traits.stress_gain, 1.0);
    }
}
