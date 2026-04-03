use bevy::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::structure::Structure;

#[derive(Component)]
pub struct ParasiticArchitecture {
    pub radius: f32,
    pub consumption_rate: f32,
}

pub fn process_megastructure_consumption(
    mut commands: Commands,
    megastructures: Query<(&GridPosition, &ParasiticArchitecture)>,
    mut buildings: Query<(Entity, &GridPosition, &mut Structure), Without<ParasiticArchitecture>>,
) {
    for (mega_pos, parasitic) in megastructures.iter() {
        for (entity, build_pos, mut structure) in buildings.iter_mut() {
            let distance = ((mega_pos.x as f32 - build_pos.x as f32).powi(2) +
                            (mega_pos.y as f32 - build_pos.y as f32).powi(2)).sqrt();

            if distance <= parasitic.radius {
                structure.current_hp -= parasitic.consumption_rate;

                // Destroy if integrity is depleted
                if structure.current_hp <= 0.0 {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_megastructure_consumes_nearby_building_integrity() {
        let mut app = App::new();
        app.add_systems(Update, process_megastructure_consumption);

        let megastructure = app.world_mut().spawn((
            GridPosition { x: 10, y: 10 },
            Building { building_type: BuildingType::Housing }, // Spire not in BuildingType, using Housing for now.
            ParasiticArchitecture { radius: 2.0, consumption_rate: 10.0 },
        )).id();

        let victim = app.world_mut().spawn((
            GridPosition { x: 11, y: 10 },
            Building { building_type: BuildingType::Housing },
            Structure { current_hp: 100.0, max_hp: 100.0 },
        )).id();

        // Assume consumption event occurs every tick
        app.update();

        // Assert the victim building lost integrity
        let integrity = app.world().get::<Structure>(victim).unwrap();
        assert_eq!(integrity.current_hp, 90.0);
    }
}
