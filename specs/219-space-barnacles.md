# 219: Space Barnacles

## Overview

Space Barnacles are parasitic void-fauna that attach to ships during interstellar transit. As they accumulate, they increase the effective mass of the fleet, significantly slowing down travel times. If left unchecked, they can cripple a fleet's mobility.

This feature adds an environmental hazard to Layer 2 (System Simulation), forcing players to maintain their fleets rather than just sending them on endless loops.

## Dependencies

- `099` — Fleet Movement (Implemented in `src/layer2/fleet.rs`)

## RED Phase: Tests First

Write these tests in `src/layer2/barnacle_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::fleet::{Fleet, InTransit, InOrbit};
    use crate::layer2::barnacles::{SpaceBarnacles, barnacle_accumulation_system, barnacle_drag_system};

    // 1. Test Barnacle Accumulation
    #[test]
    fn test_barnacle_accumulation() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(barnacle_accumulation_system);

        // Spawn a fleet in transit
        let fleet = world.spawn((
            Fleet,
            InTransit {
                origin: Entity::from_raw(0),
                destination: Entity::from_raw(1),
                progress: 0.0,
                duration: 100.0,
            },
            SpaceBarnacles { count: 0 },
        )).id();

        // Run system multiple times to simulate time passing
        for _ in 0..100 {
            schedule.run(&mut world);
        }

        let barnacles = world.get::<SpaceBarnacles>(fleet).unwrap();
        assert!(barnacles.count > 0, "Barnacles should accumulate during transit");
    }

    // 2. Test Drag Effect on Speed
    #[test]
    fn test_barnacle_drag() {
        let mut world = World::new();
        // Note: This test assumes we modify or wrap the fleet_movement_system
        // OR that we have a separate system that modifies the transit progress/duration.
        // For TDD, let's assume we implement a `calculate_speed_modifier` function.

        let clean_speed = crate::layer2::barnacles::calculate_speed_modifier(0);
        let dirty_speed = crate::layer2::barnacles::calculate_speed_modifier(100);

        assert_eq!(clean_speed, 1.0);
        assert!(dirty_speed < 1.0, "Barnacles should reduce speed");
        assert!(dirty_speed > 0.1, "Should not completely stop (min speed cap)");
    }

    // 3. Test Cleaning
    #[test]
    fn test_barnacle_cleaning() {
        let mut world = World::new();
        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: Entity::from_raw(0) }, // Must be in orbit to clean
            SpaceBarnacles { count: 50 },
        )).id();

        // Run cleaning command/system
        crate::layer2::barnacles::clean_barnacles(&mut world, fleet);

        let barnacles = world.get::<SpaceBarnacles>(fleet).unwrap();
        assert_eq!(barnacles.count, 0, "Cleaning should remove all barnacles");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Component

In `src/layer2/barnacles.rs`:

```rust
use bevy_ecs::prelude::*;
use rand::Rng;
use crate::layer2::fleet::InTransit;

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct SpaceBarnacles {
    pub count: u32,
}

pub const MAX_BARNACLES: u32 = 1000;
pub const DRAG_PER_BARNACLE: f32 = 0.0005; // 0.05% slow per barnacle
pub const MIN_SPEED: f32 = 0.1;

/// Calculates the speed multiplier (0.1 to 1.0).
pub fn calculate_speed_modifier(count: u32) -> f32 {
    let penalty = count as f32 * DRAG_PER_BARNACLE;
    (1.0 - penalty).max(MIN_SPEED)
}

/// System to randomly add barnacles to fleets in transit.
pub fn barnacle_accumulation_system(
    mut query: Query<(&mut SpaceBarnacles, &InTransit)>,
) {
    let mut rng = rand::thread_rng();
    for (mut barnacles, _transit) in query.iter_mut() {
        // 5% chance per tick to gain a barnacle
        if rng.gen_bool(0.05) {
            barnacles.count = (barnacles.count + 1).min(MAX_BARNACLES);
        }
    }
}

/// Helper to clean barnacles (called via command or interaction).
pub fn clean_barnacles(world: &mut World, fleet_entity: Entity) {
    if let Some(mut barnacles) = world.get_mut::<SpaceBarnacles>(fleet_entity) {
        barnacles.count = 0;
    }
}
```

### 2. Integrate with Movement

In `src/layer2/fleet.rs`, modify `fleet_movement_system` to account for drag:

```rust
// Add SpaceBarnacles to the query
use crate::layer2::barnacles::{SpaceBarnacles, calculate_speed_modifier};

pub fn fleet_movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut InTransit, Option<&SpaceBarnacles>)>
) {
    for (entity, mut transit, maybe_barnacles) in &mut query {
        let speed_mod = if let Some(b) = maybe_barnacles {
            calculate_speed_modifier(b.count)
        } else {
            1.0
        };

        // Apply drag to the delta
        let delta = (1.0 / transit.duration) * speed_mod;
        transit.progress += delta;

        if transit.progress >= 1.0 {
            // ... arrival logic ...
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Visuals**: Barnacle count should affect the fleet's appearance (e.g., change color to greenish-grey, or append status icon).
- **Location Variance**: Barnacles should accumulate faster in "Nebula" zones (future feature) or Deep Space, and slower near Stars.
- **Cleaning Cost**: Cleaning shouldn't be free. It should cost Energy or Time (turns skipped).
- **Events**: Critical mass of barnacles could trigger a "Hull Breach" event.

## Acceptance Criteria

- [ ] `SpaceBarnacles` component exists and tracks count.
- [ ] Fleets in transit accumulate barnacles over time.
- [ ] `fleet_movement_system` applies a speed penalty based on barnacle count.
- [ ] Speed penalty is capped (fleets don't stop completely).
- [ ] Cleaning mechanism exists to reset count to 0.

## Technical Guidance

- Ensure `SpaceBarnacles` is added to new Fleets by default (or handle `Option` gracefully as shown).
- Tweak `DRAG_PER_BARNACLE` and accumulation rate so it's noticeable but not annoying in early game.
- Use `#[allow(dead_code)]` if `clean_barnacles` isn't wired up to UI yet.

## Questions

- *Builder: Should barnacles fall off if the fleet moves fast enough?*
  *Architect:* Barnacles are shed upon entering subspace jumps, otherwise they persist.
  *Architect:* No, they must be manually scrubbed off or handled via specific drydock cleaning events.
