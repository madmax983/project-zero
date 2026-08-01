use bevy::prelude::*;

#[derive(Component)]
pub struct Position {
    pub z: f32, // 0.0 is Ice crust, negative is deep water
}

#[derive(Component)]
pub struct Mass(pub f32);

#[derive(Component)]
pub struct Buoyancy(pub f32);

#[derive(Component, PartialEq, Eq)]
pub enum AnchorState {
    Intact,
    Snapped,
}

#[derive(Component)]
pub struct Velocity(pub f32);

#[derive(Component)]
pub struct StructuralIntegrity(pub f32);

pub fn buoyancy_movement_system(
    mut query: Query<(&mut Position, &mut Velocity, &Mass, &Buoyancy, &AnchorState)>,
) {
    for (mut pos, mut vel, mass, buoyancy, anchor) in query.iter_mut() {
        if *anchor == AnchorState::Snapped {
            let net_force = buoyancy.0 - mass.0;
            let acceleration = net_force / mass.0;
            vel.0 += acceleration * 0.1; // Delta time simplification
            pos.z += vel.0;
        } else {
            vel.0 = 0.0;
        }
    }
}

pub fn ice_collision_system(
    mut query: Query<(&mut Position, &mut Velocity, &mut StructuralIntegrity)>,
) {
    for (mut pos, mut vel, mut health) in query.iter_mut() {
        if pos.z >= 0.0 {
            pos.z = 0.0; // Stop at crust
            if vel.0 > 0.0 {
                // Take impact damage
                health.0 -= vel.0 * 5.0;
                vel.0 = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapped_anchor_causes_upward_floating() {
        let mut app = App::new();
        app.add_systems(Update, buoyancy_movement_system);

        let module = app
            .world_mut()
            .spawn((
                Position { z: -100.0 }, // 100 meters below ice
                Mass(1000.0),
                Buoyancy(1500.0), // Buoyancy > Mass
                AnchorState::Snapped,
                Velocity(0.0),
            ))
            .id();

        app.update();

        // Object should move upwards (z increases towards 0.0)
        let pos = app.world().get::<Position>(module).unwrap();
        assert!(pos.z > -100.0);
    }

    #[test]
    fn test_intact_anchor_prevents_floating() {
        let mut app = App::new();
        app.add_systems(Update, buoyancy_movement_system);

        let module = app
            .world_mut()
            .spawn((
                Position { z: -100.0 },
                Mass(1000.0),
                Buoyancy(1500.0),
                AnchorState::Intact, // Anchor holding it down
                Velocity(0.0),
            ))
            .id();

        app.update();

        // Object should not move
        let pos = app.world().get::<Position>(module).unwrap();
        assert_eq!(pos.z, -100.0);
    }

    #[test]
    fn test_collision_with_ice_crust() {
        let mut app = App::new();
        app.add_systems(
            Update,
            (buoyancy_movement_system, ice_collision_system).chain(),
        );

        let module = app
            .world_mut()
            .spawn((
                Position { z: -1.0 }, // Right below ice
                Mass(1000.0),
                Buoyancy(5000.0), // High buoyancy = fast ascent
                Velocity(10.0),
                AnchorState::Snapped,
                StructuralIntegrity(100.0),
            ))
            .id();

        app.update();

        // Module hits z = 0.0, takes collision damage based on velocity/buoyancy
        let pos = app.world().get::<Position>(module).unwrap();
        let health = app.world().get::<StructuralIntegrity>(module).unwrap();

        assert_eq!(pos.z, 0.0); // Stopped at ceiling
        assert!(health.0 < 100.0); // Took damage
    }
}

use crate::layer1::core::map::GridPosition;
use crate::layer1::nature::terrain::TerrainGrid;

#[derive(Component)]
pub struct SubterraneanOcean {
    pub pressure: f32,
}

#[derive(Component)]
pub struct FluidLevel {
    pub amount: f32,
}

#[derive(Event)]
pub struct MiningEvent {
    pub target: Entity,
}

#[derive(Event)]
pub struct OceanBreachEvent {
    pub location: GridPosition,
}

#[derive(Component)]
pub struct DeepPearl {
    pub value: u32,
}

#[derive(Event)]
pub struct HarvestEvent {
    pub target: Entity,
}

#[derive(Resource, Default)]
pub struct ColonyWealth {
    pub credits: u32,
}

pub fn process_mining_breach(
    mut events: EventReader<MiningEvent>,
    ocean_query: Query<(&GridPosition, &SubterraneanOcean)>,
    mut mine_query: Query<(&GridPosition, &mut FluidLevel)>,
    terrain_grid: Res<TerrainGrid>,
    mut breach_events: EventWriter<OceanBreachEvent>,
) {
    for event in events.read() {
        if let Ok((ocean_pos, ocean)) = ocean_query.get(event.target) {
            breach_events.send(OceanBreachEvent {
                location: GridPosition {
                    x: ocean_pos.x,
                    y: ocean_pos.y,
                },
            });

            for (mine_pos, mut fluid) in mine_query.iter_mut() {
                if mine_pos.x >= 0 && mine_pos.y >= 0 {
                    if let Some(terrain_type) =
                        terrain_grid.get(mine_pos.x as usize, mine_pos.y as usize)
                    {
                        if terrain_type.is_walkable() {
                            let dx = (ocean_pos.x - mine_pos.x).abs();
                            let dy = (ocean_pos.y - mine_pos.y).abs();
                            if dx <= 1 && dy <= 1 {
                                fluid.amount += ocean.pressure * 0.1;
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn process_pearl_harvesting(
    mut commands: Commands,
    mut events: EventReader<HarvestEvent>,
    pearl_query: Query<&DeepPearl>,
    mut wealth: ResMut<ColonyWealth>,
) {
    for event in events.read() {
        if let Ok(pearl) = pearl_query.get(event.target) {
            wealth.credits += pearl.value;
            commands.entity(event.target).despawn();
        }
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::layer1::nature::terrain::TerrainType;

    #[test]
    fn test_ocean_breach_floods_mine() {
        let mut app = App::new();
        app.add_event::<MiningEvent>();
        app.add_event::<OceanBreachEvent>();
        app.add_systems(Update, process_mining_breach);

        let mut tiles = vec![TerrainType::Rock; 100];
        tiles[55] = TerrainType::Grass; // open tile at (5, 5) assuming 10x10
        app.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        let ocean = app
            .world_mut()
            .spawn((
                GridPosition { x: 5, y: 6 },
                SubterraneanOcean { pressure: 1000.0 },
            ))
            .id();

        let mine_tile = app
            .world_mut()
            .spawn((GridPosition { x: 5, y: 5 }, FluidLevel { amount: 0.0 }))
            .id();

        app.world_mut().send_event(MiningEvent { target: ocean });
        app.update();

        let fluid = app.world().get::<FluidLevel>(mine_tile).unwrap();
        assert!(
            fluid.amount > 0.0,
            "Mine tile should be flooded after breaching the ocean"
        );

        let events = app.world().resource::<Events<OceanBreachEvent>>();
        let mut reader = events.get_cursor();
        assert!(
            reader.read(events).next().is_some(),
            "An ocean breach event should have been dispatched"
        );
    }

    #[test]
    fn test_deep_pearl_harvest() {
        let mut app = App::new();
        app.add_event::<HarvestEvent>();
        app.add_systems(Update, process_pearl_harvesting);

        let pearl = app
            .world_mut()
            .spawn((GridPosition { x: 5, y: -10 }, DeepPearl { value: 500 }))
            .id();

        app.world_mut().send_event(HarvestEvent { target: pearl });
        app.world_mut().insert_resource(ColonyWealth { credits: 0 });

        app.update();

        let wealth = app.world().get_resource::<ColonyWealth>().unwrap();
        assert_eq!(
            wealth.credits, 500,
            "Harvesting a deep pearl should add to colony wealth"
        );

        let pearl_exists = app.world().get::<DeepPearl>(pearl).is_some();
        assert!(!pearl_exists, "Harvested pearl should be removed");
    }
}
