# Specification: Indoor Weather (Feature 807)

## 1. Overview
The **Indoor Weather** feature simulates macro-climates inside massive enclosed structures. Mega-factories or Bio-Domes now track internal humidity and temperature. Poor ventilation leads to adverse conditions like "Indoor Rain" (which causes corrosion to machinery and slipping hazards for pops) or "Fog" (which creates sight and efficiency penalties). This introduces a tension between efficient, open-plan factory layouts and compartmentalized, climate-controlled designs.

## 2. Dependencies
- `Layer 1 System`
- `Atmosphere/Temperature Simulation System` (from nature)
- `Structure/Building components`
- `Pop Components` (Mood/Efficiency)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_enclosed_structure_tracks_internal_climate() {
        let mut app = App::new();
        app.add_systems(Update, update_indoor_weather_system);

        // Arrange
        let entity = app.world_mut().spawn((
            EnclosedStructure { size: 100 },
            InternalClimate { humidity: 0.0, temperature: 20.0, ventilation_rate: 0.1 },
            MachineryHeatOutput { heat_per_tick: 5.0 }
        )).id();

        // Act
        app.update();

        // Assert
        let climate = app.world().get::<InternalClimate>(entity).unwrap();
        assert!(climate.temperature > 20.0, "Machinery heat should increase internal temperature");
    }

    #[test]
    fn test_high_humidity_causes_indoor_rain() {
        let mut app = App::new();
        app.add_systems(Update, resolve_indoor_weather_effects_system);

        // Arrange
        let entity = app.world_mut().spawn((
            EnclosedStructure { size: 100 },
            InternalClimate { humidity: 100.0, temperature: 15.0, ventilation_rate: 0.0 }
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<IndoorRainEffect>(entity).is_some(), "High humidity without ventilation should cause Indoor Rain");
    }

    #[test]
    fn test_indoor_rain_corrodes_machinery() {
        let mut app = App::new();
        app.add_systems(Update, apply_weather_corrosion_system);

        // Arrange
        let machinery = app.world_mut().spawn((
            Machinery { health: 100.0 },
            IndoorRainEffect
        )).id();

        // Act
        app.update();

        // Assert
        let machine = app.world().get::<Machinery>(machinery).unwrap();
        assert!(machine.health < 100.0, "Machinery should take damage under Indoor Rain effect");
    }

    #[test]
    fn test_fog_reduces_pop_efficiency() {
         let mut app = App::new();
         app.add_systems(Update, apply_weather_efficiency_system);

         // Arrange
         let pop = app.world_mut().spawn((
             Pop { efficiency: 1.0 },
             FogEffect
         )).id();

         // Act
         app.update();

         // Assert
         let pop_comp = app.world().get::<Pop>(pop).unwrap();
         assert!(pop_comp.efficiency < 1.0, "Pop efficiency should be reduced by fog");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct EnclosedStructure {
    pub size: u32,
}

#[derive(Component)]
pub struct InternalClimate {
    pub humidity: f32,
    pub temperature: f32,
    pub ventilation_rate: f32,
}

#[derive(Component)]
pub struct MachineryHeatOutput {
    pub heat_per_tick: f32,
}

#[derive(Component)]
pub struct Machinery {
    pub health: f32,
}

#[derive(Component)]
pub struct Pop {
    pub efficiency: f32,
}

#[derive(Component)]
pub struct IndoorRainEffect;

#[derive(Component)]
pub struct FogEffect;

pub fn update_indoor_weather_system(mut query: Query<(&mut InternalClimate, &MachineryHeatOutput)>) {
    for (mut climate, heat) in query.iter_mut() {
        climate.temperature += heat.heat_per_tick;
    }
}

pub fn resolve_indoor_weather_effects_system(mut commands: Commands, query: Query<(Entity, &InternalClimate)>) {
    for (entity, climate) in query.iter() {
        if climate.humidity >= 100.0 && climate.ventilation_rate == 0.0 {
            commands.entity(entity).insert(IndoorRainEffect);
        }
    }
}

pub fn apply_weather_corrosion_system(mut query: Query<&mut Machinery, With<IndoorRainEffect>>) {
    for mut machinery in query.iter_mut() {
        machinery.health -= 1.0;
    }
}

pub fn apply_weather_efficiency_system(mut query: Query<&mut Pop, With<FogEffect>>) {
    for mut pop in query.iter_mut() {
        pop.efficiency -= 0.1;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Centralize weather conditions into a state enum (e.g., `IndoorWeatherState::Clear`, `IndoorWeatherState::Rain`, `IndoorWeatherState::Fog`) instead of relying on discrete marker components for better state management.
- Calculate ventilation dynamically based on adjacent structures and ventilation upgrades.
- Allow pops to equip "weather gear" to mitigate Fog or Rain penalties.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Internal climate updates correctly when exposed to heat and moisture.
- [ ] Indoor weather effects naturally trigger and penalize efficiency/health when conditions are met.

## 7. Technical Guidance
- Integration point: Hook the `InternalClimate` into the general energy and structural loop so that running more power-intensive machines naturally drives up temperature.
- Be careful with `app.update()` in the tests; despawns and inserts require an update tick to process commands. Ensure sufficient ticks when asserting entity states.
- Be sure to update SEAM_MAP and COMPLETED when hooking up these events to observation streams.

## 8. Questions
*Builder: add questions here if spec is unclear.*
