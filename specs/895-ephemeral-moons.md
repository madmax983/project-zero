# Ephemeral Moons

## 1. Overview
The colony's night sky is not static. The planet occasionally captures temporary asteroids or moons. These celestial bodies alter tides, light, and potentially gravity for a few months before being ejected back into space or crashing. A "Bright Moon," for example, provides a significant boost to solar power during night cycles, allowing players to plan production sprints or delay launch windows to capitalize on the temporary environmental shift.

## 2. Dependencies
- Base simulation framework (`App`, `World`)
- `Time` resource for tracking durations
- Event system (e.g., `MoonCapturedEvent`, `MoonEjectedEvent`)
- Layer 1 environmental modifiers (e.g., `SolarPowerYield`, `NightCycleModifier`)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_moon_capture_creates_ephemeral_moon_entity() {
    // Arrange
    let mut app = App::new();
    app.add_event::<MoonCapturedEvent>();
    app.add_systems(Update, handle_moon_capture_system);

    // Act
    app.world_mut().send_event(MoonCapturedEvent {
        moon_type: MoonType::Bright,
        duration_days: 30.0,
    });
    app.update();

    // Assert: An EphemeralMoon entity should exist
    let moon_query = app.world_mut().query::<&EphemeralMoon>().iter(&app.world()).next();
    assert!(moon_query.is_some());
    assert_eq!(moon_query.unwrap().days_remaining, 30.0);
}

#[test]
fn test_bright_moon_boosts_night_solar_power() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, apply_moon_modifiers_system);

    app.world_mut().spawn(EphemeralMoon {
        moon_type: MoonType::Bright,
        days_remaining: 10.0,
    });

    let solar_panel = app.world_mut().spawn(SolarPowerYield { base_yield: 10.0, current_yield: 10.0 }).id();
    let mut env = EnvironmentState { is_night: true };
    app.insert_resource(env);

    // Act
    app.update();

    // Assert: Solar yield should be boosted because it is night and a bright moon is present
    let panel = app.world().get::<SolarPowerYield>(solar_panel).unwrap();
    assert!(panel.current_yield > panel.base_yield);
}

#[test]
fn test_moon_ejected_after_duration() {
    // Arrange
    let mut app = App::new();
    app.insert_resource(Time::default());
    app.add_event::<MoonEjectedEvent>();
    app.add_systems(Update, decay_ephemeral_moons_system);

    let moon = app.world_mut().spawn(EphemeralMoon {
        moon_type: MoonType::Bright,
        days_remaining: 1.0, // 1 day remaining
    }).id();

    // Act: Advance time by more than 1 day (assuming 1 day = 100 seconds for test purposes)
    // Note: Builder should adjust time scaling based on actual game design
    app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(101));
    app.update();

    // Assert: Moon entity should be despawned and event emitted
    assert!(app.world().get::<EphemeralMoon>(moon).is_none());
    let events = app.world().resource::<Events<MoonEjectedEvent>>();
    let mut reader = events.get_reader();
    assert!(reader.read(events).next().is_some());
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MoonType {
    Bright,
    Dark,
    Gravitational,
}

#[derive(Event)]
pub struct MoonCapturedEvent {
    pub moon_type: MoonType,
    pub duration_days: f32,
}

#[derive(Event)]
pub struct MoonEjectedEvent {
    pub entity: Entity,
}

#[derive(Component)]
pub struct EphemeralMoon {
    pub moon_type: MoonType,
    pub days_remaining: f32,
}

#[derive(Component)]
pub struct SolarPowerYield {
    pub base_yield: f32,
    pub current_yield: f32,
}

#[derive(Resource)]
pub struct EnvironmentState {
    pub is_night: bool,
}

pub fn handle_moon_capture_system(
    mut commands: Commands,
    mut events: EventReader<MoonCapturedEvent>,
) {
    for event in events.read() {
        commands.spawn(EphemeralMoon {
            moon_type: event.moon_type,
            days_remaining: event.duration_days,
        });
    }
}

pub fn apply_moon_modifiers_system(
    moons: Query<&EphemeralMoon>,
    env: Res<EnvironmentState>,
    mut panels: Query<&mut SolarPowerYield>,
) {
    let mut bright_moon_present = false;
    for moon in moons.iter() {
        if moon.moon_type == MoonType::Bright {
            bright_moon_present = true;
            break;
        }
    }

    for mut panel in panels.iter_mut() {
        if env.is_night && bright_moon_present {
            panel.current_yield = panel.base_yield * 1.5; // 50% boost at night
        } else if env.is_night {
             panel.current_yield = 0.0; // Normal night behavior
        } else {
             panel.current_yield = panel.base_yield; // Day time
        }
    }
}

pub fn decay_ephemeral_moons_system(
    mut commands: Commands,
    time: Res<Time>,
    mut moons: Query<(Entity, &mut EphemeralMoon)>,
    mut eject_events: EventWriter<MoonEjectedEvent>,
) {
    // Assuming 1 day = 100 seconds in game time for simplicity.
    // Builder should use actual constant.
    let day_delta = time.delta_secs() / 100.0;

    for (entity, mut moon) in moons.iter_mut() {
        moon.days_remaining -= day_delta;
        if moon.days_remaining <= 0.0 {
            eject_events.send(MoonEjectedEvent { entity });
            commands.entity(entity).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Time Scaling**: The hardcoded `100.0` seconds per day in `decay_ephemeral_moons_system` should be extracted into a global `TimeScale` resource or constant defined in `layer1::time` or similar.
- **Modifier Stacking**: `apply_moon_modifiers_system` currently overwrites `current_yield` entirely. If other systems (like Smog or Degradation) also modify solar yield, this will cause conflicts. Use a component like `SolarYieldModifiers` that aggregates multipliers before calculating the final yield.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes cleanly.
- [ ] Test coverage for new code is >= 85%.
- [ ] Integration with existing `Time` systems is logical and non-conflicting.

## 7. Technical Guidance
- Ensure `apply_moon_modifiers_system` is correctly ordered in the schedule (after base yields are calculated, but before consumption logic).
- Multiple moons could technically be captured simultaneously. Ensure the modifier logic handles overlapping (e.g., capping the max boost) if multiple `Bright` moons exist.

## 8. Questions
*Builder: Add questions here about specific visual UI elements for Ephemeral Moons, or whether they show up on the Layer 2 map.*
