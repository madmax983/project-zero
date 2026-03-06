# Spec 356: Orbital Crossfire

## 1. Overview
Warring Layer 3 fleets in orbit may miss shots or drop debris. Impact craters destroy tiles/buildings on Layer 1. Debris can be harvested for high-tech scrap. This creates an unpredictable hazard from above that players must adapt to.

## 2. Dependencies
- `002-terrain-grid.md` (for Map / Grid positions)
- `006-building-placement.md` (for Buildings)
- `018-mining-resources.md` (for Resource nodes / Harvesting)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::{Map, TileType};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::resource::{ResourceNode, ResourceType};

    #[test]
    fn test_orbital_strike_damages_terrain() {
        let mut app = App::new();
        app.add_event::<OrbitalStrikeEvent>();
        app.add_systems(Update, process_orbital_strikes_system);

        let mut map = Map::new(10, 10);
        map.set_tile(5, 5, TileType::Plains);
        app.insert_resource(map);

        // Fire strike
        app.world_mut().send_event(OrbitalStrikeEvent { x: 5, y: 5 });
        app.update();

        // Terrain should be changed to crater
        let updated_map = app.world().get_resource::<Map>().unwrap();
        assert_eq!(updated_map.get_tile(5, 5), Some(TileType::Crater));
    }

    #[test]
    fn test_orbital_strike_destroys_buildings() {
        let mut app = App::new();
        app.add_event::<OrbitalStrikeEvent>();
        app.add_systems(Update, process_orbital_strikes_system);

        let mut map = Map::new(10, 10);
        app.insert_resource(map);

        let building_ent = app.world_mut().spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 5, y: 5 }
        )).id();

        app.world_mut().send_event(OrbitalStrikeEvent { x: 5, y: 5 });
        app.update();

        // Building should be destroyed
        assert!(app.world().get_entity(building_ent).is_err());
    }

    #[test]
    fn test_orbital_strike_spawns_harvestable_debris() {
        let mut app = App::new();
        app.add_event::<OrbitalStrikeEvent>();
        app.add_systems(Update, process_orbital_strikes_system);

        let mut map = Map::new(10, 10);
        app.insert_resource(map);

        app.world_mut().send_event(OrbitalStrikeEvent { x: 5, y: 5 });
        app.update();

        // Should spawn HighTechScrap resource node
        let mut query = app.world_mut().query::<(&GridPosition, &ResourceNode)>();
        let mut found = false;
        for (pos, node) in query.iter(app.world()) {
            if pos.x == 5 && pos.y == 5 && node.resource_type == ResourceType::HighTechScrap {
                found = true;
                break;
            }
        }
        assert!(found, "Harvestable debris was not spawned at the impact site.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::map::{Map, TileType};
use crate::layer1::building::Building;
use crate::layer1::resource::{ResourceNode, ResourceType};
use crate::layer1::grid::GridPosition;

#[derive(Event)]
pub struct OrbitalStrikeEvent {
    pub x: i32,
    pub y: i32,
}

pub fn process_orbital_strikes_system(
    mut commands: Commands,
    mut events: EventReader<OrbitalStrikeEvent>,
    mut map: ResMut<Map>,
    building_query: Query<(Entity, &GridPosition), With<Building>>,
) {
    for event in events.read() {
        // Change terrain
        if map.is_in_bounds(event.x, event.y) {
            map.set_tile(event.x, event.y, TileType::Crater);
        }

        // Destroy buildings at location
        for (entity, pos) in building_query.iter() {
            if pos.x == event.x && pos.y == event.y {
                commands.entity(entity).despawn_recursive();
            }
        }

        // Spawn debris
        commands.spawn((
            ResourceNode {
                resource_type: ResourceType::HighTechScrap,
                amount: 100,
            },
            GridPosition { x: event.x, y: event.y },
        ));
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Batching:** Process multiple strikes in a single pass efficiently.
- **VFX/SFX Triggers:** `OrbitalStrikeEvent` could also be listened to by the audio/visual systems to spawn explosion effects and camera shake.
- **Configurable Debris:** Extract `amount: 100` into a configuration or randomize it slightly for varied drops.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] An `OrbitalStrikeEvent` changes the tile at the coordinates to a crater.
- [ ] An `OrbitalStrikeEvent` destroys any building at the target coordinates.
- [ ] An `OrbitalStrikeEvent` spawns a `HighTechScrap` resource node at the target coordinates.

## 7. Technical Guidance
- `process_orbital_strikes_system` should be registered in the `Layer1SystemSet::Execution` set.
- Ensure `HighTechScrap` is properly defined in the `ResourceType` enum.
- Ensure `Crater` is properly defined in `TileType` enum.

## 8. Questions
*Builder: add questions here if spec is unclear.*
