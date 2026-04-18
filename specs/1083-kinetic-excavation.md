# 1083: Kinetic Excavation

## 1. Overview
Instead of digging slowly, you order a "Kinetic Strike" from a Layer 2 ship to expose deep ore veins on Layer 1. It creates a crater, destroys surface buildings/biomes, and exposes the resource instantly. However, if the target coordinates are slightly off, it might hit a magma layer instead of the ore vein, creating an active volcano in the middle of your base. This creates a tension between slow, safe mining vs fast, destructive terraforming.

## 2. Dependencies
- Layer 1 terrain, biome, and building systems.
- Layer 2 fleet/orbital strike mechanics.
- Subsurface resource generation (ore/magma layers).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_kinetic_strike_exposes_ore() {
        let mut app = App::new();
        app.add_systems(Update, kinetic_strike_system);

        let target_tile = app.world_mut().spawn((
            TerrainTile { kind: TerrainKind::Plains },
            SubsurfaceResource { kind: ResourceKind::OreVein, depth: 50.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        app.add_event::<KineticStrikeEvent>();
        app.world_mut().send_event(KineticStrikeEvent {
            target_x: 5,
            target_y: 5,
            accuracy_offset: 0.0,
        });

        app.update();

        let terrain = app.world().get::<TerrainTile>(target_tile).unwrap();
        // The strike should turn plains into an exposed ore crater
        assert_eq!(terrain.kind, TerrainKind::Crater(ResourceKind::OreVein));
    }

    #[test]
    fn test_kinetic_strike_misses_hits_magma() {
        let mut app = App::new();
        app.add_systems(Update, kinetic_strike_system);

        let _target_tile = app.world_mut().spawn((
            TerrainTile { kind: TerrainKind::Plains },
            SubsurfaceResource { kind: ResourceKind::OreVein, depth: 50.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        let adjacent_tile = app.world_mut().spawn((
            TerrainTile { kind: TerrainKind::Plains },
            SubsurfaceResource { kind: ResourceKind::Magma, depth: 40.0 },
            GridPosition { x: 6, y: 5 },
        )).id();

        app.add_event::<KineticStrikeEvent>();
        // Simulate a strike that missed by 1 unit on the X axis
        app.world_mut().send_event(KineticStrikeEvent {
            target_x: 6,
            target_y: 5,
            accuracy_offset: 1.0,
        });

        app.update();

        let terrain = app.world().get::<TerrainTile>(adjacent_tile).unwrap();
        // Missing and hitting magma creates a volcano
        assert_eq!(terrain.kind, TerrainKind::Volcano);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct TerrainTile {
    pub kind: TerrainKind,
}

#[derive(PartialEq, Debug, Clone)]
pub enum TerrainKind {
    Plains,
    Crater(ResourceKind),
    Volcano,
}

#[derive(PartialEq, Debug, Clone)]
pub enum ResourceKind {
    OreVein,
    Magma,
}

#[derive(Component)]
pub struct SubsurfaceResource {
    pub kind: ResourceKind,
    pub depth: f32,
}

#[derive(Component, PartialEq, Eq)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Event)]
pub struct KineticStrikeEvent {
    pub target_x: i32,
    pub target_y: i32,
    pub accuracy_offset: f32,
}

pub fn kinetic_strike_system(
    mut events: EventReader<KineticStrikeEvent>,
    mut tile_query: Query<(&mut TerrainTile, &SubsurfaceResource, &GridPosition)>,
) {
    for event in events.read() {
        // In this minimal implementation, the event target coordinates already account for the offset
        let actual_hit_x = event.target_x;
        let actual_hit_y = event.target_y;

        for (mut tile, resource, pos) in tile_query.iter_mut() {
            if pos.x == actual_hit_x && pos.y == actual_hit_y {
                match resource.kind {
                    ResourceKind::OreVein => {
                        tile.kind = TerrainKind::Crater(ResourceKind::OreVein);
                    }
                    ResourceKind::Magma => {
                        tile.kind = TerrainKind::Volcano;
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: The current target resolution relies on exact coordinates. An actual kinetic strike should probably have a radius effect, destroying surrounding tiles as well.
- **Performance**: Grid lookup using iteration is slow. We should use a spatial hash map or the central `TerrainGrid` resource for fast tile lookup.
- **Design Improvements**: Add logic to destroy any `Building` components on the affected tiles. Trigger a `ChronicleEvent` for "The Colony Called Down the Thunder" or similar.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Kinetic strike on an ore vein creates an Ore Crater.
- [ ] Kinetic strike missing and hitting magma creates a Volcano.
- [ ] Surface buildings/entities in the blast radius are destroyed.

## 7. Technical Guidance
- Connect `KineticStrikeEvent` generation to the UI of Layer 2 ships capable of orbital bombardment.
- Make sure to apply the accuracy offset logic during event generation based on the ship's stats or weather conditions.

## 8. Questions
*Builder: add questions here if spec is unclear.*
