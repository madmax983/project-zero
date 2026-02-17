# 149: Cometary Injection

## Overview

Introduces dynamic **Comets** to the System View (Layer 2). Comets are temporary celestial bodies that spawn on parabolic trajectories, traversing the system map before exiting.
They serve as high-risk, high-reward resource nodes. Players must intercept them with fleets (Spec 099) to mine rare resources before they disappear.

## Dependencies

- `094` — System View Architecture (OrbitalBody, System Map)
- `099` — Fleet Movement (Required to interact with comets)
- `001` — Project Scaffold (SimulationTime)

## RED Phase: Tests First

Write these tests in `src/layer2/comet_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::system::OrbitalBody;
    use crate::layer2::comet::{Comet, CometSpawner, SystemPosition, comet_spawning_system, comet_movement_system, comet_despawn_system};
    use crate::shared::time::SimulationTime;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(CometSpawner::default());
        world
    }

    #[test]
    fn test_comet_spawns_on_timer() {
        let mut world = setup_world();
        let mut spawner = world.resource_mut::<CometSpawner>();
        spawner.timer = 0; // Ready to spawn

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(comet_spawning_system);
        schedule.run(&mut world);

        // Check for Comet entity
        let count = world.query::<(&Comet, &OrbitalBody, &SystemPosition)>().iter(&world).count();
        assert_eq!(count, 1, "Should spawn a comet when timer expires");

        // Check timer reset
        let spawner = world.resource::<CometSpawner>();
        assert!(spawner.timer > 0, "Timer should reset");
    }

    #[test]
    fn test_comet_moves() {
        let mut world = setup_world();

        // Spawn comet at (0,0) with velocity (1,0)
        let comet = world.spawn((
            Comet { velocity: (1.0, 0.0).into(), ..Default::default() },
            SystemPosition { x: 0.0, y: 0.0 },
        )).id();

        // Run movement
        let mut schedule = Schedule::default();
        schedule.add_systems(comet_movement_system);
        schedule.run(&mut world);

        let pos = world.get::<SystemPosition>(comet).unwrap();
        assert!((pos.x - 1.0).abs() < f32::EPSILON, "Comet should move by velocity");
    }

    #[test]
    fn test_comet_despawns_out_of_bounds() {
        let mut world = setup_world();

        // Spawn comet far away
        let comet = world.spawn((
            Comet::default(),
            SystemPosition { x: 1000.0, y: 1000.0 },
        )).id();

        // Run despawn system
        let mut schedule = Schedule::default();
        schedule.add_systems(comet_despawn_system);
        schedule.run(&mut world);

        assert!(world.get_entity(comet).is_none(), "Comet out of bounds should despawn");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer2/comet.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer2::system::OrbitalBody;
use ratatui::style::Color;
use rand::Rng;

/// Represents coordinates in the System View for non-orbiting bodies.
/// Distinct from `Orbit` which uses polar coordinates relative to a parent.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct SystemPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Default)]
pub struct Comet {
    pub velocity: bevy_math::Vec2,
    pub resource_amount: f32,
}

#[derive(Resource)]
pub struct CometSpawner {
    pub timer: u32,
    pub spawn_interval: u32,
}

impl Default for CometSpawner {
    fn default() -> Self {
        Self {
            timer: 1000, // Ticks
            spawn_interval: 1000,
        }
    }
}
```

### 2. Spawning System

```rust
pub fn comet_spawning_system(
    mut commands: Commands,
    mut spawner: ResMut<CometSpawner>,
) {
    if spawner.timer > 0 {
        spawner.timer -= 1;
        return;
    }

    spawner.timer = spawner.spawn_interval;

    let mut rng = rand::thread_rng();

    // Spawn logic: Start at edge (e.g., +/- 100), aim towards center
    let start_x = if rng.gen_bool(0.5) { -100.0 } else { 100.0 };
    let start_y = rng.gen_range(-100.0..100.0);

    // Velocity towards 0,0 roughly
    let vel_x = -start_x.signum() * rng.gen_range(0.5..1.5);
    let vel_y = rng.gen_range(-0.5..0.5);

    commands.spawn((
        Comet {
            velocity: bevy_math::Vec2::new(vel_x, vel_y),
            resource_amount: 1000.0,
        },
        SystemPosition { x: start_x, y: start_y },
        OrbitalBody {
            name: "Comet".to_string(),
            radius: 0.5,
            color: Color::Cyan,
            char: '☄',
        },
    ));
}
```

### 3. Movement System

```rust
pub fn comet_movement_system(mut query: Query<(&mut SystemPosition, &Comet)>) {
    for (mut pos, comet) in &mut query {
        pos.x += comet.velocity.x;
        pos.y += comet.velocity.y;
    }
}
```

### 4. Despawn System

```rust
pub fn comet_despawn_system(
    mut commands: Commands,
    query: Query<(Entity, &SystemPosition), With<Comet>>,
) {
    let bounds = 200.0;
    for (entity, pos) in &query {
        if pos.x.abs() > bounds || pos.y.abs() > bounds {
            commands.entity(entity).despawn();
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Unified Positioning**: Consider updating `Orbit` system to write to a `SystemPosition` component so all L2 entities share a common coordinate component for rendering and interaction.
- **Trajectory Math**: Implement hyperbolic/parabolic orbits using Keplerian physics instead of simple velocity vectors for realism.
- **Visuals**: Add a "Tail" effect (trail of particles or UI visual).
- **Resources**: Randomize resource types (Ice, Rare Metals).
- **Interception**: Add logic for fleets to "dock" or "mine" moving targets.

## Acceptance Criteria

- [ ] `CometSpawner` resource exists.
- [ ] `SystemPosition` component is defined.
- [ ] Comets spawn periodically at system edges.
- [ ] Comets move across the map using `SystemPosition`.
- [ ] Comets despawn when leaving the system bounds.
- [ ] Comets have `OrbitalBody` component for rendering.
- [ ] Tests pass.

## Technical Guidance

- Register `CometSpawner` in `layer2/mod.rs`.
- Ensure `SystemPosition` is used for rendering comets in `014` (Rendering Architecture) or the System View UI.
- Use `bevy_math::Vec2` for vector operations.
