use bevy::prelude::DespawnRecursiveExt;
use bevy_ecs::prelude::*;
use bevy_time::Time;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectorId(pub u32);

#[derive(Debug, Clone)]
pub struct SectorData {
    pub last_scanned: u64,
    pub accuracy: f32,
}

#[derive(Resource, Default)]
pub struct MapData {
    pub sectors: HashMap<SectorId, SectorData>,
}

impl MapData {
    pub fn new() -> Self {
        Self {
            sectors: HashMap::new(),
        }
    }

    pub fn insert_sector(&mut self, id: SectorId, data: SectorData) {
        self.sectors.insert(id, data);
    }

    pub fn get_sector(&self, id: SectorId) -> Option<&SectorData> {
        self.sectors.get(&id)
    }
}

pub fn map_data_rot_system(time: Option<Res<Time>>, mut map: Option<ResMut<MapData>>) {
    let delta = if let Some(time) = time {
        time.delta_secs()
    } else {
        return;
    };

    if let Some(map) = map.as_deref_mut() {
        for data in map.sectors.values_mut() {
            // Decay accuracy over time
            data.accuracy -= delta * 0.01;
            if data.accuracy < 0.0 {
                data.accuracy = 0.0;
            }
        }
    }
}

#[derive(Component)]
pub struct ScoutShip;

#[derive(Component)]
pub struct SectorPosition(pub SectorId);

pub fn scout_ship_scan_system(
    query: Query<&SectorPosition, With<ScoutShip>>,
    mut map: Option<ResMut<MapData>>,
) {
    if let Some(map) = map.as_deref_mut() {
        for pos in query.iter() {
            if let Some(data) = map.sectors.get_mut(&pos.0) {
                data.accuracy = 1.0;
            }
        }
    }
}

#[derive(Event, Debug)]
pub struct FleetArrivalEvent {
    pub fleet: Entity,
    pub sector: SectorId,
}

#[derive(Event, Debug)]
pub struct AnomalyDiscoveredEvent {
    pub sector: SectorId,
}

pub fn fleet_arrival_anomaly_system(
    mut events: EventReader<FleetArrivalEvent>,
    map: Option<Res<MapData>>,
    mut anomalies: EventWriter<AnomalyDiscoveredEvent>,
) {
    if let Some(map) = map.as_deref() {
        for event in events.read() {
            if let Some(data) = map.get_sector(event.sector) {
                if data.accuracy < 0.5 {
                    anomalies.send(AnomalyDiscoveredEvent {
                        sector: event.sector,
                    });
                }
            }
        }
    }
}

#[derive(Component)]
pub struct StarSystem {
    pub id: u32,
}

#[derive(Component)]
pub struct Hyperlane {
    pub start: Entity,
    pub end: Entity,
    pub stability: f32,
}

#[derive(Event)]
pub struct HyperlaneCollapseEvent {
    pub lane_entity: Entity,
}

#[derive(Event)]
pub struct TradeRouteSeveredEvent {
    pub system_a: Entity,
    pub system_b: Entity,
}

pub fn trigger_hyperlane_collapse_system(
    query: Query<(Entity, &Hyperlane)>,
    mut collapse_events: EventWriter<HyperlaneCollapseEvent>,
) {
    for (entity, lane) in query.iter() {
        if lane.stability <= 0.0 {
            collapse_events.send(HyperlaneCollapseEvent {
                lane_entity: entity,
            });
        }
    }
}

pub fn process_hyperlane_collapse_system(
    mut commands: Commands,
    mut events: EventReader<HyperlaneCollapseEvent>,
    lane_query: Query<&Hyperlane>,
    mut severed_events: EventWriter<TradeRouteSeveredEvent>,
) {
    for ev in events.read() {
        if let Ok(lane) = lane_query.get(ev.lane_entity) {
            severed_events.send(TradeRouteSeveredEvent {
                system_a: lane.start,
                system_b: lane.end,
            });
            commands.entity(ev.lane_entity).despawn_recursive();
        }
    }
}

pub fn recalculate_trade_routes_system(mut events: EventReader<TradeRouteSeveredEvent>) {
    for _ev in events.read() {
        // Trigger a global recalculation of paths and trade networks
        // If a route cannot be reformed, emit a Starvation/Shortage event for affected colonies
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};
    use bevy_time::TimePlugin;
    use std::time::Duration;

    #[test]
    fn test_map_data_rots_over_time() {
        let mut app = App::new();
        app.add_plugins(TimePlugin);

        let mut map = MapData::new();
        map.insert_sector(
            SectorId(1),
            SectorData {
                last_scanned: 0,
                accuracy: 1.0,
            },
        );
        app.insert_resource(map);

        app.add_systems(Update, map_data_rot_system);

        // Initial tick to set up time
        app.update();

        // Advance time
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(Duration::from_secs(100));

        // Tick to run systems
        app.update();

        let map = app.world().resource::<MapData>();
        assert!(map.get_sector(SectorId(1)).unwrap().accuracy < 1.0);
    }

    #[test]
    fn test_scout_ship_refreshes_map_data() {
        let mut app = App::new();
        let mut map = MapData::new();
        map.insert_sector(
            SectorId(1),
            SectorData {
                last_scanned: 0,
                accuracy: 0.5,
            },
        );
        app.insert_resource(map);

        app.world_mut()
            .spawn((ScoutShip, SectorPosition(SectorId(1))));

        app.add_systems(Update, scout_ship_scan_system);
        app.update();

        let map = app.world().resource::<MapData>();
        assert_eq!(map.get_sector(SectorId(1)).unwrap().accuracy, 1.0);
    }

    #[test]
    fn test_fleet_arrival_on_inaccurate_map_triggers_anomaly() {
        let mut app = App::new();
        app.add_event::<FleetArrivalEvent>();
        app.add_event::<AnomalyDiscoveredEvent>();

        let mut map = MapData::new();
        map.insert_sector(
            SectorId(1),
            SectorData {
                last_scanned: 0,
                accuracy: 0.2,
            },
        );
        app.insert_resource(map);

        app.add_systems(Update, fleet_arrival_anomaly_system);

        app.world_mut()
            .resource_mut::<Events<FleetArrivalEvent>>()
            .send(FleetArrivalEvent {
                fleet: Entity::PLACEHOLDER,
                sector: SectorId(1),
            });

        app.update();

        let anomalies = app.world().resource::<Events<AnomalyDiscoveredEvent>>();
        let mut reader = anomalies.get_cursor();
        assert!(reader.read(anomalies).count() > 0);
    }

    fn setup_app() -> App {
        use crate::shared::time::SimulationTime;
        let mut app = App::new();
        app.insert_resource(SimulationTime::default());
        app.add_event::<HyperlaneCollapseEvent>();
        app.add_event::<TradeRouteSeveredEvent>();
        app.add_systems(
            Update,
            (
                trigger_hyperlane_collapse_system,
                process_hyperlane_collapse_system,
                recalculate_trade_routes_system,
            ),
        );
        app
    }

    #[test]
    fn test_hyperlane_collapse_severs_connection() {
        let mut app = setup_app();

        let sys_a = app.world_mut().spawn(StarSystem { id: 1 }).id();
        let sys_b = app.world_mut().spawn(StarSystem { id: 2 }).id();

        // Spawn a hyperlane connecting A and B
        let lane = app
            .world_mut()
            .spawn(Hyperlane {
                start: sys_a,
                end: sys_b,
                stability: 100.0,
            })
            .id();

        // Trigger a collapse event
        app.world_mut()
            .resource_mut::<Events<HyperlaneCollapseEvent>>()
            .send(HyperlaneCollapseEvent { lane_entity: lane });

        app.update();

        // The hyperlane should be despawned or marked as collapsed
        assert!(
            app.world().get::<Hyperlane>(lane).is_none(),
            "Hyperlane should be destroyed after a collapse"
        );
    }

    #[test]
    fn test_fleet_pathfinding_fails_when_lane_collapses() {
        let mut app = setup_app();

        let sys_a = app.world_mut().spawn(StarSystem { id: 1 }).id();
        let sys_b = app.world_mut().spawn(StarSystem { id: 2 }).id();

        let lane = app
            .world_mut()
            .spawn(Hyperlane {
                start: sys_a,
                end: sys_b,
                stability: 0.0, // trigger system requires stability <= 0.0
            })
            .id();

        app.update();

        // Need an extra update tick to process the despawn commands that were
        // queued during process_hyperlane_collapse_system.
        app.update();

        // After update, recalculate_trade_routes_system or similar should run
        assert!(app.world().get::<Hyperlane>(lane).is_none());
        let events = app.world().resource::<Events<TradeRouteSeveredEvent>>();
        let mut reader = events.get_cursor();
        assert!(reader.read(events).count() > 0);
    }
}
