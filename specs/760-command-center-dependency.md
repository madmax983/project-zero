# Spec 760: Command Center Dependency

## 1. Overview
**Layer:** Cross-layer
**Fantasy:** You are the Commander, but you are not omniscient. Your view of the empire depends on your sensors.
**Mechanic:** Access to the System (L2) and Galaxy (L3) maps requires a functioning, staffed "Command Center" on the Colony (L1). If the building is destroyed or unpowered, the player loses access to those views (Fog of War / UI disabled).
**Emergence:** A raid targets your Command Center. You are blind to the reinforcement fleet arriving in orbit until they actually land.
**Tension:** Protect the "Brain" (Command Center) in the deepest bunker, or put it high up for better reception (bonus)?

## 2. Dependencies
- Layer 1 Buildings (Command Center definition).
- Power and Staffing systems (Active / Staffed states).
- Global UI state for map visibility.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::power::PowerConsumer;
    use crate::layer1::building::Building;

    #[test]
    fn test_active_command_center_enables_map_view() {
        let mut app = App::new();

        let building = app.world_mut().spawn((
            Building { building_type: "command_center".into() },
            PowerConsumer { active: true },
            Staffed { current: 5, required: 5 },
        )).id();

        app.add_systems(Update, update_map_visibility_system);
        app.update();

        let visibility = app.world().resource::<MapVisibility>();
        assert!(visibility.layer_2_visible, "Map should be visible with an active, staffed command center");
    }

    #[test]
    fn test_unpowered_command_center_disables_map_view() {
        let mut app = App::new();

        // Unpowered CC
        app.world_mut().spawn((
            Building { building_type: "command_center".into() },
            PowerConsumer { active: false },
            Staffed { current: 5, required: 5 },
        ));

        // Initialize to visible so we test the disabling behavior
        app.world_mut().insert_resource(MapVisibility { layer_2_visible: true, layer_3_visible: true });

        app.add_systems(Update, update_map_visibility_system);
        app.update();

        let visibility = app.world().resource::<MapVisibility>();
        assert!(!visibility.layer_2_visible, "Map should be hidden when CC loses power");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct MapVisibility {
    pub layer_2_visible: bool,
    pub layer_3_visible: bool,
}

#[derive(Component)]
pub struct Building {
    pub building_type: String,
}

#[derive(Component)]
pub struct PowerConsumer {
    pub active: bool,
}

#[derive(Component)]
pub struct Staffed {
    pub current: u32,
    pub required: u32,
}

pub fn update_map_visibility_system(
    query: Query<(&Building, &PowerConsumer, &Staffed)>,
    mut visibility: ResMut<MapVisibility>,
) {
    let mut has_active_cc = false;

    for (building, power, staffed) in query.iter() {
        if building.building_type == "command_center" && power.active && staffed.current >= staffed.required {
            has_active_cc = true;
            break; // Found one working CC
        }
    }

    visibility.layer_2_visible = has_active_cc;
    visibility.layer_3_visible = has_active_cc;
}
```

## 5. REFACTOR Phase: Quality & Design
- **Map View Types**: Instead of global booleans, consider an Enum or Bitmask for different sensor layers, allowing future upgrades (e.g., Basic Radar vs. Deep Space Telemetry).
- **Grace Period**: If a CC blinks out of power for a split second, it shouldn't instantly blind the player. Add a small grace period or "Sensor Cache" that decays over a few ticks.
- **Reception Bonus**: The mechanic suggests a bonus for "putting it high up". If a `GridPosition.z` (altitude) or `TerrainElevation` exists, you could calculate a sensor range multiplier rather than a hard boolean toggle.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code
- [ ] UI logic responds to the `MapVisibility` resource and prevents rendering/interaction with Layer 2/3 when disabled.

## 7. Technical Guidance
- Register the `MapVisibility` resource globally.
- Update `ui/layer2.rs` or the ratatui equivalent to check the resource before drawing the orbital maps.

## 8. Questions
*Builder: add questions here if spec is unclear.*
