# 740 - The Subterranean Heat Sink

## 1. Overview
The planet is freezing, so you dig deep for warmth, but the deep is violently unstable. Lower Z-levels have naturally higher ambient temperatures. Players can route ventilation to pull heat up to the surface to save on power. However, extreme temperature differentials between adjacent tiles cause "Thermal Stress," leading to rapid structural decay and cave-ins.

## 2. Dependencies
- Layer 1 Z-level terrain and temperature grid system.
- Ventilation/Heat routing mechanics.
- Structural integrity system.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_thermal_stress_applies_damage_to_structures() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<TemperatureChangedEvent>();
        app.add_systems(Update, apply_thermal_stress_system);

        let structure = app.world_mut().spawn((
            Building,
            StructuralIntegrity { health: 100.0, max_health: 100.0 },
            GridPosition { x: 0, y: 0, z: -5 }, // Deep level
            Temperature { value: 50.0 }, // Hot
        )).id();

        app.world_mut().spawn((
            GridPosition { x: 1, y: 0, z: -5 },
            Temperature { value: -20.0 }, // Cold, simulating a routed vent or surface exposure
        ));

        // Act
        app.update();

        // Assert
        let health = app.world().get::<StructuralIntegrity>(structure).unwrap().health;
        assert!(health < 100.0, "Structure should take damage from extreme temperature differential");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct StructuralIntegrity {
    pub health: f32,
    pub max_health: f32,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Component)]
pub struct Temperature {
    pub value: f32,
}

#[derive(Event)]
pub struct TemperatureChangedEvent {
    pub entity: Entity,
    pub new_temp: f32,
}

pub fn apply_thermal_stress_system(
    mut query: Query<(&GridPosition, &Temperature, &mut StructuralIntegrity), With<Building>>,
    temp_query: Query<(&GridPosition, &Temperature)>,
) {
    let threshold = 50.0; // Minimal differential threshold
    let damage_rate = 5.0;

    for (pos, temp, mut integrity) in query.iter_mut() {
        for (adj_pos, adj_temp) in temp_query.iter() {
            // Check if adjacent (simplified Manhattan distance for MVP)
            let is_adjacent = (pos.x - adj_pos.x).abs() + (pos.y - adj_pos.y).abs() + (pos.z - adj_pos.z).abs() == 1;

            if is_adjacent {
                let diff = (temp.value - adj_temp.value).abs();
                if diff >= threshold {
                    integrity.health -= damage_rate;
                    break; // Apply damage once per tick based on any adjacent extreme
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Replace the O(N^2) query nested loop with a proper spatial hash map or grid resource lookup for adjacent tiles to improve performance.
- Move the hardcoded `threshold` and `damage_rate` into a `ThermalConfig` resource.
- Integrate with the full event system so damage only triggers on meaningful temperature shifts or at intervals, not every frame.

## 6. Acceptance Criteria (Testable!)
- [ ] RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] High temperature differentials cause structural integrity loss on adjacent tiles.

## 7. Technical Guidance
- Ensure the `apply_thermal_stress_system` runs after temperature diffusion/routing systems update the grid.
- Consider adding visual/UI warnings for "Thermal Stress" to alert the player before a collapse.

## 8. Questions
*Builder: add questions here if spec is unclear.*
