# 302 - The Solar Sail Migration

## 1. Overview
The Solar Sail Migration introduces massive "Solar Sail" nomad fleets drifting through the system (Layer 2). When they pass near the colony's planet, their colossal sails reflect intense sunlight down to the surface (Layer 1). This causes temporary extreme heatwaves across the map but simultaneously provides massive boosts to Solar Power generation. The event challenges players to balance free energy against the dangers of overheating, fires, and crop failure.

## 2. Dependencies
- `042` Energy System (for Solar Power)
- `140` Thermal Management (for heatwaves)
- `094` System View Architecture (for Layer 2 fleet representation)
- `065` Day/Night Cycle (to interact with sunlight intensity)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_solar_sail_transit_starts_event() {
        // Arrange: Fleet moving into planet's orbit
        let mut app = App::new();
        app.add_systems(Update, check_sail_fleet_proximity);
        app.insert_resource(SolarMigrationState { is_active: false, intensity_multiplier: 1.0 });

        // Spawn planet and fleet nearby
        let planet = app.world_mut().spawn((Planet, Position { x: 0.0, y: 0.0 })).id();
        let fleet = app.world_mut().spawn((SolarSailFleet, Position { x: 5.0, y: 5.0 })).id();

        // Act: Run system
        app.update();

        // Assert: Migration event active with an intensity > 1.0
        let state = app.world().resource::<SolarMigrationState>();
        assert!(state.is_active, "Migration event should be active.");
        assert!(state.intensity_multiplier > 1.0, "Sunlight intensity should be multiplied.");
    }

    #[test]
    fn test_solar_sail_increases_solar_power_and_heat() {
        // Arrange: Active migration state
        let mut app = App::new();
        app.insert_resource(SolarMigrationState { is_active: true, intensity_multiplier: 3.0 });
        app.add_systems(Update, apply_solar_sail_effects);

        let solar_panel = app.world_mut().spawn((SolarPanel { base_output: 10.0 }, PowerGenerator { current_output: 10.0 })).id();
        let surface_tile = app.world_mut().spawn((SurfaceTile, Temperature { current: 20.0 })).id();

        // Act: Apply effects
        app.update();

        // Assert: Power output tripled, Temperature increased
        let power = app.world().get::<PowerGenerator>(solar_panel).unwrap().current_output;
        assert_eq!(power, 30.0, "Power output should be multiplied.");

        let temp = app.world().get::<Temperature>(surface_tile).unwrap().current;
        assert!(temp > 20.0, "Surface temperature should increase.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// --- Components and Resources ---
#[derive(Component)]
pub struct Planet;

#[derive(Component)]
pub struct SolarSailFleet;

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct SolarPanel {
    pub base_output: f32,
}

#[derive(Component)]
pub struct PowerGenerator {
    pub current_output: f32,
}

#[derive(Component)]
pub struct SurfaceTile;

#[derive(Component)]
pub struct Temperature {
    pub current: f32,
}

#[derive(Resource)]
pub struct SolarMigrationState {
    pub is_active: bool,
    pub intensity_multiplier: f32,
}

// --- Systems ---
pub fn check_sail_fleet_proximity(
    mut state: ResMut<SolarMigrationState>,
    planet_query: Query<&Position, With<Planet>>,
    fleet_query: Query<&Position, With<SolarSailFleet>>,
) {
    if let Ok(planet_pos) = planet_query.get_single() {
        let mut active = false;
        let mut intensity = 1.0;

        for fleet_pos in fleet_query.iter() {
            let distance = ((planet_pos.x - fleet_pos.x).powi(2) + (planet_pos.y - fleet_pos.y).powi(2)).sqrt();
            if distance < 50.0 { // Threshold distance
                active = true;
                intensity = 1.0 + (50.0 - distance) * 0.1; // Scale intensity based on closeness
            }
        }

        state.is_active = active;
        state.intensity_multiplier = intensity.max(1.0);
    }
}

pub fn apply_solar_sail_effects(
    state: Res<SolarMigrationState>,
    mut power_query: Query<(&SolarPanel, &mut PowerGenerator)>,
    mut temp_query: Query<&mut Temperature, With<SurfaceTile>>,
) {
    if !state.is_active {
        // Reset power outputs
        for (panel, mut gen) in power_query.iter_mut() {
            gen.current_output = panel.base_output;
        }
        return;
    }

    // Apply multiplier to solar panels
    for (panel, mut gen) in power_query.iter_mut() {
        gen.current_output = panel.base_output * state.intensity_multiplier;
    }

    // Increment surface temperature (needs to be balanced with normal cooling in full game)
    for mut temp in temp_query.iter_mut() {
        temp.current += 1.0 * state.intensity_multiplier; // Simplified arbitrary heat addition
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactor Opportunity:** The distance calculation and threshold (`50.0`) should be configurable in a `SolarSailConfig` resource.
- **Refactor Opportunity:** Instead of iterating all `SurfaceTile` entities, integrate this with the `Atmospheric Simulation` (`063`) or `Thermal Management` (`140`) grid systems to apply a global heat influx modifier.
- **Design Improvement:** Allow the player to launch countermeasures (e.g., reflective dust) to mitigate the heat at the cost of losing the solar power bonus.
- **Integration:** The UI must display an "Intense Sunlight" alert when `SolarMigrationState::is_active` is true.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.
- [ ] The migration state activates when fleets are near the planet.
- [ ] Active migrations multiply solar power generation.
- [ ] Active migrations increase surface tile temperatures globally.

## 7. Technical Guidance
- **ECS Pattern:** The `SolarMigrationState` acts as the bridge between Layer 2 (Fleet movement) and Layer 1 (Colony temperature and power).
- **Seams:** Ensure the temperature increase hooks into the existing `TemperatureGrid` update logic rather than overwriting entity values directly if a grid is used.
- **Balance:** The heat increase should be significant enough to cause fires (`033`) or heatstroke (`034`) if the colony is unprepared (e.g., passing during summer).

## 8. Questions
*Builder: add questions here if spec is unclear.*
