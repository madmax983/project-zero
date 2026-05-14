use bevy_ecs::prelude::*;
use crate::layer1::pop::PopBundle;

#[derive(Component)]
pub struct DiasporaFleet {
    pub refugee_count: u32,
    pub arrival_time: f32,
}

pub fn handle_diaspora_arrival_system(
    mut commands: Commands,
    time: Res<crate::shared::time::SimulationTime>,
    query: Query<(Entity, &DiasporaFleet)>,
) {
    let mut rng = rand::thread_rng();

    for (entity, fleet) in query.iter() {
        if (time.tick as f32) >= fleet.arrival_time {
            // Arrived! Spawn the refugees in Layer 1.
            // Using a simple 0,0 location for MVP or random. Let's use 0,0 for simplicity since they might just "appear" at the port.
            for _ in 0..fleet.refugee_count {
                commands.spawn(PopBundle::random(0, 0, &mut rng));
            }

            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_diaspora_fleet_creation() {
        let mut app = App::new();

        let fleet_entity = app.world_mut().spawn((
            DiasporaFleet {
                refugee_count: 100,
                arrival_time: 10.0,
            },
        )).id();

        let fleet = app.world().get::<DiasporaFleet>(fleet_entity).unwrap();
        assert_eq!(fleet.refugee_count, 100);
    }

    #[test]
    fn test_diaspora_fleet_arrives_and_despawns() {
        let mut app = App::new();
        app.insert_resource(crate::shared::time::SimulationTime { tick: 15, ..Default::default() });
        app.add_systems(Update, handle_diaspora_arrival_system);

        let fleet_entity = app.world_mut().spawn((
            DiasporaFleet {
                refugee_count: 100,
                arrival_time: 10.0,
            },
        )).id();

        app.update();

        // Ensure the fleet entity is removed
        assert!(app.world().get_entity(fleet_entity).is_err() || app.world().get::<DiasporaFleet>(fleet_entity).is_none());

        // Ensure pops were spawned
        let pop_count = app.world_mut().query::<&Pop>().iter(app.world()).count();
        assert_eq!(pop_count, 100, "Refugees should be spawned as Pops");
    }
}
