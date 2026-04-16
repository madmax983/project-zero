# 1054: Thermal Camouflage

## 1. Overview
Detection systems and predatory fauna rely on thermal signatures. Pops generate heat through activities ("Running Hot" when working or shooting) which increases their detection radius. Being idle or entering Cryo ("Running Cold") reduces their signature, making them virtually invisible. This introduces a tension between action (Heat) and safety (Cold), allowing for tense scenarios where shutting down life support to freeze is the only way to survive a hunt.

## 2. Dependencies
- Base simulation framework (`App`, `World`)
- `GridPosition` for spatial reasoning
- Environmental systems (`Temperature` or similar heat representation)
- Fauna/Detection AI

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_activity_generates_thermal_signature() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, update_thermal_signatures);

        let idle_pop = app.world_mut().spawn((
            PopAction { state: ActionState::Idle },
            ThermalSignature { current_heat: 30.0, detection_radius: 0.0 },
        )).id();

        let working_pop = app.world_mut().spawn((
            PopAction { state: ActionState::Working },
            ThermalSignature { current_heat: 30.0, detection_radius: 0.0 },
        )).id();

        // Act
        app.update();

        // Assert
        let idle_sig = app.world().get::<ThermalSignature>(idle_pop).unwrap();
        let work_sig = app.world().get::<ThermalSignature>(working_pop).unwrap();

        assert!(work_sig.current_heat > idle_sig.current_heat);
        assert!(work_sig.detection_radius > idle_sig.detection_radius);
    }

    #[test]
    fn test_predator_detects_hot_targets() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, predator_detection_system);

        let predator = app.world_mut().spawn((
            GridPosition { x: 0, y: 0 },
            ThermalSensor { sensitivity: 5.0, targets: vec![] },
        )).id();

        let hot_pop = app.world_mut().spawn((
            GridPosition { x: 5, y: 0 },
            ThermalSignature { current_heat: 100.0, detection_radius: 10.0 },
        )).id();

        let cold_pop = app.world_mut().spawn((
            GridPosition { x: 2, y: 0 }, // Closer, but cold
            ThermalSignature { current_heat: 10.0, detection_radius: 1.0 },
        )).id();

        // Act
        app.update();

        // Assert: Predator detects hot_pop but not cold_pop
        let sensor = app.world().get::<ThermalSensor>(predator).unwrap();
        assert!(sensor.targets.contains(&hot_pop));
        assert!(!sensor.targets.contains(&cold_pop));
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, PartialEq)]
pub enum ActionState {
    Idle,
    Working,
    Shooting,
}

#[derive(Component)]
pub struct PopAction {
    pub state: ActionState,
}

#[derive(Component)]
pub struct ThermalSignature {
    pub current_heat: f32,
    pub detection_radius: f32,
}

#[derive(Component)]
pub struct ThermalSensor {
    pub sensitivity: f32,
    pub targets: Vec<Entity>,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub fn distance(&self, other: &GridPosition) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        ((dx * dx + dy * dy) as f32).sqrt()
    }
}

pub fn update_thermal_signatures(
    mut query: Query<(&PopAction, &mut ThermalSignature)>,
) {
    for (action, mut sig) in query.iter_mut() {
        match action.state {
            ActionState::Idle => {
                sig.current_heat = (sig.current_heat - 1.0).max(10.0);
            }
            ActionState::Working => {
                sig.current_heat = (sig.current_heat + 5.0).min(100.0);
            }
            ActionState::Shooting => {
                sig.current_heat = (sig.current_heat + 20.0).min(200.0);
            }
        }

        // Radius scales with heat
        sig.detection_radius = sig.current_heat * 0.1;
    }
}

pub fn predator_detection_system(
    mut predators: Query<(&GridPosition, &mut ThermalSensor)>,
    targets: Query<(Entity, &GridPosition, &ThermalSignature)>,
) {
    for (pred_pos, mut sensor) in predators.iter_mut() {
        sensor.targets.clear();
        for (target_ent, target_pos, target_sig) in targets.iter() {
            let dist = pred_pos.distance(target_pos);

            // If the target's thermal radius reaches the predator's sensor
            if dist <= target_sig.detection_radius + sensor.sensitivity {
                sensor.targets.push(target_ent);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Ambient Temperature Integration**: `current_heat` should trend towards the ambient temperature of the tile (Grid Temp) rather than an arbitrary minimum. If a pop turns off life support, their environment gets cold, allowing their heat to drop further.
- **Equipment Modifiers**: Add support for thermal suits that dampen `detection_radius` without reducing internal `current_heat`.
- **Spatial Optimization**: The O(N*M) detection loop is fine for MVP but should utilize spatial hashing/quadtrees if predator/target counts scale up.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage for new code is >= 85%.
- [ ] Activity state dynamically modifies thermal signatures.
- [ ] Predators acquire targets based on thermal radius overlap.

## 7. Technical Guidance
- Ambient temperature linking is key for the "freeze to hide" fantasy. You can simulate this by querying an `AtmosphereGrid` or `TemperatureGrid` resource if it exists.
- Ensure the detection system correctly updates/clears old targets if they cool down and fall out of sensor range.

## 8. Questions
*Builder: add questions here if spec is unclear.*
