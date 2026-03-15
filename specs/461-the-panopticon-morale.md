# 461: The Panopticon Morale

## 1. Overview

The "Overseer Cameras" building creates a panopticon effect. It forces Pops in its radius to work at 120% speed, significantly increasing productivity. However, this comes at a steep psychological cost: their "Liberty" and "Leisure" needs drain twice as fast while under surveillance. If the cameras lose power, the accumulated hidden stress instantly converts into vandalism, creating a fragile, high-strung workforce.

## 2. Dependencies

- `004` — Basic Building
- `031` — Pop Morale
- `042` — Energy System
- `127` — Stress Breakdowns (for vandalism)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use super::*;
    use crate::layer1::pop::{Pop, Mood, Needs};
    use crate::layer1::map::GridPosition;
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::execution::work::WorkSpeed;

    #[test]
    fn test_overseer_cameras_increase_work_speed() {
        let mut world = World::new();

        let camera_pos = GridPosition { x: 5, y: 5 };
        world.spawn((
            OverseerCamera { radius: 5 },
            PowerConsumer { active: true, draw: 10.0 },
            camera_pos,
        ));

        let pop = world.spawn((
            Pop,
            camera_pos,
            WorkSpeed { multiplier: 1.0 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_panopticon_effect_system);
        schedule.run(&mut world);

        let work_speed = world.get::<WorkSpeed>(pop).unwrap();
        assert_eq!(work_speed.multiplier, 1.2, "Work speed should be increased by Overseer Cameras");
    }

    #[test]
    fn test_overseer_cameras_drain_needs_faster() {
        let mut world = World::new();

        let camera_pos = GridPosition { x: 5, y: 5 };
        world.spawn((
            OverseerCamera { radius: 5 },
            PowerConsumer { active: true, draw: 10.0 },
            camera_pos,
        ));

        let pop = world.spawn((
            Pop,
            camera_pos,
            Needs { liberty: 50.0, leisure: 50.0, ..Default::default() },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(drain_surveilled_needs_system);
        schedule.run(&mut world);

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.liberty < 50.0, "Liberty should drain under surveillance");
        assert!(needs.leisure < 50.0, "Leisure should drain under surveillance");
    }

    #[test]
    fn test_camera_power_loss_triggers_stress_spike() {
        let mut world = World::new();

        let camera_pos = GridPosition { x: 5, y: 5 };
        let camera_entity = world.spawn((
            OverseerCamera { radius: 5 },
            PowerConsumer { active: true, draw: 10.0 },
            camera_pos,
        )).id();

        let pop = world.spawn((
            Pop,
            camera_pos,
            SurveillanceStress { hidden_stress: 20.0 },
            Mood { stress: 10.0, ..Default::default() },
        )).id();

        // Simulate power loss
        world.entity_mut(camera_entity).get_mut::<PowerConsumer>().unwrap().active = false;

        let mut schedule = Schedule::default();
        schedule.add_systems(camera_power_loss_system);
        schedule.run(&mut world);

        let mood = world.get::<Mood>(pop).unwrap();
        assert_eq!(mood.stress, 30.0, "Hidden stress should convert to actual stress on power loss");

        let surveillance = world.get::<SurveillanceStress>(pop).unwrap();
        assert_eq!(surveillance.hidden_stress, 0.0, "Hidden stress should be cleared");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::{Needs, Mood};
use crate::layer1::energy::PowerConsumer;
use crate::layer1::execution::work::WorkSpeed;

#[derive(Component)]
pub struct OverseerCamera {
    pub radius: i32,
}

#[derive(Component, Default)]
pub struct SurveillanceStress {
    pub hidden_stress: f32,
}

pub fn apply_panopticon_effect_system(
    cameras: Query<(&GridPosition, &OverseerCamera, &PowerConsumer)>,
    mut pops: Query<(&GridPosition, &mut WorkSpeed)>,
) {
    for (pop_pos, mut work_speed) in pops.iter_mut() {
        let mut surveilled = false;
        for (cam_pos, camera, power) in cameras.iter() {
            if power.active && pop_pos.distance_chebyshev(*cam_pos) <= camera.radius {
                surveilled = true;
                break;
            }
        }

        if surveilled {
            work_speed.multiplier = work_speed.multiplier.max(1.2);
        } else {
            // Assume reset logic is handled elsewhere or reset here
             work_speed.multiplier = 1.0;
        }
    }
}

pub fn drain_surveilled_needs_system(
    cameras: Query<(&GridPosition, &OverseerCamera, &PowerConsumer)>,
    mut pops: Query<(&GridPosition, &mut Needs, &mut SurveillanceStress)>,
) {
    for (pop_pos, mut needs, mut stress) in pops.iter_mut() {
        let mut surveilled = false;
        for (cam_pos, camera, power) in cameras.iter() {
            if power.active && pop_pos.distance_chebyshev(*cam_pos) <= camera.radius {
                surveilled = true;
                break;
            }
        }

        if surveilled {
            needs.liberty -= 1.0; // Needs tuned
            needs.leisure -= 1.0;
            stress.hidden_stress += 0.5;
        }
    }
}

pub fn camera_power_loss_system(
    cameras: Query<(&GridPosition, &OverseerCamera, &PowerConsumer), Changed<PowerConsumer>>,
    mut pops: Query<(&GridPosition, &mut Mood, &mut SurveillanceStress)>,
) {
    // Only trigger if a camera became inactive
    let mut any_loss = false;
    for (_, _, power) in cameras.iter() {
        if !power.active {
            any_loss = true;
            break;
        }
    }

    if any_loss {
        // Find pops near the inactive camera
        for (pop_pos, mut mood, mut stress) in pops.iter_mut() {
            for (cam_pos, camera, power) in cameras.iter() {
                if !power.active && pop_pos.distance_chebyshev(*cam_pos) <= camera.radius {
                    mood.stress += stress.hidden_stress;
                    stress.hidden_stress = 0.0;
                    break;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Spatial Indexing:** O(N*M) iteration for distance checking between Pops and cameras is expensive. Use a spatial grid or sector lookup to optimize proximity checks.
- **Gradual Decay:** Instead of instantly converting `hidden_stress` to `stress` on power loss, it could rapidly decay into real stress over a few ticks, allowing for some visual feedback (e.g., a "Panic" state).
- **Vandalism Hook:** Ensure the sudden spike in `stress` correctly triggers the `127` Stress Breakdowns system (specifically the Vandalism action). The stress spike might need to forcefully interrupt current actions.
- **Multiple Cameras:** Handle overlapping camera radii elegantly so `hidden_stress` doesn't accumulate multiple times if a Pop is in the range of two cameras.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/infrastructure/panopticon.rs`.
- [ ] Pops under powered cameras work 20% faster but lose liberty and leisure.
- [ ] Camera power loss converts hidden surveillance stress to actual stress immediately.

## 7. Technical Guidance

- You will likely need to create the `SurveillanceStress` component and add it to the default `PopBundle` or insert it dynamically when a Pop enters a camera's radius.
- The `camera_power_loss_system` uses `Changed<PowerConsumer>`. Ensure it correctly identifies the transition from `active: true` to `active: false`. It might be necessary to store previous state if `PowerConsumer` doesn't handle transitions inherently well in queries.

## 8. Questions

*Builder: add questions here if spec is unclear.*
