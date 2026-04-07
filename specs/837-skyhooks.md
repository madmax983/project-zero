# Skyhooks

## 1. Overview
**Layer:** 2
**Fantasy:** Catching the train to the stars.
**Mechanic:** A rotating orbital structure that dips into the upper atmosphere. Ships can "Hook" onto it for a momentum boost to orbit without burning fuel. Requires precise timing window. Miss the window = crash.
**Emergence:** You time the launch of a heavy freighter perfectly. But the Skyhook was damaged by debris and releases early, flinging the freighter into deep space instead of orbit.
**Tension:** High skill/risk logistics (Free fuel) vs. Reliable rockets (Fuel cost).

## 2. Dependencies
- Layer 2 Orbit and Movement mechanics.
- Ship logistics and fuel consumption.
- Infrastructure durability/damage system.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct Ship { fuel: f32, in_orbit: bool, trajectory_error: f32 }

    #[derive(Component)]
    struct Skyhook { is_damaged: bool, active_window: bool }

    #[test]
    fn test_skyhook_successful_launch_saves_fuel() {
        let mut app = App::new();
        app.add_systems(Update, process_skyhook_launch);

        let ship = app.world_mut().spawn(Ship { fuel: 100.0, in_orbit: false, trajectory_error: 0.0 }).id();
        let skyhook = app.world_mut().spawn(Skyhook { is_damaged: false, active_window: true }).id();

        app.world_mut().resource_mut::<Events<LaunchIntent>>().send(LaunchIntent { ship, via_skyhook: Some(skyhook) });
        app.update();

        let launched_ship = app.world().get::<Ship>(ship).unwrap();
        assert!(launched_ship.in_orbit, "Ship should reach orbit.");
        assert_eq!(launched_ship.fuel, 100.0, "Fuel should not be consumed via skyhook.");
    }

    #[test]
    fn test_skyhook_missed_window_causes_crash() {
        let mut app = App::new();
        app.add_systems(Update, process_skyhook_launch);

        let ship = app.world_mut().spawn(Ship { fuel: 100.0, in_orbit: false, trajectory_error: 0.0 }).id();
        let skyhook = app.world_mut().spawn(Skyhook { is_damaged: false, active_window: false }).id();

        app.world_mut().resource_mut::<Events<LaunchIntent>>().send(LaunchIntent { ship, via_skyhook: Some(skyhook) });
        app.update();

        assert!(app.world().get::<Ship>(ship).is_none(), "Ship should be destroyed on missed window crash.");
    }

    #[test]
    fn test_damaged_skyhook_flings_ship_off_course() {
        let mut app = App::new();
        app.add_systems(Update, process_skyhook_launch);

        let ship = app.world_mut().spawn(Ship { fuel: 100.0, in_orbit: false, trajectory_error: 0.0 }).id();
        let skyhook = app.world_mut().spawn(Skyhook { is_damaged: true, active_window: true }).id();

        app.world_mut().resource_mut::<Events<LaunchIntent>>().send(LaunchIntent { ship, via_skyhook: Some(skyhook) });
        app.update();

        let launched_ship = app.world().get::<Ship>(ship).unwrap();
        assert!(!launched_ship.in_orbit, "Ship should fail to reach intended orbit.");
        assert!(launched_ship.trajectory_error > 0.0, "Ship should have significant trajectory error (deep space fling).");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Event)]
pub struct LaunchIntent {
    pub ship: Entity,
    pub via_skyhook: Option<Entity>,
}

pub fn process_skyhook_launch(
    mut commands: Commands,
    mut intents: EventReader<LaunchIntent>,
    mut ships: Query<&mut Ship>,
    skyhooks: Query<&Skyhook>,
) {
    for intent in intents.read() {
        if let Some(skyhook_entity) = intent.via_skyhook {
            if let Ok(skyhook) = skyhooks.get(skyhook_entity) {
                if !skyhook.active_window {
                    commands.entity(intent.ship).despawn();
                    continue;
                }

                if let Ok(mut ship) = ships.get_mut(intent.ship) {
                    if skyhook.is_damaged {
                        ship.trajectory_error = 100.0;
                    } else {
                        ship.in_orbit = true;
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Robust Event Handling:** Abstract the launch intent to support standard rocketry alongside skyhook mechanics.
- **Timing Windows:** The `active_window` bool should likely be replaced by a periodic float timer synced with global rotation mechanics, granting a limited time frame (e.g., 5 game ticks) for success.
- **Debris/Damage Calculation:** Damaged skyhooks might just impart a probabilistic failure rate rather than guaranteed error. Consider rolling a chance to release early.
- **Visual Feedback:** Consider spawning warning markers or logging messages when the window is tight.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Launching via an intact Skyhook during an active window places the ship in orbit with zero fuel consumed.
- [ ] Missed windows result in catastrophic failure (ship destruction).
- [ ] Damaged skyhooks cause massive trajectory errors.

## 7. Technical Guidance
- Integrate with Layer 2's movement and positioning system. If orbital mechanics use specific coordinate models (e.g., polar coordinates), ensure the fling error reflects a tangential velocity vector pointing away from the planet.
- Avoid locking the simulation when waiting for the active window; use asynchronous intents or a dedicated scheduling system to queue launches.

## 8. Questions
*Builder: add questions here if spec is unclear.*
