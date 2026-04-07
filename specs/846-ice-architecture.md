# Feature Specification: Ice Architecture (846)

## 1. Overview
**Layer**: 1 (Colony Layer)
**Fantasy**: Building with the season. The ultimate temporary housing.
**Mechanic**: "Ice" is a buildable material in freezing biomes. It is free and fast to build but melts if Temperature > 0C.
**Emergence**: You build a cheap ice-wall against the winter raid. Spring comes early. The wall melts during the night. The wolves get in.
**Tension**: Cheap/Temporary vs. Expensive/Permanent.

## 2. Dependencies
- `Temperature Grid` (for determining cell temperature)
- `Building System` (for material types and construction)
- `Season System` (drives temperature changes)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // Dummy structs for compilation
    #[derive(Component, PartialEq)]
    enum Material {
        Wood,
        Stone,
        Ice,
    }

    #[derive(Component)]
    struct Health {
        current: f32,
        max: f32,
    }

    #[derive(Component)]
    struct Building;

    #[derive(Resource)]
    struct TemperatureGrid {
        cell_temp: f32,
    }

    #[test]
    fn test_ice_building_takes_melting_damage_above_freezing() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(TemperatureGrid { cell_temp: 5.0 }); // 5C is above freezing
        app.add_systems(Update, process_ice_melting);

        let entity = app.world_mut().spawn((
            Building,
            Material::Ice,
            Health { current: 100.0, max: 100.0 },
            Transform::from_xyz(0.0, 0.0, 0.0)
        )).id();

        // Act
        app.update();

        // Assert
        let health = app.world().get::<Health>(entity).unwrap();
        assert!(health.current < 100.0); // Took damage
    }

    #[test]
    fn test_ice_building_stable_below_freezing() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(TemperatureGrid { cell_temp: -5.0 }); // -5C is below freezing
        app.add_systems(Update, process_ice_melting);

        let entity = app.world_mut().spawn((
            Building,
            Material::Ice,
            Health { current: 100.0, max: 100.0 },
            Transform::from_xyz(0.0, 0.0, 0.0)
        )).id();

        // Act
        app.update();

        // Assert
        let health = app.world().get::<Health>(entity).unwrap();
        assert_eq!(health.current, 100.0); // No damage taken
    }

    #[test]
    fn test_wood_building_ignores_temperature() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(TemperatureGrid { cell_temp: 15.0 });
        app.add_systems(Update, process_ice_melting);

        let entity = app.world_mut().spawn((
            Building,
            Material::Wood,
            Health { current: 100.0, max: 100.0 },
            Transform::from_xyz(0.0, 0.0, 0.0)
        )).id();

        // Act
        app.update();

        // Assert
        let health = app.world().get::<Health>(entity).unwrap();
        assert_eq!(health.current, 100.0); // Wood doesn't melt
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, PartialEq)]
pub enum Material {
    Wood,
    Stone,
    Ice,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct Building;

#[derive(Resource)]
pub struct TemperatureGrid {
    pub cell_temp: f32, // simplified for minimal impl
}

pub fn process_ice_melting(
    mut query: Query<(&Material, &mut Health), With<Building>>,
    temp_grid: Res<TemperatureGrid>
) {
    let melt_threshold = 0.0;

    for (material, mut health) in query.iter_mut() {
        if *material == Material::Ice {
            // Simplified: grab the global cell temp. Real impl will look up temp by transform
            if temp_grid.cell_temp > melt_threshold {
                let melt_rate = (temp_grid.cell_temp - melt_threshold) * 2.0;
                health.current -= melt_rate;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smell**: Hardcoded `melt_threshold = 0.0`.
- **Improvement**: Make melting point a property of the material so we could have "Super Ice" or other exotic temperature-sensitive materials.
- **API Change**: `TemperatureGrid` in real implementation is spatial. Need to ensure we fetch the temperature at the `Transform`'s grid cell.
- **Optimization**: Don't run the `process_ice_melting` system at all if the global average temperature is way below freezing (e.g. deep winter).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Ice buildings lose health proportionally to how far above 0C the local temperature is
- [ ] Ice buildings despawn when health reaches 0
- [ ] Non-ice buildings do not melt

## 7. Technical Guidance
- Integrate with `src/layer1/nature/temperature.rs`.
- Look at `src/layer1/building/mod.rs` to hook into building destruction events when health reaches 0.
- Multiply `melt_rate` by `time.delta_seconds()` in the final implementation.

## 8. Questions
*Builder: add questions here if spec is unclear.*
