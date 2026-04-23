# Specification: 1143 Pulsar Timing

## 1. Overview
**What:** Introduce Pulsars as celestial objects in Layer 2 (System Map) that emit a rotating beam of radiation.
**Why:** To create timing-based tension for ship movement and colony placement. The "lighthouse of death" creates tactical movement puzzles.

## 2. Dependencies
- Layer 2 system map/coordinate system
- Ship entity movement mechanics
- Entity damage/health system

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_pulsar_beam_rotation() {
        let mut world = World::new();
        // Arrange
        let pulsar = world.spawn((
            Pulsar {
                rotation_period: 10.0,
                beam_width: std::f32::consts::PI / 4.0, // 45 degrees
            },
            Facing(0.0), // Starts pointing at 0 radians
        )).id();

        // Act - advance time by 2.5 seconds (1/4 period)
        world.insert_resource(SimulationTime { delta_seconds: 2.5, ..Default::default() });
        world.run_system_once(pulsar_rotation_system);

        // Assert - should rotate by PI/2 radians (90 degrees)
        let facing = world.get::<Facing>(pulsar).unwrap();
        assert!((facing.0 - std::f32::consts::PI / 2.0).abs() < 0.001);
    }

    #[test]
    fn test_ship_takes_damage_in_beam() {
        let mut world = World::new();
        // Arrange
        let pulsar = world.spawn((
            Pulsar {
                rotation_period: 10.0,
                beam_width: std::f32::consts::PI / 4.0,
            },
            Position { x: 0.0, y: 0.0 },
            Facing(0.0), // Pointing right
        )).id();

        let ship_in_beam = world.spawn((
            Ship,
            Position { x: 10.0, y: 0.0 }, // Right
            Health(100.0),
        )).id();

        let ship_out_beam = world.spawn((
            Ship,
            Position { x: 0.0, y: 10.0 }, // Up
            Health(100.0),
        )).id();

        // Act
        world.insert_resource(SimulationTime { delta_seconds: 1.0, ..Default::default() });
        world.run_system_once(pulsar_radiation_damage_system);

        // Assert
        let health_in = world.get::<Health>(ship_in_beam).unwrap();
        assert!(health_in.0 < 100.0, "Ship in beam should take damage");

        let health_out = world.get::<Health>(ship_out_beam).unwrap();
        assert_eq!(health_out.0, 100.0, "Ship out of beam should not take damage");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation to satisfy the tests
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Pulsar {
    pub rotation_period: f32, // Seconds for a full 360 degree rotation
    pub beam_width: f32,      // Radians
}

#[derive(Component)]
pub struct Facing(pub f32); // Radians

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct Ship;

#[derive(Resource, Default)]
pub struct SimulationTime {
    pub delta_seconds: f32,
}

pub fn pulsar_rotation_system(
    time: Res<SimulationTime>,
    mut query: Query<(&Pulsar, &mut Facing)>,
) {
    for (pulsar, mut facing) in query.iter_mut() {
        let rotation_speed = (2.0 * std::f32::consts::PI) / pulsar.rotation_period;
        facing.0 += rotation_speed * time.delta_seconds;
        // Normalize to 0..2PI
        facing.0 %= 2.0 * std::f32::consts::PI;
    }
}

pub fn pulsar_radiation_damage_system(
    time: Res<SimulationTime>,
    pulsars: Query<(&Position, &Facing, &Pulsar)>,
    mut ships: Query<(&Position, &mut Health), With<Ship>>,
) {
    let damage_per_second = 10.0;

    for (pulsar_pos, facing, pulsar) in pulsars.iter() {
        for (ship_pos, mut health) in ships.iter_mut() {
            let dx = ship_pos.x - pulsar_pos.x;
            let dy = ship_pos.y - pulsar_pos.y;
            let angle_to_ship = dy.atan2(dx);

            // Normalize angle relative to facing
            let mut angle_diff = (angle_to_ship - facing.0).abs();
            if angle_diff > std::f32::consts::PI {
                angle_diff = 2.0 * std::f32::consts::PI - angle_diff;
            }

            if angle_diff <= pulsar.beam_width / 2.0 {
                health.0 -= damage_per_second * time.delta_seconds;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells:** `Position` and `Health` might already exist in the codebase. Use existing generic components instead of creating new ones. Ensure angle wrapping logic is robust (using a helper function if one exists in math utils).
- **Performance:** If there are many ships, calculating atan2 for every ship against every pulsar could become expensive. Consider spatial partitioning or checking distance first before doing trig math.
- **API Improvements:** Extract the damage value to a configuration resource or a field on the `Pulsar` component itself to allow different pulsars to have different damage values.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pulsar rotation and beam damage logic verified.

## 7. Technical Guidance
- **Code Structure:** Place the systems in `src/layer2/celestial.rs` or similar, depending on the current Layer 2 directory structure.
- **Integration Points:** Ensure these systems are added to the simulation schedule in `src/simulation.rs` or `src/layer2/mod.rs`.
- **Gotchas:** Be careful with floating point math and radian wrapping.

## 8. Questions
*Builder: add questions here if spec is unclear.*
