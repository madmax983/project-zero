# 505 - Tectonic Extraction

## 1. Overview
Dragging entire mountains into space. Massive "Tether Extractors" from Layer 2 ships anchor into Layer 1 terrain. They literally rip chunks of the crust off the planet to harvest raw materials in bulk. This leaves massive, un-buildable "Crater" tiles and causes localized earthquakes. Tension: Incredibly fast, massive resource gathering vs. permanent destruction of the colony's buildable area and geological stability.

## 2. Dependencies
- `153` Geological Instability
- `018` Mining Resources
- `099` Fleet Movement

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;
    // Mock imports
    // use crate::layer1::terrain::{TerrainTile, TerrainType};
    // use crate::layer2::fleet::{Fleet, Extracting};

    #[derive(Component)]
    struct TerrainTile {
        pub position: (i32, i32),
        pub terrain_type: String, // "Mountain", "Crater"
    }

    #[derive(Component)]
    struct Extracting {
        pub target: (i32, i32),
        pub progress: f32, // 0 to 100
    }

    fn tectonic_extraction_system(
        mut commands: Commands,
        mut tiles: Query<(Entity, &mut TerrainTile)>,
        mut extractors: Query<(Entity, &mut Extracting)>,
    ) {
        // Implementation omitted for RED phase. Should fail tests.
    }

    #[test]
    fn test_extraction_completes_and_leaves_crater() {
        let mut app = App::new();

        app.world_mut().spawn(TerrainTile { position: (5, 5), terrain_type: "Mountain".to_string() });
        app.world_mut().spawn(Extracting { target: (5, 5), progress: 90.0 });

        app.add_systems(Update, tectonic_extraction_system);
        app.update();

        // Find the tile and check its type
        let mut crater_found = false;
        for tile in app.world().query::<&TerrainTile>().iter(app.world()) {
            if tile.position == (5, 5) {
                assert_eq!(tile.terrain_type, "Crater");
                crater_found = true;
            }
        }
        assert!(crater_found, "Crater should be created at target position");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct TerrainTile {
    pub position: (i32, i32),
    pub terrain_type: String,
}

#[derive(Component)]
pub struct Extracting {
    pub target: (i32, i32),
    pub progress: f32,
}

pub fn tectonic_extraction_system(
    mut commands: Commands,
    mut tiles: Query<&mut TerrainTile>,
    mut extractors: Query<(Entity, &mut Extracting)>,
) {
    for mut extractor in extractors.iter_mut() {
        extractor.progress += 10.0; // Arbitrary extraction speed
        if extractor.progress >= 100.0 {
            for mut tile in tiles.iter_mut() {
                if tile.position == extractor.target {
                    if tile.terrain_type == "Mountain" {
                        tile.terrain_type = "Crater".to_string();
                        // TODO: Add resources to inventory, trigger earthquake
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Dispatching**: When `progress >= 100.0`, emit a `TectonicExtractionComplete` event, which a resource system handles (giving resources) and a disaster system handles (creating the earthquake).
- **Scale Issues**: "Chunks of crust" implies an AoE, not a single tile. The `target` should be a radius, and the crater should be massive.
- **Resource Rewards**: Hook into `018 Mining Resources` to dynamically calculate the reward based on the value of the terrain consumed.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Layer 2 extractors converting Layer 1 terrain to "Crater".
- [ ] Extraction takes time (progress).

## 7. Technical Guidance
- Make sure `Extracting` cleans itself up or goes dormant once `progress >= 100.0`.
- Emit an `EarthquakeEvent` centered on the target position with magnitude scaling based on extraction volume.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
