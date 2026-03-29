# 733 - Planetary Core Tap

## 1. Overview
The Planetary Core Tap represents the greed of digging too deep. It's an endgame structure that drills into the mantle, providing infinite Energy/Heat. However, it carries massive risks: "Core Destabilization" causing earthquakes, or a "Magma Breach" that fills the map from the bottom up, forcing a rush to escape before total doom.

## 2. Dependencies
- Energy/Heat resource system.
- Map generation Z-levels (for magma breach mechanics).
- Event generation (for earthquakes).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_core_tap_generates_infinite_energy() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut energy = GlobalEnergy { current: 0.0, capacity: 1000.0 };
        app.world_mut().insert_resource(energy);

        let tap_entity = app.world_mut().spawn((
            CoreTap { active: true, output: 9999.0 },
        )).id();

        // Act
        // Process core tap logic
        app.update();

        // Assert
        let res = app.world().get_resource::<GlobalEnergy>().unwrap();
        assert!(res.current >= 9999.0);
    }

    #[test]
    fn test_core_tap_risks_magma_breach() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add random source and tap
        app.world_mut().spawn((
            CoreTap { active: true, breach_chance: 0.1, ..default() },
        ));

        app.world_mut().init_resource::<Events<MagmaBreachEvent>>();

        // Act
        // Process risk logic
        app.update();

        // Assert
        let events = app.world().resource::<Events<MagmaBreachEvent>>();
        // Might not trigger every tick, but test ensures the event is queued if chance is met
        // (Mock RNG for actual tests)
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use rand::Rng;

#[derive(Resource)]
pub struct GlobalEnergy {
    pub current: f32,
    pub capacity: f32,
}

#[derive(Component, Default)]
pub struct CoreTap {
    pub active: bool,
    pub output: f32,
    pub breach_chance: f32,
}

#[derive(Event)]
pub struct MagmaBreachEvent {
    pub severity: u32,
}

pub fn process_core_taps(
    mut query: Query<&CoreTap>,
    mut energy: ResMut<GlobalEnergy>,
    mut events: EventWriter<MagmaBreachEvent>,
) {
    let mut rng = rand::thread_rng();

    for tap in query.iter() {
        if tap.active {
            energy.current += tap.output;
            energy.current = energy.current.min(energy.capacity);

            if rng.gen::<f32>() < tap.breach_chance {
                events.send(MagmaBreachEvent { severity: 1 });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Inject a seeded RNG component or resource for testing instead of `thread_rng()`.
- The `CoreTap` component should also track the depth or 'stability' of the core to gradually increase `breach_chance` over time, adding to the tension.

## 6. Acceptance Criteria (Testable!)
- [ ] Core Tap structure generates vast amounts of energy.
- [ ] Active Core Taps periodically emit `MagmaBreachEvent` or `EarthquakeEvent`.
- [ ] `cargo test` returns 0 failures with 85%+ coverage for the new feature code.
- [ ] `cargo clippy -- -D warnings` passes.

## 7. Technical Guidance
- Integrate the `MagmaBreachEvent` with the map/grid system to start turning low Z-levels into magma tiles.
- `EarthquakeEvent` should damage structures globally.

## 8. Questions
*Builder: add questions here if spec is unclear.*
