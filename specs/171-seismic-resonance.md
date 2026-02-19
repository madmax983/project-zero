# 171: Seismic Resonance

## Overview

Introduces "Seismic Resonance" (Vibration) as a distinct mechanic from Acoustic Noise. Heavy machinery (drills, stampers) generates vibrations that travel through the ground.

- **VibrationGrid**: A dynamic grid tracking seismic intensity.
- **SeismicSource**: Component for machines that shake the ground.
- **Propagation**: Unlike sound (blocked by rock), vibration travels *better* through hard terrain (Rock) and is dampened by soft terrain (Sand, Mud).
- **Consequences**:
    - **Flora Agitation**: `AntagonisticFlora` grows and attacks faster in high-vibration zones.
    - **Geological Instability**: High sustained vibration triggers `Earthquake` or `CaveIn` events (from Spec 153).

## Dependencies

- `153` — Geological Instability (Trigger target)
- `092` — Antagonistic Flora (Reaction target)
- `060` — Acoustic Simulation (Conceptually similar, but distinct logic)

## RED Phase: Tests First

Write these tests in `src/layer1/seismic_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::seismic::{VibrationGrid, SeismicSource, update_seismic_system, seismic_flora_reaction_system};
    use crate::layer1::flora::{Flora, FloraType};
    use crate::layer1::geology::GeologicalInstabilityEvent; // Assuming from Spec 153

    #[test]
    fn test_vibration_grid_initialization() {
        let grid = VibrationGrid::new(10, 10);
        assert_eq!(grid.width, 10);
        assert_eq!(grid.height, 10);
        assert_eq!(grid.get(0, 0), 0.0);
    }

    #[test]
    fn test_seismic_source_propagation_rock_vs_sand() {
        let mut world = World::new();
        let mut terrain = TerrainGrid::new(10, 10);

        // Row 0 is Rock (High transmission)
        for x in 0..10 { terrain.set(x, 0, TerrainType::Rock); }
        // Row 5 is Sand (Low transmission)
        for x in 0..10 { terrain.set(x, 5, TerrainType::Sand); }

        world.insert_resource(terrain);
        world.insert_resource(VibrationGrid::new(10, 10));

        // Source on Rock
        world.spawn((
            SeismicSource { intensity: 1.0, radius: 5.0 },
            GridPosition { x: 0, y: 0 },
        ));

        // Source on Sand
        world.spawn((
            SeismicSource { intensity: 1.0, radius: 5.0 },
            GridPosition { x: 0, y: 5 },
        ));

        update_seismic_system(&mut world);

        let grid = world.resource::<VibrationGrid>();

        // Rock should transmit further/stronger
        let rock_val = grid.get(3, 0);
        let sand_val = grid.get(3, 5);

        assert!(rock_val > sand_val, "Rock should transmit vibration better than Sand");
    }

    #[test]
    fn test_seismic_flora_agitation() {
        let mut world = World::new();
        let mut grid = VibrationGrid::new(10, 10);

        // Set high vibration at (5, 5)
        grid.set(5, 5, 0.8);
        world.insert_resource(grid);

        // Spawn Flora
        let flora_entity = world.spawn((
            Flora {
                growth_timer: 100,
                attack_timer: 100,
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Run reaction system
        seismic_flora_reaction_system(&mut world);

        let flora = world.get::<Flora>(flora_entity).unwrap();

        // Timers should decrease faster than normal tick (or be reduced directly)
        assert!(flora.growth_timer < 100);
        assert!(flora.attack_timer < 100);
    }

    #[test]
    fn test_seismic_triggers_instability() {
        let mut world = World::new();
        let mut grid = VibrationGrid::new(10, 10);
        world.insert_resource(Events::<GeologicalInstabilityEvent>::default());

        // Set EXTREME vibration at (5, 5)
        grid.set(5, 5, 1.5); // Over threshold
        world.insert_resource(grid);

        // Run system that checks for critical thresholds
        // (Assuming checking system is part of update or separate)
        // Let's assume `seismic_instability_system` handles this.
        crate::layer1::seismic::seismic_instability_system(&mut world);

        let events = world.resource::<Events<GeologicalInstabilityEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).len() > 0, "Should trigger instability event");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components and Resources

```rust
// src/layer1/seismic.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::{TerrainGrid, TerrainType};

#[derive(Resource)]
pub struct VibrationGrid {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
}

impl VibrationGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![0.0; width * height],
        }
    }

    pub fn get(&self, x: i32, y: i32) -> f32 {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return 0.0;
        }
        self.values[(y as usize) * self.width + (x as usize)]
    }

    pub fn set(&mut self, x: i32, y: i32, val: f32) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        self.values[(y as usize) * self.width + (x as usize)] = val;
    }
}

#[derive(Component)]
pub struct SeismicSource {
    pub intensity: f32,
    pub radius: f32,
}
```

### 2. Implement Propagation System

```rust
pub fn update_seismic_system(
    mut grid: ResMut<VibrationGrid>,
    terrain: Res<TerrainGrid>,
    sources: Query<(&SeismicSource, &GridPosition)>,
) {
    grid.values.fill(0.0);

    for (source, pos) in &sources {
        let r = source.radius.ceil() as i32;
        let cx = pos.x;
        let cy = pos.y;

        for dy in -r..=r {
            for dx in -r..=r {
                let dist_sq = (dx * dx + dy * dy) as f32;
                if dist_sq > source.radius * source.radius { continue; }

                let tx = cx + dx;
                let ty = cy + dy;

                // Simple transmission check (inverted damping)
                // We raycast or just check destination terrain for MVP
                // For Green phase, we'll just check destination terrain
                // Rock = 1.0 transmission, Sand = 0.5, Dirt = 0.7

                let transmission = if let Some(tile) = terrain.get(tx as usize, ty as usize) {
                    match tile {
                        TerrainType::Rock => 1.0,
                        TerrainType::Sand => 0.5,
                        _ => 0.7,
                    }
                } else {
                    0.0
                };

                let raw = source.intensity * (1.0 - (dist_sq.sqrt() / source.radius));
                let final_val = raw * transmission;

                if final_val > 0.0 {
                    let cur = grid.get(tx, ty);
                    grid.set(tx, ty, (cur + final_val).min(2.0)); // Cap at 2.0
                }
            }
        }
    }
}
```

### 3. Implement Flora Reaction

```rust
use crate::layer1::flora::Flora;

pub fn seismic_flora_reaction_system(
    grid: Res<VibrationGrid>,
    mut flora_query: Query<(&mut Flora, &GridPosition)>,
) {
    for (mut flora, pos) in &mut flora_query {
        let vibration = grid.get(pos.x, pos.y);
        if vibration > 0.1 {
            // Agitate: reduce timers
            // Logic: 0.1 vibration = 1 extra tick reduction
            let agitation = (vibration * 10.0) as u32;
            flora.growth_timer = flora.growth_timer.saturating_sub(agitation);
            flora.attack_timer = flora.attack_timer.saturating_sub(agitation);
        }
    }
}
```

### 4. Implement Instability Trigger

```rust
use crate::layer1::geology::GeologicalInstabilityEvent;

pub fn seismic_instability_system(
    grid: Res<VibrationGrid>,
    mut events: EventWriter<GeologicalInstabilityEvent>,
    // For MVP, we iterate grid or check source locations?
    // Iterating whole grid is O(N), acceptable for small maps.
    // Better: Random sampling or checking source vicinity.
) {
    // Check random tiles or critical spots
    // For MVP, just scan high values?
    // Or let sources trigger it locally.
    // Let's implement a global check for max vibration.

    // Simplification: Trigger if ANY tile > 1.2
    // To find where, we'd need to iterate.
    for y in 0..grid.height {
        for x in 0..grid.width {
            if grid.get(x as i32, y as i32) > 1.2 {
                // Chance to trigger
                if rand::random::<f32>() < 0.01 {
                    events.send(GeologicalInstabilityEvent {
                        x: x as i32,
                        y: y as i32,
                        severity: 1.0,
                    });
                }
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: `update_seismic_system` raycasting or flood fill for proper transmission through veins of rock.
- **Tuning**: Adjust `transmission` values. Maybe `Sand` should absorb vibration (dampen it quickly) while `Rock` carries it far.
- **Integration**: Add `SeismicSource` to `Drill` and `Generator` buildings.
- **Visuals**: Add "Screen Shake" effect if `Camera` is near high vibration.

## Acceptance Criteria

- [ ] `VibrationGrid` tracks intensity.
- [ ] `SeismicSource` propagates vibration.
- [ ] `Rock` transmits vibration better/further than `Sand`.
- [ ] `AntagonisticFlora` grows/attacks faster in high vibration.
- [ ] High vibration triggers `GeologicalInstabilityEvent`.
- [ ] All tests pass.

## Technical Guidance

- Use `saturating_sub` for timers to avoid panic.
- Ensure `seismic_instability_system` doesn't spam events every tick (add a cooldown or low probability).
