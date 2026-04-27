# Invasive Bureaucracy

## 1. Overview
**Layer:** Cross-layer (1, 3)
**Fantasy:** The paperwork is literally expanding and consuming the colony.
**Mechanic:** As your empire's administrative complexity grows (Layer 3), "Bureaucracy Nodes" must be built on Layer 1 colonies to process data. These buildings are unique: they slowly, physically expand by automatically claiming adjacent tiles and converting them into filing rooms and server banks, destroying whatever was there previously. They provide massive empire-wide stability buffs but consume local land.

## 2. Dependencies
- Layer 1 Building System
- Grid/Tile Map System
- Layer 3 Empire Stability System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_bureaucracy_node_expansion() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<TileGrid>();

        let node_entity = app.world_mut().spawn((
            BureaucracyNode { expansion_timer: Timer::from_seconds(1.0, TimerMode::Once) },
            GridPosition { x: 5, y: 5 }
        )).id();

        app.world_mut().resource_mut::<TileGrid>().set_tile(5, 5, TileType::Bureaucracy);
        app.world_mut().resource_mut::<TileGrid>().set_tile(5, 6, TileType::Farm);

        // Act
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(2));
        app.add_systems(Update, expand_bureaucracy_nodes_system);
        app.update();

        // Assert
        let grid = app.world().resource::<TileGrid>();
        // Assuming it expands predictably to 5,6 for this test
        assert_eq!(grid.get_tile(5, 6), Some(TileType::Bureaucracy), "The node should have consumed the adjacent farm");
    }

    #[test]
    fn test_bureaucracy_provides_stability() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<EmpireStability>();
        app.world_mut().spawn((BureaucracyNode::default(), GridPosition { x: 0, y: 0 }));

        // Act
        app.add_systems(Update, calculate_bureaucracy_stability_system);
        app.update();

        // Assert
        let stability = app.world().resource::<EmpireStability>();
        assert!(stability.value > 0.0, "Bureaucracy node should provide empire stability");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TileType {
    Empty,
    Farm,
    Bureaucracy,
}

#[derive(Resource, Default)]
pub struct TileGrid {
    pub tiles: HashMap<(i32, i32), TileType>,
}

impl TileGrid {
    pub fn set_tile(&mut self, x: i32, y: i32, tile: TileType) {
        self.tiles.insert((x, y), tile);
    }

    pub fn get_tile(&self, x: i32, y: i32) -> Option<TileType> {
        self.tiles.get(&(x, y)).copied()
    }
}

#[derive(Component)]
pub struct BureaucracyNode {
    pub expansion_timer: Timer,
}

impl Default for BureaucracyNode {
    fn default() -> Self {
        Self {
            expansion_timer: Timer::from_seconds(10.0, TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Resource, Default)]
pub struct EmpireStability {
    pub value: f32,
}

pub fn expand_bureaucracy_nodes_system(
    time: Res<Time>,
    mut query: Query<(&mut BureaucracyNode, &GridPosition)>,
    mut grid: ResMut<TileGrid>,
) {
    for (mut node, pos) in query.iter_mut() {
        node.expansion_timer.tick(time.delta());
        if node.expansion_timer.just_finished() {
            // Simplistic expansion logic: just overwrite the tile above
            grid.set_tile(pos.x, pos.y + 1, TileType::Bureaucracy);
        }
    }
}

pub fn calculate_bureaucracy_stability_system(
    query: Query<&BureaucracyNode>,
    mut stability: ResMut<EmpireStability>,
) {
    let node_count = query.iter().count() as f32;
    stability.value = node_count * 10.0;
}
```

## 5. REFACTOR Phase: Quality & Design
- Implement proper pathfinding/flood-fill for expansion rather than hardcoding `y + 1`.
- Ensure any buildings consumed by the expansion trigger the correct despawn events and refund/penalty logic.
- Consider an "Admin Cap" mechanism on Layer 3 that dictates how aggressively the nodes expand.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Bureaucracy nodes physically spread to adjacent tiles over time.
- [ ] Existing structures on consumed tiles are overwritten.
- [ ] The total amount of bureaucracy tiles positively influences global stability.

## 7. Technical Guidance
- **Expansion Algorithm:** Keep it simple but non-deterministic if possible, perhaps randomly selecting one of the four cardinal adjacent tiles that isn't already a `BureaucracyNode`.
- **Despawning:** When a tile is overwritten, ensure you send a `BuildingDestroyedEvent` or equivalent so that pops working there are properly handled.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
