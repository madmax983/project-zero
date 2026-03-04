# 283: Acoustic Zones

## 1. Overview
The colony is noisy. Machines emit `Noise` that spreads across the grid, attenuating over distance. High noise reduces Sleep quality and increases Stress, creating tension between building efficient, compact housing near industrial zones versus spreading things out for quality of life.

## 2. Dependencies
- `005` Pop needs (hunger, rest)
- `127` Stress Breakdowns
- `064` Room Quality

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::StressTracker;
    use crate::layer1::map::GridMap;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(GridMap::new(50, 50));
        app.insert_resource(AcousticMap::new(50, 50));
        app.add_systems(Update, (propagate_noise_system, apply_noise_stress_system));
        app
    }

    #[test]
    fn test_noise_propagates_from_source() {
        let mut app = setup_app();

        // Spawn a loud machine
        app.world_mut().spawn((
            NoiseEmitter { volume: 10.0 },
            GridPosition { x: 10, y: 10 },
        ));

        app.update();

        let acoustic_map = app.world().get_resource::<AcousticMap>().unwrap();

        // Epicenter should be loud
        assert_eq!(acoustic_map.get(10, 10), 10.0);
        // Adjacent tiles should be quieter but not 0
        assert!(acoustic_map.get(11, 10) > 0.0);
        assert!(acoustic_map.get(11, 10) < 10.0);
    }

    #[test]
    fn test_noise_increases_pop_stress() {
        let mut app = setup_app();

        // Set background noise high
        app.world_mut().resource_mut::<AcousticMap>().set(5, 5, 8.0);

        let pop_id = app.world_mut().spawn((
            GridPosition { x: 5, y: 5 },
            StressTracker { accumulated_stress: 0.0, max: 100.0 },
            Sleeping, // Noise affects sleepers more
        )).id();

        app.update();

        let stress = app.world().get::<StressTracker>(pop_id).unwrap();
        assert!(stress.accumulated_stress > 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::needs::StressTracker;

#[derive(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct NoiseEmitter {
    pub volume: f32,
}

#[derive(Component)]
pub struct Sleeping;

#[derive(Resource)]
pub struct AcousticMap {
    width: i32,
    height: i32,
    grid: Vec<f32>,
}

impl AcousticMap {
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height, grid: vec![0.0; (width * height) as usize] }
    }

    pub fn get(&self, x: i32, y: i32) -> f32 {
        if x < 0 || x >= self.width || y < 0 || y >= self.height { return 0.0; }
        self.grid[(y * self.width + x) as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, val: f32) {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            self.grid[(y * self.width + x) as usize] = val;
        }
    }

    pub fn clear(&mut self) {
        self.grid.fill(0.0);
    }
}

pub fn propagate_noise_system(
    mut acoustic_map: ResMut<AcousticMap>,
    emitters: Query<(&GridPosition, &NoiseEmitter)>,
) {
    acoustic_map.clear();

    // Very naive flood fill for MVP
    for (pos, emitter) in emitters.iter() {
        let max_radius = emitter.volume as i32;

        for dx in -max_radius..=max_radius {
            for dy in -max_radius..=max_radius {
                let dist = ((dx*dx + dy*dy) as f32).sqrt();
                if dist <= emitter.volume {
                    let current = acoustic_map.get(pos.x + dx, pos.y + dy);
                    let new_val = current.max(emitter.volume - dist);
                    acoustic_map.set(pos.x + dx, pos.y + dy, new_val);
                }
            }
        }
    }
}

pub fn apply_noise_stress_system(
    acoustic_map: Res<AcousticMap>,
    mut pops: Query<(&GridPosition, &mut StressTracker, Option<&Sleeping>)>,
) {
    for (pos, mut stress, sleeping) in pops.iter_mut() {
        let local_noise = acoustic_map.get(pos.x, pos.y);
        if local_noise > 0.0 {
            let multiplier = if sleeping.is_some() { 2.0 } else { 1.0 };
            stress.accumulated_stress += (local_noise * 0.1 * multiplier);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- The noise propagation should be blocked or muffled by walls. A raycast or true flood-fill pathfinding algorithm considering tile density (Spec 258 Acoustic Shadows) is needed for realism.
- `AcousticMap` should be visualized in the UI via an overlay so the player can intentionally zone their city.
- Add `Deaf` traits or `Earplugs` equipment to negate these effects.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Entities with `NoiseEmitter` populate the `AcousticMap`.
- [ ] High noise on a Pop's tile increases their `StressTracker`, with an increased penalty if they are `Sleeping`.

## 7. Technical Guidance
- Integrate into `src/layer1/environment/acoustics.rs`.
- Make sure `propagate_noise_system` runs *before* `apply_noise_stress_system`.
- Hook into the UI layer to provide a "Noise Overlay" view mode.

## 8. Questions
*Builder: add questions here if spec is unclear.*
