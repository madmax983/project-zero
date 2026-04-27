use crate::layer1::architecture::structure::Structure;
use crate::layer1::map::GridPosition;
use crate::layer1::nature::terrain::TerrainType;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct IncomingImpact {
    pub target_pos: GridPosition,
    pub radius: u32,
    pub ticks_remaining: u32,
}

#[derive(Event)]
pub struct ImpactWarningEvent {
    pub target_pos: GridPosition,
    pub ticks_remaining: u32,
}

#[derive(Event)]
pub struct ImpactStrikeEvent {
    pub center: GridPosition,
    pub radius: u32,
}

pub fn process_impact_countdown_system(
    mut commands: Commands,
    mut impacts: Query<(Entity, &mut IncomingImpact)>,
    mut strike_events: EventWriter<ImpactStrikeEvent>,
    mut warning_events: EventWriter<ImpactWarningEvent>,
    mut chronicle_events: EventWriter<crate::layer1::core::chronicle::AddChronicleEvent>,
) {
    for (entity, mut impact) in impacts.iter_mut() {
        if impact.ticks_remaining > 0 {
            impact.ticks_remaining -= 1;
            warning_events.send(ImpactWarningEvent {
                target_pos: impact.target_pos,
                ticks_remaining: impact.ticks_remaining,
            });
        }

        if impact.ticks_remaining == 0 {
            strike_events.send(ImpactStrikeEvent {
                center: impact.target_pos,
                radius: impact.radius,
            });
            chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
                importance: crate::layer1::core::chronicle::EventImportance::Major,
                text: "A catastrophic impact struck the colony.".to_string(),
            });
            commands.entity(entity).despawn();
        }
    }
}

pub fn process_impact_strike_system(
    mut strike_events: EventReader<ImpactStrikeEvent>,
    mut commands: Commands,
    mut structures: Query<(Entity, &GridPosition, &mut Structure)>,
    mut terrains: Query<(&GridPosition, &mut TerrainType)>,
    mut spark_events: EventWriter<crate::layer1::environment::ignition::SparkEvent>,
) {
    for ev in strike_events.read() {
        // Obliterate structures
        for (entity, pos, mut structure) in structures.iter_mut() {
            let dist = pos.distance_chebyshev(ev.center);
            if dist <= ev.radius / 2 {
                commands.entity(entity).despawn();
            } else if dist <= ev.radius {
                structure.current_hp /= 2.0;
                spark_events
                    .send(crate::layer1::environment::ignition::SparkEvent { position: *pos });
            }
        }

        // Crater terrain
        for (pos, mut terrain) in terrains.iter_mut() {
            if pos.distance_chebyshev(ev.center) <= ev.radius / 2 {
                *terrain = TerrainType::Crater;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::structure::Structure;
    use crate::layer1::map::GridPosition;
    use crate::layer1::nature::terrain::TerrainType;
    use crate::shared::time::SimulationTime;
    use bevy::prelude::App;
    use bevy::prelude::Events;
    use bevy::prelude::Update;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(SimulationTime::default());
        app.add_event::<ImpactWarningEvent>();
        app.add_event::<ImpactStrikeEvent>();
        app.add_event::<crate::layer1::core::chronicle::AddChronicleEvent>();
        app.add_event::<crate::layer1::environment::ignition::SparkEvent>();
        app.add_systems(
            Update,
            (
                process_impact_countdown_system,
                process_impact_strike_system,
            ),
        );
        app
    }

    #[test]
    fn test_impact_countdown_triggers_strike() {
        let mut app = setup_app();

        let impact = app
            .world_mut()
            .spawn(IncomingImpact {
                target_pos: GridPosition { x: 20, y: 20 },
                radius: 5,
                ticks_remaining: 10,
            })
            .id();

        // Advance time 9 ticks
        for _ in 0..9 {
            app.world_mut().resource_mut::<SimulationTime>().tick += 1;
            app.update();
        }

        // Still exists
        assert!(app.world().get::<IncomingImpact>(impact).is_some());

        // Advance 10th tick
        app.world_mut().resource_mut::<SimulationTime>().tick += 1;
        app.update();

        // Impact should have struck and despawned
        assert!(app.world().get::<IncomingImpact>(impact).is_none());

        // Verify strike event
        let strike_events = app
            .world()
            .get_resource::<Events<ImpactStrikeEvent>>()
            .unwrap();
        let mut reader = strike_events.get_cursor();
        let event = reader.read(strike_events).next().unwrap();
        assert_eq!(event.center, GridPosition { x: 20, y: 20 });
        assert_eq!(event.radius, 5);
    }

    #[test]
    fn test_impact_strike_destroys_structures_and_alters_terrain() {
        let mut app = setup_app();

        let center = GridPosition { x: 10, y: 10 };

        // Setup terrain and structure
        let terrain = app.world_mut().spawn((center, TerrainType::Grass)).id();

        let structure = app
            .world_mut()
            .spawn((
                center,
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        // Spawn another structure on the edge of the radius
        let edge = GridPosition { x: 10, y: 12 }; // distance 2 from center
        let edge_structure = app
            .world_mut()
            .spawn((
                edge,
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        app.world_mut().send_event(ImpactStrikeEvent {
            center,
            radius: 2, // Will hit center (0 <= 1), edge (2 <= 2)
        });

        app.update();

        // Structure at center should be obliterated
        assert!(app.world().get_entity(structure).is_err());

        // Terrain at center should be converted to Crater
        let new_terrain = app.world().get::<TerrainType>(terrain).unwrap();
        assert_eq!(*new_terrain, TerrainType::Crater);

        // Structure at edge should be damaged but not obliterated
        assert!(app.world().get_entity(edge_structure).is_ok());
        let edge_s = app.world().get::<Structure>(edge_structure).unwrap();
        assert_eq!(edge_s.current_hp, 50.0);
    }
}
