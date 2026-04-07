# Feature Specification: Window Views (847)

## 1. Overview
**Layer**: 1 (Colony Layer)
**Fantasy**: A room with a view. The psychological impact of architecture.
**Mechanic**: Room "Beauty" calculation includes raycasting out of windows. Seeing Nature/Sky = Good. Seeing a Brick Wall/Factory = Bad.
**Emergence**: You accidentally block the Governor's view of the mountains with a new smokestack. He gets depressed and starts passing draconian laws.
**Tension**: Density (Efficiency) vs. Aesthetics/Mental Health.

## 2. Dependencies
- `Building System` (Rooms and Windows)
- `Grid System` (Spatial raycasting)
- `Pop Mood System` (Beauty affects mood)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // Dummy structs for compilation
    #[derive(Component)]
    struct Window {
        direction: Vec2, // Normal vector facing out
    }

    #[derive(Component)]
    struct Room {
        base_beauty: f32,
        total_beauty: f32,
    }

    #[derive(Component, PartialEq, Clone, Copy)]
    enum TileType {
        Nature,
        Sky,
        Factory,
        BrickWall,
        Empty,
    }

    #[derive(Component)]
    struct GridPosition(i32, i32);

    #[derive(Resource)]
    struct MapGrid {
        tiles: Vec<Vec<TileType>>, // Simplified grid for raycasting
    }

    #[test]
    fn test_window_viewing_nature_increases_beauty() {
        // Arrange
        let mut app = App::new();
        let grid = MapGrid {
            tiles: vec![
                vec![TileType::Empty, TileType::Nature, TileType::Nature],
            ]
        };
        app.insert_resource(grid);
        app.add_systems(Update, calculate_room_beauty);

        let window = app.world_mut().spawn((
            Window { direction: Vec2::new(1.0, 0.0) },
            GridPosition(0, 0)
        )).id();

        let room = app.world_mut().spawn((
            Room { base_beauty: 10.0, total_beauty: 0.0 },
        )).add_child(window).id();

        // Act
        app.update();

        // Assert
        let r = app.world().get::<Room>(room).unwrap();
        assert!(r.total_beauty > 10.0); // Beauty increased
    }

    #[test]
    fn test_window_viewing_factory_decreases_beauty() {
        // Arrange
        let mut app = App::new();
        let grid = MapGrid {
            tiles: vec![
                vec![TileType::Empty, TileType::Factory, TileType::Sky],
            ]
        };
        app.insert_resource(grid);
        app.add_systems(Update, calculate_room_beauty);

        let window = app.world_mut().spawn((
            Window { direction: Vec2::new(1.0, 0.0) },
            GridPosition(0, 0)
        )).id();

        let room = app.world_mut().spawn((
            Room { base_beauty: 10.0, total_beauty: 0.0 },
        )).add_child(window).id();

        // Act
        app.update();

        // Assert
        let r = app.world().get::<Room>(room).unwrap();
        assert!(r.total_beauty < 10.0); // Beauty decreased
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Window {
    pub direction: Vec2,
}

#[derive(Component)]
pub struct Room {
    pub base_beauty: f32,
    pub total_beauty: f32,
}

#[derive(Component, PartialEq, Clone, Copy)]
pub enum TileType {
    Nature,
    Sky,
    Factory,
    BrickWall,
    Empty,
}

#[derive(Component)]
pub struct GridPosition(pub i32, pub i32);

#[derive(Resource)]
pub struct MapGrid {
    pub tiles: Vec<Vec<TileType>>,
}

pub fn calculate_room_beauty(
    mut rooms: Query<(&mut Room, &Children)>,
    windows: Query<(&Window, &GridPosition)>,
    grid: Res<MapGrid>
) {
    for (mut room, children) in rooms.iter_mut() {
        let mut view_modifier = 0.0;

        for &child in children.iter() {
            if let Ok((window, pos)) = windows.get(child) {
                // Raycast in window direction (simplified for minimal impl)
                let mut current_x = pos.0;
                let mut current_y = pos.1;

                // Raycast up to 3 tiles away
                for _ in 0..3 {
                    current_x += window.direction.x as i32;
                    current_y += window.direction.y as i32;

                    if current_y >= 0 && (current_y as usize) < grid.tiles.len() &&
                       current_x >= 0 && (current_x as usize) < grid.tiles[current_y as usize].len() {

                        let tile = grid.tiles[current_y as usize][current_x as usize];
                        match tile {
                            TileType::Nature | TileType::Sky => {
                                view_modifier += 5.0;
                                break; // Sightline terminates on good view
                            },
                            TileType::Factory | TileType::BrickWall => {
                                view_modifier -= 5.0;
                                break; // Sightline terminates on bad view
                            },
                            TileType::Empty => {
                                // Keep raycasting
                            }
                        }
                    } else {
                        break; // Out of bounds
                    }
                }
            }
        }

        room.total_beauty = room.base_beauty + view_modifier;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smell**: Hardcoded `Vec<Vec<TileType>>` raycasting.
- **Improvement**: Use a proper Bresenham's line algorithm on the actual `Map` grid structure.
- **API Change**: `TileType` beauty scores should be configurable components (e.g. `BeautyEmitter { score: 5.0, range: 10 }`), not hardcoded enums.
- **Optimization**: Only recalculate window views when a tile in the window's view cone changes, rather than every frame.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Nature views increase room beauty
- [ ] Industrial views decrease room beauty
- [ ] Empty tiles are skipped during raycasting until an object is hit

## 7. Technical Guidance
- Integrate with `src/layer1/building/room.rs`.
- Look at `bevy_math` for any useful ray intersection utilities if adapting to 3D/continuous space, though grid-based DDA (Digital Differential Analyzer) is better for integer grids.
- Keep the max raycast distance relatively short (e.g., 5-10 tiles) to prevent performance issues.

## 8. Questions
*Builder: add questions here if spec is unclear.*
