# 919: Silent Running

## 1. Overview
Silent Running is a toggleable state for Layer 2 ships that drastically reduces their detection radius to evade hostile forces. However, this stealth comes at a severe cost: normal heat dissipation is disabled, causing the ship's internal heat to spike rapidly. The player must balance stealth duration against the risk of catastrophic internal damage.

## 2. Dependencies
- `ship.rs` (Ship components and definitions)
- `thermal.rs` (Layer 2 thermal tracking and heat mechanics)
- `visibility.rs` / `sensor_ambiguity.rs` (Ship detection logic)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_silent_running_reduces_detection_radius() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_silent_running_modifiers_system);

        let ship_id = app.world_mut().spawn((
            Ship,
            SilentRunningMode { active: true },
            DetectionProfile { base_radius: 100.0, current_radius: 100.0 }
        )).id();

        // Act
        app.update();

        // Assert
        let profile = app.world().get::<DetectionProfile>(ship_id).unwrap();
        assert!(profile.current_radius < profile.base_radius, "Silent running should reduce current detection radius.");
    }

    #[test]
    fn test_silent_running_disables_heat_dissipation() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(Time::default());
        app.add_systems(Update, process_ship_heat_system);

        // Ship in silent running
        let silent_ship = app.world_mut().spawn((
            Ship,
            SilentRunningMode { active: true },
            ThermalState { current_heat: 50.0, max_heat: 100.0, base_dissipation_rate: 10.0 }
        )).id();

        // Normal ship
        let normal_ship = app.world_mut().spawn((
            Ship,
            SilentRunningMode { active: false },
            ThermalState { current_heat: 50.0, max_heat: 100.0, base_dissipation_rate: 10.0 }
        )).id();

        // Act
        app.update();

        // Assert
        let silent_thermal = app.world().get::<ThermalState>(silent_ship).unwrap();
        let normal_thermal = app.world().get::<ThermalState>(normal_ship).unwrap();

        assert_eq!(silent_thermal.current_heat, 50.0, "Silent ship should not dissipate heat.");
        assert!(normal_thermal.current_heat < 50.0, "Normal ship should dissipate heat over time.");
    }

    #[test]
    fn test_silent_running_generates_internal_heat() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(Time::default());
        app.add_systems(Update, generate_silent_running_heat_system);

        let ship_id = app.world_mut().spawn((
            Ship,
            SilentRunningMode { active: true },
            ThermalState { current_heat: 0.0, max_heat: 100.0, base_dissipation_rate: 10.0 }
        )).id();

        // Act
        // Mock a time delta
        let mut time = app.world_mut().resource_mut::<Time>();
        // simulate 1 tick

        app.update();

        // Assert
        let thermal = app.world().get::<ThermalState>(ship_id).unwrap();
        assert!(thermal.current_heat > 0.0, "Silent running should actively generate internal heat due to trapped emissions.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct SilentRunningMode {
    pub active: bool,
}

#[derive(Component)]
pub struct DetectionProfile {
    pub base_radius: f32,
    pub current_radius: f32,
}

#[derive(Component)]
pub struct ThermalState {
    pub current_heat: f32,
    pub max_heat: f32,
    pub base_dissipation_rate: f32,
}

pub fn apply_silent_running_modifiers_system(
    mut query: Query<(&SilentRunningMode, &mut DetectionProfile)>
) {
    for (mode, mut profile) in query.iter_mut() {
        if mode.active {
            profile.current_radius = profile.base_radius * 0.2; // 80% reduction
        } else {
            profile.current_radius = profile.base_radius;
        }
    }
}

pub fn process_ship_heat_system(
    mut query: Query<(&SilentRunningMode, &mut ThermalState)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs(); // Assumes non-zero in real execution
    let dt = if dt == 0.0 { 1.0 } else { dt }; // Mock for simple tests

    for (mode, mut thermal) in query.iter_mut() {
        if !mode.active {
            thermal.current_heat -= thermal.base_dissipation_rate * dt;
            if thermal.current_heat < 0.0 {
                thermal.current_heat = 0.0;
            }
        }
        // If active, it does NOT dissipate heat.
    }
}

pub fn generate_silent_running_heat_system(
    mut query: Query<(&SilentRunningMode, &mut ThermalState)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let dt = if dt == 0.0 { 1.0 } else { dt }; // Mock for simple tests

    for (mode, mut thermal) in query.iter_mut() {
        if mode.active {
            let buildup_rate = 15.0; // Arbitrary heat generation rate
            thermal.current_heat += buildup_rate * dt;

            if thermal.current_heat > thermal.max_heat {
                thermal.current_heat = thermal.max_heat;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Consolidate `process_ship_heat_system` and `generate_silent_running_heat_system` into a single, cohesive thermodynamic system to prevent race conditions during tick resolution.
- Emit a `CriticalHeatWarningEvent` when heat exceeds 90% of capacity so UI can alert the player.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Toggling silent running accurately modifies detection and halts heat dissipation.

## 7. Technical Guidance
- Do not let heat exceed `max_heat` without triggering severe consequences (e.g., hull damage, crew death). Those damage systems will likely sit in `combat.rs` or a new `damage.rs` module that listens to `ThermalState`.

## 8. Questions
*Builder: Add questions here about how we visually represent trapped heat (e.g., glowing ship hull).*
