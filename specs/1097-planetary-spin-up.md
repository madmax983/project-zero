# 1097: Planetary Spin-Up

## Overview

A Layer 2 -> 1 feature allowing massive engineering projects (surface engines or orbital tethers) to apply torque to a planet, changing its Day/Night cycle length. This can fix tidally locked worlds or optimize solar energy, but causes extreme weather chaos via the Coriolis effect and slightly alters effective gravity.

## Dependencies

- Layer 1 Weather and Gravity mechanics.
- Layer 1/2 Day/Night Cycle mechanics.

## RED Phase: Tests First

```rust
#[test]
fn test_applying_torque_changes_day_length() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, apply_planetary_torque_system);

    // Arrange: Planet with 24h day, and a planetary engine applying torque
    let planet = app.world_mut().spawn((
        Planet,
        DayLength { hours: 24.0 },
    )).id();

    app.world_mut().spawn((
        PlanetaryEngine { torque: 1.0, active: true },
        AttachedToPlanet(planet),
    ));

    app.update();

    // Assert: Day length has decreased (planet spinning faster)
    let day_length = app.world().get::<DayLength>(planet).unwrap().hours;
    assert!(day_length < 24.0, "Day length should decrease when positive torque is applied");
}

#[test]
fn test_rapid_spin_reduces_effective_gravity() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, calculate_effective_gravity_system);

    // Arrange: Base gravity of 1.0, fast spin (short day length)
    let planet = app.world_mut().spawn((
        Planet,
        BaseGravity { g: 1.0 },
        DayLength { hours: 10.0 }, // Very fast spin
        EffectiveGravity { g: 1.0 },
    )).id();

    app.update();

    // Assert: Effective gravity is lower due to centrifugal force
    let eff_g = app.world().get::<EffectiveGravity>(planet).unwrap().g;
    assert!(eff_g < 1.0, "Effective gravity should be reduced by rapid spin");
}

#[test]
fn test_spin_up_causes_chaotic_weather() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, trigger_coriolis_weather_system);

    // Arrange: Apply sudden torque
    let planet = app.world_mut().spawn((
        Planet,
        DayLength { hours: 24.0 },
        WeatherState::Calm,
    )).id();

    app.world_mut().resource_mut::<Events<PlanetaryTorqueEvent>>().send(PlanetaryTorqueEvent {
        planet,
        torque_amount: 5.0, // High sudden torque
    });

    app.update();

    // Assert: Weather becomes chaotic
    assert_eq!(*app.world().get::<WeatherState>(planet).unwrap(), WeatherState::Chaotic);
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Planet;

#[derive(Component)]
pub struct DayLength {
    pub hours: f32,
}

#[derive(Component)]
pub struct PlanetaryEngine {
    pub torque: f32,
    pub active: bool,
}

#[derive(Component)]
pub struct AttachedToPlanet(pub Entity);

pub fn apply_planetary_torque_system(
    engine_query: Query<(&PlanetaryEngine, &AttachedToPlanet)>,
    mut planet_query: Query<&mut DayLength>,
) {
    for (engine, attachment) in engine_query.iter() {
        if engine.active {
            if let Ok(mut day_length) = planet_query.get_mut(attachment.0) {
                // Simplified physics: positive torque reduces day length
                day_length.hours -= engine.torque * 0.1;
                // Clamp to avoid negative or zero day length
                day_length.hours = day_length.hours.max(1.0);
            }
        }
    }
}

#[derive(Component)]
pub struct BaseGravity {
    pub g: f32,
}

#[derive(Component)]
pub struct EffectiveGravity {
    pub g: f32,
}

pub fn calculate_effective_gravity_system(
    mut planet_query: Query<(&BaseGravity, &DayLength, &mut EffectiveGravity)>,
) {
    for (base_g, day_length, mut eff_g) in planet_query.iter_mut() {
        // Simplified centrifugal reduction: faster spin (lower hours) = more reduction
        let reduction = 1.0 / day_length.hours;
        eff_g.g = (base_g.g - reduction).max(0.1);
    }
}

#[derive(Component, PartialEq, Debug)]
pub enum WeatherState {
    Calm,
    Chaotic,
}

#[derive(Event)]
pub struct PlanetaryTorqueEvent {
    pub planet: Entity,
    pub torque_amount: f32,
}

pub fn trigger_coriolis_weather_system(
    mut events: EventReader<PlanetaryTorqueEvent>,
    mut planet_query: Query<&mut WeatherState>,
) {
    for event in events.read() {
        if event.torque_amount.abs() > 2.0 {
            if let Ok(mut weather) = planet_query.get_mut(event.planet) {
                *weather = WeatherState::Chaotic;
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Physics Realism**: Torque altering Day Length directly is a shortcut. It should modify angular velocity, which then derives day length.
- **Event vs Continuous**: The weather chaos is triggered by an event in the test, but the `apply_planetary_torque_system` continuously applies torque without sending events. They should be unified so continuous high torque generates weather instability.
- **Tidally Locked Support**: Ensure `DayLength` can represent "Infinite" (tidally locked) and gracefully transition to a finite number when torque is applied.

## Acceptance Criteria

- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the Spin-Up logic.
- [ ] Spin changes accurately affect day length, gravity, and trigger weather events.

## Technical Guidance

- Integrate `apply_planetary_torque_system` with the energy grid (engines should consume massive power).
- `EffectiveGravity` should be the value queried by Layer 1 pop movement/construction systems.

## Questions

*Builder: add questions here if spec is unclear.*
