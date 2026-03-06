# 340 - Light & Darkness

## 1. Overview
The **Light & Darkness** feature transforms light from a purely visual effect into a core mechanic. Tiles now track a "Light Level". Pops working or moving in darkness suffer movement penalties and increased Stress. Certain hostile entities (like "Shadow Stalkers" or specific vermin) may only spawn or path through total darkness. This forces the player to actively manage their power grid and fuel supplies to keep the colony illuminated, or suffer severe operational penalties.

## 2. Dependencies
- `053` Lighting System
- `042` Energy System
- `127` Stress Breakdowns
- `002` Basic Map (TerrainGrid)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_pop_speed_reduced_in_darkness() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_darkness_penalties_system);

        let grid = app.world_mut().spawn(TerrainGrid::new(10, 10)).id();
        let dark_tile = app.world().get::<TerrainGrid>(grid).unwrap().get_tile(5, 5).unwrap();
        app.world_mut().entity_mut(dark_tile).insert(LightLevel { intensity: 0.0 });

        let pop = app.world_mut().spawn((
            Speed { current: 10.0, base: 10.0 },
            Needs { stress: 0.0 },
            Transform::from_xyz(5.0, 5.0, 0.0), // Standing on dark tile
        )).id();

        // Act
        app.update();

        // Assert
        let speed = app.world().get::<Speed>(pop).unwrap().current;
        assert!(speed < 10.0, "Pop speed should be reduced when in a dark tile.");
    }

    #[test]
    fn test_pop_stress_increases_in_darkness() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_darkness_penalties_system);

        let grid = app.world_mut().spawn(TerrainGrid::new(10, 10)).id();
        let dark_tile = app.world().get::<TerrainGrid>(grid).unwrap().get_tile(5, 5).unwrap();
        app.world_mut().entity_mut(dark_tile).insert(LightLevel { intensity: 0.0 });

        let pop = app.world_mut().spawn((
            Speed { current: 10.0, base: 10.0 },
            Needs { stress: 10.0 },
            Transform::from_xyz(5.0, 5.0, 0.0), // Standing on dark tile
        )).id();

        // Act
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(1));
        app.update();

        // Assert
        let stress = app.world().get::<Needs>(pop).unwrap().stress;
        assert!(stress > 10.0, "Pop stress should increase over time when in total darkness.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct LightLevel {
    pub intensity: f32, // 0.0 (Pitch Black) to 1.0 (Bright)
}

#[derive(Component)]
pub struct Speed {
    pub base: f32,
    pub current: f32,
}

#[derive(Component)]
pub struct Needs {
    pub stress: f32,
}

const DARKNESS_SPEED_MODIFIER: f32 = 0.5; // 50% slower in the dark
const DARKNESS_STRESS_RATE: f32 = 2.0;    // +2 stress per second in the dark

pub fn apply_darkness_penalties_system(
    time: Res<Time>,
    grid_query: Query<&TerrainGrid>,
    mut pops: Query<(&mut Speed, &mut Needs, &Transform)>,
) {
    let grid = match grid_query.get_single() {
        Ok(g) => g,
        Err(_) => return, // No grid yet
    };

    for (mut speed, mut needs, transform) in pops.iter_mut() {
        let x = transform.translation.x as i32; // Simplified coord logic
        let y = transform.translation.y as i32;

        // Find light level of current tile
        let mut current_light_intensity = 1.0; // Assume lit if outside grid
        if let Some(tile_entity) = grid.get_tile(x, y) {
            // Assume we can query the tile's LightLevel component directly
            // In a real implementation, you'd use the ECS world or a hashmap inside the grid
            if let Ok(light) = grid.world().get::<LightLevel>(tile_entity) {
                current_light_intensity = light.intensity;
            }
        }

        // Apply penalties if below a threshold (e.g., 0.2)
        if current_light_intensity < 0.2 {
            speed.current = speed.base * DARKNESS_SPEED_MODIFIER;
            needs.stress += DARKNESS_STRESS_RATE * time.delta_seconds();
        } else {
            speed.current = speed.base; // Restore speed if moving into light
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**:
  - The lighting implementation is tightly coupled to position. Extract "Lighting Status" into a component on the Pop (e.g., `InDarkness`) updated by an observation system, and have the execution system read that.
  - Make pops aware of light via the Utility AI. If a path passes through darkness, they should prefer a longer, lit route if the dark route causes too much stress.
- **Code Smells**:
  - Checking `grid.world().get` inside a system loop is invalid Bevy ECS design. We need a `Query<&LightLevel>` for tiles, or `TerrainGrid` needs to store the light value directly instead of an entity reference.
- **Performance**:
  - Ensure updating light values (which happens when lights turn on/off) is decoupled from reading them (which happens every tick for speed/stress). The `LightGrid` from `053` should be a resource.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops in tiles with light intensity < 0.2 suffer a significant speed penalty and gain stress over time.

## 7. Technical Guidance
- Integrate with `src/layer1/needs.rs` for `Stress` and the existing movement execution system for `Speed.current`.
- Verify the existing Lighting System (`053`) propagates light correctly and exposes an accessible grid or component system for querying light levels at specific X,Y coordinates.

## 8. Questions
*Builder: add questions here if spec is unclear.*
