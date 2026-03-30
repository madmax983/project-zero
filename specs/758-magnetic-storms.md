# Spec 758: Magnetic Storms

## 1. Overview
**Layer:** 2 -> 1
**Fantasy:** The sky is beautiful and deadly to machines.
**Mechanic:** Solar event. Beautiful shaders in the sky. Unshielded electronics (Turrets, Bots, High-tech benches) are disabled or damaged. Comms are cut.
**Emergence:** A storm hits during a raid. Your automated defenses shut down. You have to send the pacifist scientists out with wrenches to fight the raiders.
**Tension:** Low-tech reliability (Mechanical) vs. High-tech power (Electronic).

## 2. Dependencies
- Global weather/event systems (`PlanetWeather` or equivalent).
- `ElectronicDevice` and `PowerGrid` components.
- Status effect / Disabled flags for buildings and bots.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::power::PowerConsumer;

    #[test]
    fn test_magnetic_storm_disables_unshielded_electronics() {
        let mut app = App::new();

        let turret = app.world_mut().spawn((
            ElectronicDevice,
            PowerConsumer { active: true },
        )).id();

        let shielded_turret = app.world_mut().spawn((
            ElectronicDevice,
            MagneticShield,
            PowerConsumer { active: true },
        )).id();

        // Trigger a magnetic storm
        app.add_systems(Update, process_magnetic_storm_system);
        app.world_mut().insert_resource(GlobalWeather { is_magnetic_storm: true });
        app.update();

        // Assert unshielded is disabled
        let power = app.world().get::<PowerConsumer>(turret).unwrap();
        assert!(!power.active, "Unshielded electronics should be disabled");

        // Assert shielded is still active
        let shielded_power = app.world().get::<PowerConsumer>(shielded_turret).unwrap();
        assert!(shielded_power.active, "Shielded electronics should remain active");
    }

    #[test]
    fn test_magnetic_storm_cuts_communications() {
        let mut app = App::new();

        let comms_array = app.world_mut().spawn((
            CommunicationsArray,
            ElectronicDevice,
            PowerConsumer { active: true },
        )).id();

        app.add_systems(Update, check_comms_status_system);
        app.world_mut().insert_resource(GlobalWeather { is_magnetic_storm: true });
        app.update();

        let comms = app.world().get::<CommunicationsArray>(comms_array).unwrap();
        assert!(comms.is_offline, "Communications should be cut during a storm");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct GlobalWeather {
    pub is_magnetic_storm: bool,
}

#[derive(Component)]
pub struct ElectronicDevice;

#[derive(Component)]
pub struct MagneticShield;

#[derive(Component)]
pub struct PowerConsumer {
    pub active: bool,
}

#[derive(Component, Default)]
pub struct CommunicationsArray {
    pub is_offline: bool,
}

pub fn process_magnetic_storm_system(
    weather: Res<GlobalWeather>,
    mut devices: Query<(&ElectronicDevice, Option<&MagneticShield>, &mut PowerConsumer)>,
) {
    if weather.is_magnetic_storm {
        for (_, shield, mut power) in devices.iter_mut() {
            if shield.is_none() {
                power.active = false;
            }
        }
    }
}

pub fn check_comms_status_system(
    weather: Res<GlobalWeather>,
    mut comms: Query<(&mut CommunicationsArray, Option<&MagneticShield>)>,
) {
    if weather.is_magnetic_storm {
        for (mut array, shield) in comms.iter_mut() {
            if shield.is_none() {
                array.is_offline = true;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **State Reversion**: The minimal implementation turns things off but never turns them back on when the storm ends. Implement a robust `StatusEffect::Disabled(Duration)` or rely on a `DisabledByWeather` marker component that gets added/removed so the base state isn't permanently overwritten.
- **Drone Interaction**: Ensure that any `Drone` entities also receive the `ElectronicDevice` component and handle being disabled (e.g., dropping hauled items, freezing movement).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code
- [ ] Unshielded turrets, bots, and comms shut down during active storms and reactivate when it clears.

## 7. Technical Guidance
- Integrate into the `Weather` or `Environment` update schedule.
- Ensure that UI elements reflecting Comms or Drone status correctly display the disabled state to the player.

## 8. Questions
*Builder: add questions here if spec is unclear.*
