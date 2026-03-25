# 585: The Cartographic Delusion

## 1. Overview
The map is not the territory. Long-range scanners provide the Layer 3 map, but the data slowly "rots" if not regularly updated by scout ships. Sometimes, cosmic anomalies or enemy jamming artificially alter the map data. If you send colony ships or fleets based on unverified, old data, they might arrive at completely different reality. The tension is the extreme time and resource cost of constantly sending physical scouts to verify map data vs. the existential risk of making grand strategic decisions based on a ghost map. Layer: Cross-layer (3 -> 2 -> 1).

## 2. Dependencies
- Layer 3 Map and Scanner systems
- Fleet movement systems
- Event generation for anomalies

## 3. RED Phase: Tests First
```rust
#[test]
fn test_map_data_rots_over_time() {
    let mut world = World::new();
    world.insert_resource(Time::new(Duration::from_secs(0)));
    let mut map = MapData::new();
    map.insert_sector(SectorId(1), SectorData { last_scanned: 0, accuracy: 1.0 });
    world.insert_resource(map);

    // Advance time by 100 turns
    world.resource_mut::<Time>().advance_by(Duration::from_secs(100));
    map_data_rot_system(&mut world);

    let map = world.resource::<MapData>();
    assert!(map.get_sector(SectorId(1)).unwrap().accuracy < 1.0);
}

#[test]
fn test_scout_ship_refreshes_map_data() {
    let mut world = World::new();
    let mut map = MapData::new();
    map.insert_sector(SectorId(1), SectorData { last_scanned: 0, accuracy: 0.5 });
    world.insert_resource(map);
    world.spawn((ScoutShip, SectorPosition(SectorId(1))));

    scout_ship_scan_system(&mut world);

    let map = world.resource::<MapData>();
    assert_eq!(map.get_sector(SectorId(1)).unwrap().accuracy, 1.0);
}

#[test]
fn test_fleet_arrival_on_inaccurate_map_triggers_anomaly() {
    let mut world = World::new();
    let mut events = Events::<FleetArrivalEvent>::default();
    world.insert_resource(events);
    let mut anomalies = Events::<AnomalyDiscoveredEvent>::default();
    world.insert_resource(anomalies);

    let mut map = MapData::new();
    map.insert_sector(SectorId(1), SectorData { last_scanned: 0, accuracy: 0.2 });
    world.insert_resource(map);

    world.resource_mut::<Events<FleetArrivalEvent>>().send(FleetArrivalEvent { fleet: Entity::PLACEHOLDER, sector: SectorId(1) });

    fleet_arrival_anomaly_system(&mut world);

    let anomalies = world.resource::<Events<AnomalyDiscoveredEvent>>();
    let mut reader = anomalies.get_reader();
    assert!(reader.iter(&anomalies).count() > 0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal implementation to pass the tests.
pub struct MapData { ... }
pub fn map_data_rot_system(world: &mut World) { ... }
pub fn scout_ship_scan_system(world: &mut World) { ... }
pub fn fleet_arrival_anomaly_system(world: &mut World) { ... }
```

## 5. REFACTOR Phase: Quality & Design
- Extract the map rot logic into a customizable trait or function for easy tuning.
- Ensure the anomaly event generation is plugged into the main event dispatcher.
- Optimize the scan system to only query active scout ships in relevant sectors.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified: map accuracy degrades over time and causes anomalies on arrival.

## 7. Technical Guidance
- Implement map rotting gracefully using a linear or exponential decay formula.
- Anomaly triggers should consider the delta between perceived state and actual state.

## 8. Questions
*Builder: add questions here if spec is unclear.*
