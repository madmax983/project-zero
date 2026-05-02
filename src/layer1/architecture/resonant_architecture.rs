use crate::layer1::core::map::GridPosition;
use crate::layer1::entities::pop::Pop;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Room {
    pub material: BuildingMaterial,
}

#[derive(PartialEq, Clone, Debug)]
pub enum BuildingMaterial {
    Standard,
    MindStone,
    IronPlating,
}

#[derive(Component)]
pub struct RoomBoundary {
    pub radius: u32,
}

#[derive(Component)]
pub struct PopResonanceTraits {
    pub research_speed_mult: f32,
    pub stress_gain_mult: f32,
}

impl Default for PopResonanceTraits {
    fn default() -> Self {
        Self {
            research_speed_mult: 1.0,
            stress_gain_mult: 1.0,
        }
    }
}

pub fn apply_resonant_architecture_system(
    room_query: Query<(&Room, &RoomBoundary, &GridPosition)>,
    mut pop_query: Query<(&mut PopResonanceTraits, &GridPosition), With<Pop>>,
) {
    for (mut traits, pop_pos) in pop_query.iter_mut() {
        // Reset to base
        traits.research_speed_mult = 1.0;
        traits.stress_gain_mult = 1.0;

        for (room, boundary, room_pos) in room_query.iter() {
            let dx = (pop_pos.x as i64 - room_pos.x as i64).unsigned_abs() as u32;
            let dy = (pop_pos.y as i64 - room_pos.y as i64).unsigned_abs() as u32;
            let dist = dx.max(dy); // Chebyshev distance is common for grids

            if dist <= boundary.radius {
                match room.material {
                    BuildingMaterial::MindStone => {
                        traits.research_speed_mult *= 2.0;
                        traits.stress_gain_mult *= 2.0;
                    }
                    BuildingMaterial::IronPlating => {
                        // Placeholders
                    }
                    BuildingMaterial::Standard => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_resonant_material_amplifies_trait() {
        let mut app = App::new();
        app.add_systems(Update, apply_resonant_architecture_system);

        app.world_mut().spawn((
            Room {
                material: BuildingMaterial::MindStone,
            },
            RoomBoundary { radius: 5 },
            GridPosition { x: 0, y: 0 },
        ));

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 1, y: 0 }, // Inside room
                PopResonanceTraits::default(),
            ))
            .id();

        app.update();

        let traits = app.world().get::<PopResonanceTraits>(pop).unwrap();
        assert_eq!(traits.research_speed_mult, 2.0);
        assert_eq!(traits.stress_gain_mult, 2.0);
    }

    #[test]
    fn test_pop_outside_room_not_affected() {
        let mut app = App::new();
        app.add_systems(Update, apply_resonant_architecture_system);

        app.world_mut().spawn((
            Room {
                material: BuildingMaterial::MindStone,
            },
            RoomBoundary { radius: 5 },
            GridPosition { x: 0, y: 0 },
        ));

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 10, y: 0 }, // Outside room
                PopResonanceTraits::default(),
            ))
            .id();

        app.update();

        let traits = app.world().get::<PopResonanceTraits>(pop).unwrap();
        assert_eq!(traits.research_speed_mult, 1.0);
        assert_eq!(traits.stress_gain_mult, 1.0);
    }
}
