# 1019: Ecophagy

## 1. Overview
Ecophagy introduces "World Eater" ships at Layer 2 that can permanently strip-mine Layer 1 tiles, turning them into Bedrock/Void to generate massive raw resources instantly. This creates a severe consequence: by consuming the planet to build a fleet, players permanently alter the planetary geography, potentially destroying critical ecosystems, shifting weather patterns, or ruining their own farms to fuel a war engine.

## 2. Dependencies
- Layer 2 `Fleet` system (World Eater ship class).
- Layer 1 `TerrainGrid` (Tile destruction/modification).
- Layer 1 `Economy` (Resource generation).
- Layer 1 `Environment` (Weather/Atmosphere consequences).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::fleet::{WorldEater, FleetCommand};
    use crate::layer1::terrain::{TerrainGrid, TileType, GridPosition};
    use crate::layer1::economy::RawResources;

    #[test]
    fn test_world_eater_converts_tiles_to_bedrock() {
        let mut app = App::new();
        let mut grid = TerrainGrid::new(10, 10);
        grid.set_tile(5, 5, TileType::Mountain);
        app.insert_resource(grid);
        app.insert_resource(RawResources { amount: 0 });
        app.add_event::<FleetCommand>();
        app.add_systems(Update, process_world_eater_system);

        let eater = app.world_mut().spawn(WorldEater { efficiency: 100 }).id();

        app.world_mut().resource_mut::<Events<FleetCommand>>().send(FleetCommand::ConsumeTile {
            fleet: eater,
            target: GridPosition { x: 5, y: 5, z: 0 },
        });

        app.update();

        let grid = app.world().resource::<TerrainGrid>();
        assert_eq!(grid.get_tile(5, 5).unwrap(), &TileType::Bedrock, "World Eater should convert the target tile to Bedrock.");

        let resources = app.world().resource::<RawResources>();
        assert_eq!(resources.amount, 100, "World Eater should generate resources based on efficiency.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer2/ecophagy.rs
use bevy::prelude::*;
use crate::layer2::fleet::{WorldEater, FleetCommand};
use crate::layer1::terrain::{TerrainGrid, TileType, GridPosition};
use crate::layer1::economy::RawResources;

pub fn process_world_eater_system(
    mut events: EventReader<FleetCommand>,
    mut grid: ResMut<TerrainGrid>,
    mut resources: ResMut<RawResources>,
    query: Query<&WorldEater>,
) {
    for event in events.read() {
        if let FleetCommand::ConsumeTile { fleet, target } = event {
            if let Ok(eater) = query.get(*fleet) {
                // If the tile isn't already bedrock/void, consume it
                if let Some(tile) = grid.get_tile(target.x, target.y) {
                    if *tile != TileType::Bedrock {
                        grid.set_tile(target.x, target.y, TileType::Bedrock);
                        resources.amount += eater.efficiency;
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Yield Variance:** Different tiles should yield different resources (e.g., Mountain yields Minerals, Forest yields Organics) rather than a flat amount based on ship efficiency.
- **Ecological Impact:** Changing a tile to `Bedrock` must trigger updates in the `AtmosphereGrid` or pathfinding. For example, destroying a Mountain tile should change wind blocking properties.
- **Speed/Tick Rate:** A ship shouldn't consume a tile instantly in one frame. It should lock onto a tile and drain it over time, allowing the player to cancel if they realize they are eating their own food supply.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_world_eater_converts_tiles_to_bedrock` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- `FleetCommand` enum will need to be extended to support `ConsumeTile`.
- Ensure that any Layer 1 buildings on the consumed tile are gracefully destroyed or despawned when the terrain turns to Bedrock.

## 8. Questions
*Builder: add questions here if spec is unclear.*
