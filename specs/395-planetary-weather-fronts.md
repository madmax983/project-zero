# 395 - Planetary Weather Fronts

## 1. Overview
Planetary Weather Fronts introduce dynamic, moving weather systems that exist on Layer 2 (System) and affect Layer 1 (Colony) when passing overhead. Storms, droughts, or heatwaves move across the planet surface over time. When a front intersects the colony's coordinates, it applies map-wide effects such as wind damage, crop failure, or movement penalties. This forces players to adapt their infrastructure to shifting, predictable weather patterns.

## 2. Dependencies
- `094-system-view.md` (Layer 2 planet maps)
- `063-atmospheric-simulation.md` (Layer 1 atmospheric conditions)
- `079-weather-events.md` (Local weather effects)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::planet::{Planet, ColonyLocation};
    use crate::layer1::weather::LocalWeather;

    #[test]
    fn test_weather_front_movement_layer2() {
        let mut app = App::new();
        app.add_systems(Update, update_weather_fronts_l2);

        // Spawn a weather front on Layer 2
        let front = app.world_mut().spawn((
            WeatherFront {
                weather_type: WeatherType::DustStorm,
                velocity: Vec2::new(1.0, 0.0), // Moves East
                radius: 5.0,
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        app.update();

        // Front should have moved
        let transform = app.world().get::<Transform>(front).unwrap();
        assert!(transform.translation.x > 0.0, "Weather front should move across the planet surface");
    }

    #[test]
    fn test_front_intersection_applies_layer1_weather() {
        let mut app = App::new();
        app.add_systems(Update, sync_weather_l2_to_l1);

        // Define colony location on L2
        app.insert_resource(ColonyLocation(Vec2::new(10.0, 10.0)));
        app.insert_resource(LocalWeather::default());

        // Spawn a storm directly over the colony
        app.world_mut().spawn((
            WeatherFront {
                weather_type: WeatherType::DustStorm,
                velocity: Vec2::ZERO,
                radius: 5.0,
            },
            Transform::from_xyz(10.0, 10.0, 0.0), // Intersects
        ));

        app.update();

        let local = app.world().get_resource::<LocalWeather>().unwrap();
        assert_eq!(local.current_type, WeatherType::DustStorm, "Colony should inherit L2 weather front overhead");
    }

    #[test]
    fn test_front_leaves_colony() {
        let mut app = App::new();
        app.add_systems(Update, sync_weather_l2_to_l1);

        app.insert_resource(ColonyLocation(Vec2::new(10.0, 10.0)));
        app.insert_resource(LocalWeather { current_type: WeatherType::DustStorm });

        // Spawn a storm FAR AWAY from the colony
        app.world_mut().spawn((
            WeatherFront {
                weather_type: WeatherType::DustStorm,
                velocity: Vec2::ZERO,
                radius: 5.0,
            },
            Transform::from_xyz(100.0, 100.0, 0.0), // Does not intersect
        ));

        app.update();

        let local = app.world().get_resource::<LocalWeather>().unwrap();
        assert_eq!(local.current_type, WeatherType::Clear, "Colony should return to Clear weather when front passes");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum WeatherType {
    #[default]
    Clear,
    DustStorm,
    Heatwave,
}

#[derive(Component)]
pub struct WeatherFront {
    pub weather_type: WeatherType,
    pub velocity: Vec2,
    pub radius: f32,
}

#[derive(Resource)]
pub struct ColonyLocation(pub Vec2);

#[derive(Resource, Default)]
pub struct LocalWeather {
    pub current_type: WeatherType,
}

pub fn update_weather_fronts_l2(
    time: Res<Time>,
    mut query: Query<(&WeatherFront, &mut Transform)>,
) {
    for (front, mut transform) in query.iter_mut() {
        transform.translation.x += front.velocity.x * time.delta_secs();
        transform.translation.y += front.velocity.y * time.delta_secs();
    }
}

pub fn sync_weather_l2_to_l1(
    colony_loc: Res<ColonyLocation>,
    mut local_weather: ResMut<LocalWeather>,
    fronts: Query<(&WeatherFront, &Transform)>,
) {
    let mut new_weather = WeatherType::Clear;

    for (front, transform) in fronts.iter() {
        let front_pos = Vec2::new(transform.translation.x, transform.translation.y);
        if front_pos.distance(colony_loc.0) <= front.radius {
            new_weather = front.weather_type;
            break; // Colony is in this front
        }
    }

    local_weather.current_type = new_weather;
}
```

## 5. REFACTOR Phase: Quality & Design
- **Interpolation/Fading**: Weather shouldn't instantly snap from Clear to DustStorm. There should be a "transition" phase as the leading edge of the front approaches the colony radius.
- **Multiple Fronts**: What happens when a Heatwave and a DustStorm intersect over the colony? They should merge or define a priority system.
- **Visuals L2**: Weather fronts need a shader or visual representation on the `SystemView` map so the player can see them approaching.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥ 85% for L2 weather module
- [ ] Weather fronts move on the L2 map over time.
- [ ] The L1 `LocalWeather` accurately reflects when an L2 front is overhead.

## 7. Technical Guidance
- Integrate with `layer2::system_view.rs` for spawning and moving fronts.
- The actual effects of `LocalWeather` on Pops (e.g., wind damage) are likely handled by `layer1/weather.rs` (Spec 079). This spec bridges L2 moving entities with the L1 global state.

## 8. Questions
*Builder: add questions here if spec is unclear.*
