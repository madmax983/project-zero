# 1142 - Pulsar Timing

## 1. Overview
The "Pulsar Timing" feature introduces periodic, rotating beams of radiation from pulsars on the Layer 2 (System) map. Ships or colonies caught in the sweeping beam take damage or experience severe interference. Players must calculate and wait for the safe window to travel or burn extra fuel to cross the gap quickly, adding a new dimension of tension and timing to interstellar navigation.

## 2. Dependencies
- Layer 2 System map structure (`SystemNode`, `Fleet`, `Ship`).
- Ship movement mechanics (`FleetMovement` or equivalent).
- Health and damage mechanics for ships/colonies (`Health` component or equivalent).
- Simulation time or ticks resource (`SimulationTime`).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    // Mock components for the test
    #[derive(Component)]
    struct Fleet;

    #[derive(Component)]
    struct Position { x: f32, y: f32 }

    #[derive(Component)]
    struct Health(f32);

    #[derive(Component)]
    struct Pulsar {
        pub center_x: f32,
        pub center_y: f32,
        pub current_angle: f32,
        pub rotation_speed: f32,
        pub beam_width_rads: f32,
        pub damage_per_tick: f32,
    }

    fn setup_world() -> World {
        let mut world = World::new();
        // Assuming there is a time resource in the real game; we use a mock one here if needed
        world
    }

    #[test]
    fn test_pulsar_rotation_advances_angle() {
        let mut world = setup_world();
        let pulsar_entity = world.spawn(Pulsar {
            center_x: 0.0,
            center_y: 0.0,
            current_angle: 0.0,
            rotation_speed: 1.0, // radians per tick
            beam_width_rads: 0.5,
            damage_per_tick: 10.0,
        }).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(pulsar_rotation_system);

        schedule.run(&mut world);

        let pulsar = world.get::<Pulsar>(pulsar_entity).unwrap();
        assert_eq!(pulsar.current_angle, 1.0, "Pulsar angle should advance by rotation_speed");
    }

    #[test]
    fn test_fleet_caught_in_beam_takes_damage() {
        let mut world = setup_world();

        world.spawn(Pulsar {
            center_x: 0.0,
            center_y: 0.0,
            current_angle: std::f32::consts::PI / 4.0, // 45 degrees
            rotation_speed: 0.1,
            beam_width_rads: 0.2, // ~11 degrees wide
            damage_per_tick: 20.0,
        });

        // Fleet located at 45 degrees relative to pulsar (e.g., x=10, y=10)
        let fleet_entity = world.spawn((
            Fleet,
            Position { x: 10.0, y: 10.0 },
            Health(100.0)
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(pulsar_damage_system);

        schedule.run(&mut world);

        let health = world.get::<Health>(fleet_entity).unwrap();
        assert_eq!(health.0, 80.0, "Fleet should take 20 damage when caught in the beam");
    }

    #[test]
    fn test_fleet_outside_beam_takes_no_damage() {
        let mut world = setup_world();

        world.spawn(Pulsar {
            center_x: 0.0,
            center_y: 0.0,
            current_angle: 0.0, // pointing directly right
            rotation_speed: 0.1,
            beam_width_rads: 0.2,
            damage_per_tick: 20.0,
        });

        // Fleet located directly up (90 degrees), outside the beam
        let fleet_entity = world.spawn((
            Fleet,
            Position { x: 0.0, y: 10.0 },
            Health(100.0)
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(pulsar_damage_system);

        schedule.run(&mut world);

        let health = world.get::<Health>(fleet_entity).unwrap();
        assert_eq!(health.0, 100.0, "Fleet should not take damage when outside the beam");
    }

    #[test]
    fn test_angle_wrap_around() {
        let mut world = setup_world();
        let start_angle = std::f32::consts::PI * 2.0 - 0.5;
        let pulsar_entity = world.spawn(Pulsar {
            center_x: 0.0,
            center_y: 0.0,
            current_angle: start_angle,
            rotation_speed: 1.0,
            beam_width_rads: 0.5,
            damage_per_tick: 10.0,
        }).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(pulsar_rotation_system);

        schedule.run(&mut world);

        let pulsar = world.get::<Pulsar>(pulsar_entity).unwrap();
        let expected_angle = (start_angle + 1.0) % (std::f32::consts::PI * 2.0);

        // Assert float equality with a small epsilon
        assert!((pulsar.current_angle - expected_angle).abs() < 1e-4, "Angle should wrap around 2*PI");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Pulsar {
    pub center_x: f32,
    pub center_y: f32,
    pub current_angle: f32,
    pub rotation_speed: f32,
    pub beam_width_rads: f32,
    pub damage_per_tick: f32,
}

// Ensure you define or import Fleet, Position, Health as per your actual Layer 2 mechanics
#[derive(Component)]
pub struct Fleet;

#[derive(Component)]
pub struct Position { pub x: f32, pub y: f32 }

#[derive(Component)]
pub struct Health(pub f32);

pub fn pulsar_rotation_system(mut query: Query<&mut Pulsar>) {
    let tau = std::f32::consts::PI * 2.0;
    for mut pulsar in query.iter_mut() {
        pulsar.current_angle = (pulsar.current_angle + pulsar.rotation_speed) % tau;
        // ensure angle is positive
        if pulsar.current_angle < 0.0 {
            pulsar.current_angle += tau;
        }
    }
}

pub fn pulsar_damage_system(
    pulsar_query: Query<&Pulsar>,
    mut target_query: Query<(&Position, &mut Health), With<Fleet>>,
) {
    let tau = std::f32::consts::PI * 2.0;

    for pulsar in pulsar_query.iter() {
        let half_width = pulsar.beam_width_rads / 2.0;

        for (pos, mut health) in target_query.iter_mut() {
            let dx = pos.x - pulsar.center_x;
            let dy = pos.y - pulsar.center_y;

            // Calculate angle from pulsar to target
            let mut target_angle = dy.atan2(dx);
            if target_angle < 0.0 {
                target_angle += tau;
            }

            // Calculate angular difference, accounting for wrap-around
            let mut diff = (target_angle - pulsar.current_angle).abs();
            if diff > tau / 2.0 {
                diff = tau - diff;
            }

            // If within beam width, apply damage
            if diff <= half_width {
                health.0 -= pulsar.damage_per_tick;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Delta Time**: Modify `pulsar_rotation_system` to use a delta time variable rather than a per-tick fixed value, if applicable to the Layer 2 simulation loop.
- **Raycasting/Collision**: Using simple `atan2` is effective, but if Layer 2 has obstacles (like planets blocking the beam), we may need line-of-sight checks.
- **Component Filtering**: Ensure `pulsar_damage_system` correctly targets all damageable entities on Layer 2 (colonies, stations) instead of just `Fleet`. You may need to create a `DamageableLayer2` marker component or a more generic query.
- **Feedback Loop**: Consider firing an event `PulsarDamageEvent` instead of modifying `Health` directly so that the UI can flash, or ships can attempt evasion protocols.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for the new module.
- [ ] `cargo clippy -- -D warnings` passes without warnings.
- [ ] Test coverage ≥85% for the new code.
- [ ] The `Pulsar` rotation correctly wraps around $2\pi$.
- [ ] Targets within the angular threshold receive damage; targets outside do not.

## 7. Technical Guidance
- **Integration**: Add `pulsar_rotation_system` and `pulsar_damage_system` to your Layer 2 schedule. Order them appropriately (rotation first, then damage).
- **Float Arithmetic**: When testing angles, be aware of floating-point inaccuracies. Use an epsilon check for float comparisons. `atan2` returns `[-PI, PI]`, so normalize it to `[0, 2PI]` for easier logic.
- **Distance Falloff**: As a later enhancement, you might want damage to decay over distance. For MVP, flat damage within the cone is fine.

## 8. Questions
*Builder: add questions here if spec is unclear.*
