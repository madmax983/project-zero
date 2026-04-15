# 918: Chronal Anomalies

## 1. Overview
Chronal Anomalies introduce localized zones on Layer 1 where the flow of time (`TimeScale`) is accelerated or decelerated. This alters the rates of growth, healing, decay, and machine wear. It forces a trade-off: fast production at the cost of rapid maintenance, or slow deterioration for stable environments.

## 2. Dependencies
- `geography.rs` (Layer 1 tile and zone data)
- Core tick/time system that updates entity states based on delta time.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_fast_time_zone_accelerates_growth() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(Time::default());
        app.add_systems(Update, process_chronal_growth_system);

        let crop_in_fast_zone = app.world_mut().spawn((
            Crop { growth_progress: 0.0, max_growth: 100.0 },
            ChronalZone { time_scale: 2.0 } // 2x speed
        )).id();

        let normal_crop = app.world_mut().spawn((
            Crop { growth_progress: 0.0, max_growth: 100.0 },
            ChronalZone { time_scale: 1.0 } // Normal speed
        )).id();

        // Act
        // Simulate a 1-second tick
        let mut time = app.world_mut().resource_mut::<Time>();
        // Using a mock time step for the test
        let dt = 1.0;

        app.update();

        // Assert
        let fast_crop = app.world().get::<Crop>(crop_in_fast_zone).unwrap();
        let norm_crop = app.world().get::<Crop>(normal_crop).unwrap();

        assert!(fast_crop.growth_progress > norm_crop.growth_progress, "Crop in fast zone should grow faster.");
        // We expect fast crop to be exactly double if we mock dt properly
    }

    #[test]
    fn test_slow_time_zone_delays_decay() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(Time::default());
        app.add_systems(Update, process_chronal_decay_system);

        let item_in_slow_zone = app.world_mut().spawn((
            Spoilage { current_spoilage: 0.0, max_spoilage: 100.0 },
            ChronalZone { time_scale: 0.5 } // Half speed
        )).id();

        let normal_item = app.world_mut().spawn((
            Spoilage { current_spoilage: 0.0, max_spoilage: 100.0 },
            ChronalZone { time_scale: 1.0 } // Normal speed
        )).id();

        // Act
        app.update();

        // Assert
        let slow_item = app.world().get::<Spoilage>(item_in_slow_zone).unwrap();
        let norm_item = app.world().get::<Spoilage>(normal_item).unwrap();

        assert!(slow_item.current_spoilage < norm_item.current_spoilage, "Item in slow zone should spoil slower.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct ChronalZone {
    pub time_scale: f32,
}

#[derive(Component)]
pub struct Crop {
    pub growth_progress: f32,
    pub max_growth: f32,
}

#[derive(Component)]
pub struct Spoilage {
    pub current_spoilage: f32,
    pub max_spoilage: f32,
}

pub fn process_chronal_growth_system(
    mut query: Query<(&mut Crop, &ChronalZone)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs(); // In tests we might need to mock this carefully
    for (mut crop, zone) in query.iter_mut() {
        let scaled_dt = dt * zone.time_scale;
        let growth_rate = 10.0; // Arbitrary base rate
        crop.growth_progress += growth_rate * scaled_dt;
        if crop.growth_progress > crop.max_growth {
            crop.growth_progress = crop.max_growth;
        }
    }
}

pub fn process_chronal_decay_system(
    mut query: Query<(&mut Spoilage, &ChronalZone)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut spoilage, zone) in query.iter_mut() {
        let scaled_dt = dt * zone.time_scale;
        let decay_rate = 5.0; // Arbitrary base rate
        spoilage.current_spoilage += decay_rate * scaled_dt;
        if spoilage.current_spoilage > spoilage.max_spoilage {
            spoilage.current_spoilage = spoilage.max_spoilage;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Instead of adding `ChronalZone` directly to entities, the system should ideally check the tile the entity is standing on and apply a time modifier resource/effect.
- Integrate with the core Time resource to provide a `ScaledTime` resource, avoiding the need to manually multiply `dt * zone.time_scale` in every system (which scales poorly).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Entities within anomalous zones correctly process their logic at the adjusted rate.

## 7. Technical Guidance
- Be careful with Bevy's default `Time` resource in tests. Initialize components with their required mock time progress if testing `just_finished` logic.

## 8. Questions
*Builder: Will we need a new UI shader to represent these zones visually to the player without cluttering the main map?*
*Architect:* No new shader is needed for the MVP. A simple tinted overlay or particle effect over the affected tiles will suffice to avoid clutter.
