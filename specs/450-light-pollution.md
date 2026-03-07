# Specification: 450 - Light Pollution

## 1. Overview
The **Light Pollution** feature links directly with "The Overview Effect" and the general lighting system. Outdoor artificial lights reduce the efficiency of Observatories and upset nocturnal fauna. Players must choose between safety (lighting up the night to prevent monster spawns or work penalties) and science (keeping the sky dark to gather data), forcing them to build labs in dangerous, dark zones.

## 2. Dependencies
- `053` Lighting System
- `449` The Overview Effect
- `092` Antagonistic Flora/Fauna

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use scale::layer1::grid::{GridPos, GridMap};
    use scale::layer1::lighting::LightSource;

    #[test]
    fn test_light_pollution_calculation() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(LightPollutionPlugin);
        app.world.insert_resource(GridMap::new(20, 20));

        // Spawn an Observatory
        let obs_pos = GridPos::new(10, 10);
        let obs_entity = app.world.spawn((
            Observatory { efficiency: 100.0 },
            obs_pos,
        )).id();

        // Spawn a powerful outdoor light source nearby
        let light_pos = GridPos::new(10, 12);
        app.world.spawn((
            LightSource { radius: 5.0, intensity: 10.0, is_outdoor: true },
            light_pos,
        ));

        // Act: Evaluate pollution
        app.update();

        // Assert: Observatory efficiency is reduced
        let obs = app.world.get::<Observatory>(obs_entity).unwrap();
        assert!(obs.efficiency < 100.0, "Outdoor lights must reduce observatory efficiency");
    }

    #[test]
    fn test_indoor_lights_do_not_pollute() {
        let mut app = App::new();
        app.add_plugins(LightPollutionPlugin);
        app.world.insert_resource(GridMap::new(10, 10));

        let obs_pos = GridPos::new(5, 5);
        let obs_entity = app.world.spawn((
            Observatory { efficiency: 100.0 },
            obs_pos,
        )).id();

        // Spawn a powerful INDOOR light source nearby
        let light_pos = GridPos::new(5, 6);
        app.world.spawn((
            LightSource { radius: 5.0, intensity: 10.0, is_outdoor: false },
            light_pos,
        ));

        app.update();

        // Assert: Observatory efficiency is unaffected
        let obs = app.world.get::<Observatory>(obs_entity).unwrap();
        assert_eq!(obs.efficiency, 100.0, "Indoor lights should not cause sky glow");
    }

    #[test]
    fn test_nocturnal_fauna_aggression() {
        let mut app = App::new();
        app.add_plugins(LightPollutionPlugin);
        app.world.insert_resource(GridMap::new(10, 10));

        let fauna_pos = GridPos::new(8, 8);
        let fauna_entity = app.world.spawn((
            NocturnalFauna { aggression: 0.0 },
            fauna_pos,
        )).id();

        // Spawn outdoor light
        app.world.spawn((
            LightSource { radius: 10.0, intensity: 5.0, is_outdoor: true },
            GridPos::new(5, 5),
        ));

        app.update();

        // Assert: Fauna aggression increases due to light pollution
        let fauna = app.world.get::<NocturnalFauna>(fauna_entity).unwrap();
        assert!(fauna.aggression > 0.0, "Nocturnal fauna must become aggressive in light pollution");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass

#[derive(Component)]
pub struct Observatory {
    pub efficiency: f32,
}

#[derive(Component)]
pub struct LightSource {
    pub radius: f32,
    pub intensity: f32,
    pub is_outdoor: bool,
}

#[derive(Component)]
pub struct NocturnalFauna {
    pub aggression: f32,
}

#[derive(Resource, Default)]
pub struct SkyGlow {
    pub global_level: f32,
}

pub fn calculate_sky_glow_system(
    lights: Query<&LightSource>,
    mut sky_glow: ResMut<SkyGlow>,
) {
    let mut total_glow = 0.0;
    for light in lights.iter() {
        if light.is_outdoor {
            total_glow += light.intensity * (light.radius * 0.1);
        }
    }
    sky_glow.global_level = total_glow;
}

pub fn apply_light_pollution_system(
    sky_glow: Res<SkyGlow>,
    mut observatories: Query<&mut Observatory>,
    mut fauna: Query<&mut NocturnalFauna>,
) {
    // Reduce observatory efficiency
    for mut obs in observatories.iter_mut() {
        obs.efficiency = f32::max(0.0, 100.0 - sky_glow.global_level);
    }

    // Agitate nocturnal fauna
    for mut animal in fauna.iter_mut() {
        animal.aggression += sky_glow.global_level * 0.01;
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Spatial Hashing:** A single global `SkyGlow` value is too simple. Light pollution should be a localized value on the grid (e.g., using an attenuation formula like distance squared) so building an observatory far away from the city center works.
- **Occlusion:** High walls or mountains should block light pollution from bleeding into dark zones.
- **Visuals:** The sky shader or rendering pass (if applicable) should dynamically shift from a clear starry sky to a hazy orange/grey wash depending on the `SkyGlow` value above the camera.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Outdoor lights demonstrably lower Observatory efficiency.
- [ ] Nocturnal fauna aggression increases when exposed to high sky glow.

## 7. Technical Guidance
- Ensure that `LightSource` entities spawned inside rooms correctly have `is_outdoor: false`. This relies on the Room/Zone systems.
- When applying the efficiency penalty to the Observatory, ensure it limits the maximum tech output or speed of the "Observe" action defined in Spec 449.

## 8. Questions
*Builder: add questions here if spec is unclear.*
