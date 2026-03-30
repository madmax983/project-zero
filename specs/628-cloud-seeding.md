# 628: Cloud Seeding

## 1. Overview
Cloud Seeding allows players to launch rockets (Layer 1) or ships (Layer 2) to detonate chemical payloads in the atmosphere. This artificial manipulation triggers Rain/Snow to clear pollution or extinguish fires. However, seeding a dirty atmosphere risks creating Toxic Rain, causing sickness and acid damage. It plays god with the weather but adds significant risk.

## 2. Dependencies
- `033` Fire Propagation
- `063` Atmospheric Simulation
- `079` Weather Events

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_cloud_seeding_triggers_rain_and_extinguishes_fire() {
    // Arrange: Setup test data
    let mut app = App::new();
    app.insert_resource(Atmosphere { pollution: 10 });
    let tile = app.world.spawn(Fire).id();

    // Act: Call the feature
    trigger_cloud_seeding(&mut app.world);

    // Assert: Verify expected behavior
    assert!(app.world.get_resource::<WeatherEvent>().unwrap().is_rain());
    assert!(app.world.get_entity(tile).is_none());
}

#[test]
fn test_cloud_seeding_in_high_pollution_creates_toxic_rain() {
    // Test boundary conditions
    let mut app = App::new();
    app.insert_resource(Atmosphere { pollution: 90 });

    // Act
    trigger_cloud_seeding(&mut app.world);

    // Assert
    let weather = app.world.get_resource::<WeatherEvent>().unwrap();
    assert!(weather.is_toxic_rain());
}

#[test]
fn test_toxic_rain_damages_exposed_buildings_and_pops() {
    // Test boundary conditions
    let mut app = App::new();
    app.insert_resource(WeatherEvent::ToxicRain);
    let pop = app.world.spawn((Pop, Health(100), Unshielded)).id();

    // Act
    apply_weather_effects(&mut app.world);

    // Assert
    let health = app.world.get::<Health>(pop).unwrap();
    assert!(health.0 < 100);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN

pub struct Atmosphere { pub pollution: u32 }
pub enum WeatherEvent { Rain, ToxicRain, Clear }
impl WeatherEvent {
    pub fn is_rain(&self) -> bool { matches!(self, WeatherEvent::Rain) }
    pub fn is_toxic_rain(&self) -> bool { matches!(self, WeatherEvent::ToxicRain) }
}
pub struct Fire;
pub struct Pop;
pub struct Health(pub u32);
pub struct Unshielded;

pub fn trigger_cloud_seeding(world: &mut World) {
    let atmosphere = world.get_resource::<Atmosphere>().unwrap();
    if atmosphere.pollution > 50 {
        world.insert_resource(WeatherEvent::ToxicRain);
    } else {
        world.insert_resource(WeatherEvent::Rain);
        // Clear all fires simply
        let mut query = world.query::<Entity, With<Fire>>();
        let fires: Vec<Entity> = query.iter(world).collect();
        for fire in fires {
            world.despawn(fire);
        }
    }
}

pub fn apply_weather_effects(world: &mut World) {
    if let Some(weather) = world.get_resource::<WeatherEvent>() {
        if weather.is_toxic_rain() {
            let mut query = world.query::<&mut Health, With<Unshielded>>();
            for mut health in query.iter_mut(world) {
                health.0 -= 10;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** Do not simply `despawn` fires instantly. Rain should apply a heavy cooling effect that iterates the fire out naturally over a few ticks using the existing `033` Fire Propagation logic.
- **Code Smells:** Direct iteration and despawning is a bit harsh and skips event pipelines; use `ExtinguishEvent` instead.
- **Design:** Ensure toxic rain leaves temporary puddles of `Toxin` that must be cleaned by Pops, or slowly decays.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- The trigger for `trigger_cloud_seeding` should be a player command activating a payload via a rocket silo or orbital ship.
- Ensure Pops will pathfind indoors to avoid `ToxicRain` when active.
- Toxic Rain should slowly reduce global `Atmosphere.pollution` levels as it "scrubs" the air but deposits the toxins on the ground.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
