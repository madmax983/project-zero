# Specification: Cloud Seeding (Feature 808)

## 1. Overview
The **Cloud Seeding** feature introduces atmospheric manipulation. Players can launch rockets from Layer 1 or Ships in Layer 2 to detonate chemical payloads in the atmosphere. This triggers forced weather events like Rain or Snow to clear out localized Pollution or Fires. However, if the atmosphere is sufficiently polluted prior to seeding, it risks generating "Toxic Rain," turning a solution into an ecological disaster that damages structures like solar panels.

## 2. Dependencies
- `Layer 1 System`
- `Layer 2 System`
- `Weather and Atmosphere Simulation System`
- `Pollution System`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_cloud_seeding_triggers_rain() {
        let mut app = App::new();
        app.add_systems(Update, process_cloud_seeding_system);

        // Arrange
        let entity = app.world_mut().spawn((
            Atmosphere { pollution_level: 10.0, state: WeatherState::Clear },
            CloudSeedingPayload
        )).id();

        // Act
        app.update();

        // Assert
        let atmosphere = app.world().get::<Atmosphere>(entity).unwrap();
        assert_eq!(atmosphere.state, WeatherState::Rain, "Cloud seeding should trigger rain");
    }

    #[test]
    fn test_cloud_seeding_in_high_pollution_triggers_toxic_rain() {
        let mut app = App::new();
        app.add_systems(Update, process_cloud_seeding_system);

        // Arrange
        let entity = app.world_mut().spawn((
            Atmosphere { pollution_level: 80.0, state: WeatherState::Clear },
            CloudSeedingPayload
        )).id();

        // Act
        app.update();

        // Assert
        let atmosphere = app.world().get::<Atmosphere>(entity).unwrap();
        assert_eq!(atmosphere.state, WeatherState::ToxicRain, "Cloud seeding in high pollution should cause Toxic Rain");
    }

    #[test]
    fn test_toxic_rain_damages_vulnerable_structures() {
        let mut app = App::new();
        app.add_systems(Update, apply_toxic_rain_damage_system);

        // Arrange
        let structure = app.world_mut().spawn((
            SolarPanel { health: 100.0 },
            ExposedToWeather { state: WeatherState::ToxicRain }
        )).id();

        // Act
        app.update();

        // Assert
        let panel = app.world().get::<SolarPanel>(structure).unwrap();
        assert!(panel.health < 100.0, "Toxic Rain should damage vulnerable structures like solar panels");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Atmosphere {
    pub pollution_level: f32,
    pub state: WeatherState,
}

#[derive(Component)]
pub struct CloudSeedingPayload;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum WeatherState {
    Clear,
    Rain,
    ToxicRain,
}

#[derive(Component)]
pub struct ExposedToWeather {
    pub state: WeatherState,
}

#[derive(Component)]
pub struct SolarPanel {
    pub health: f32,
}

pub fn process_cloud_seeding_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Atmosphere, &CloudSeedingPayload)>,
) {
    for (entity, mut atmosphere, _) in query.iter_mut() {
        if atmosphere.pollution_level > 50.0 {
            atmosphere.state = WeatherState::ToxicRain;
        } else {
            atmosphere.state = WeatherState::Rain;
        }
        commands.entity(entity).remove::<CloudSeedingPayload>();
    }
}

pub fn apply_toxic_rain_damage_system(mut query: Query<(&mut SolarPanel, &ExposedToWeather)>) {
    for (mut panel, weather) in query.iter_mut() {
        if weather.state == WeatherState::ToxicRain {
            panel.health -= 5.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create an `Event` system for cloud seeding instead of checking a component state every frame. `CloudSeedingEvent` could carry coordinates for localized effects.
- Implement a `VulnerableToToxicRain` trait or marker component so that `apply_toxic_rain_damage_system` can affect more than just `SolarPanel` structures.
- Transition weather resolution to a time-based decay system where toxic rain naturally dissipates over `n` ticks.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] The transition logic between clean Rain and Toxic Rain operates strictly based on local pollution thresholds.

## 7. Technical Guidance
- When processing the `CloudSeedingPayload`, make sure to consume/remove the payload component to prevent it from re-triggering weather calculations every tick.
- Hook into the existing Layer 2 systems if launching from orbit, ensuring that the target coordinates in Layer 1 are accurately translated.

## 8. Questions
*Builder: add questions here if spec is unclear.*
