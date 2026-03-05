# 316: The Geothermal Heartbeat

## 1. Overview

Geothermal vents provide massive, clean energy to the colony, but their output is not constant. Instead, it "beats" on a slow cycle, alternating between periods of high power and low power. Additionally, building near them during the high-power phase risks "Lava Surges" that can damage structures. This mechanic introduces a new strategic layer where players must balance consistent but low-yield power sources against massive but cyclical and dangerous geothermal power.

## 2. Dependencies

- `042` Energy System (must exist to provide power)
- `006` Building Placement (must exist to place buildings)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_geothermal_heartbeat_cycle() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_geothermal_vents);

        let vent = app.world_mut().spawn((
            GeothermalVent {
                high_power_duration: 300,
                low_power_duration: 200,
                current_tick: 0,
                is_high_power: true,
                high_power_output: 1000.0,
                low_power_output: 100.0,
            },
            EnergySource { output: 0.0 },
        )).id();

        // Act & Assert - High Power Phase
        app.update();
        let energy = app.world().get::<EnergySource>(vent).unwrap();
        assert_eq!(energy.output, 1000.0);

        // Fast forward to low power phase
        let mut vent_mut = app.world_mut().get_mut::<GeothermalVent>(vent).unwrap();
        vent_mut.current_tick = 300;

        app.update();
        let energy_low = app.world().get::<EnergySource>(vent).unwrap();
        let vent_state = app.world().get::<GeothermalVent>(vent).unwrap();
        assert_eq!(vent_state.is_high_power, false);
        assert_eq!(energy_low.output, 100.0);
    }

    #[test]
    fn test_lava_surge_damage() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_lava_surges);

        let vent = app.world_mut().spawn((
            GeothermalVent {
                high_power_duration: 300,
                low_power_duration: 200,
                current_tick: 150,
                is_high_power: true,
                high_power_output: 1000.0,
                low_power_output: 100.0,
            },
            GridPosition { x: 10, y: 10 },
        )).id();

        let building = app.world_mut().spawn((
            GridPosition { x: 10, y: 11 },
            Health { current: 100.0, max: 100.0 },
            Structure,
        )).id();

        // Act
        // Simulate a tick where a surge occurs
        app.world_mut().insert_resource(LavaSurgeChance(1.0)); // 100% chance for testing
        app.update();

        // Assert
        let health = app.world().get::<Health>(building).unwrap();
        assert!(health.current < 100.0, "Building should have taken damage from lava surge");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct GeothermalVent {
    pub high_power_duration: u32,
    pub low_power_duration: u32,
    pub current_tick: u32,
    pub is_high_power: bool,
    pub high_power_output: f32,
    pub low_power_output: f32,
}

#[derive(Component)]
pub struct EnergySource {
    pub output: f32,
}

#[derive(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct Structure;

#[derive(Resource)]
pub struct LavaSurgeChance(pub f32);

pub fn process_geothermal_vents(mut query: Query<(&mut GeothermalVent, &mut EnergySource)>) {
    for (mut vent, mut energy) in query.iter_mut() {
        vent.current_tick += 1;

        let phase_duration = if vent.is_high_power { vent.high_power_duration } else { vent.low_power_duration };

        if vent.current_tick >= phase_duration {
            vent.current_tick = 0;
            vent.is_high_power = !vent.is_high_power;
        }

        energy.output = if vent.is_high_power { vent.high_power_output } else { vent.low_power_output };
    }
}

pub fn process_lava_surges(
    vent_query: Query<(&GeothermalVent, &GridPosition)>,
    mut structure_query: Query<(&GridPosition, &mut Health), With<Structure>>,
    surge_chance: Res<LavaSurgeChance>,
) {
    for (vent, vent_pos) in vent_query.iter() {
        if vent.is_high_power && rand::random::<f32>() < surge_chance.0 {
            // Apply damage to adjacent structures
            for (struct_pos, mut health) in structure_query.iter_mut() {
                let dx = (vent_pos.x - struct_pos.x).abs();
                let dy = (vent_pos.y - struct_pos.y).abs();

                if dx <= 1 && dy <= 1 {
                    health.current -= 25.0; // Minimal hardcoded damage
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Extract Damage Logic:** The damage logic for `process_lava_surges` should be converted into an event `DamageEvent` so the core health system handles it, preventing duplicated logic.
- **Chebyshev Distance:** Use a standardized utility function for checking grid adjacency rather than hardcoding `dx <= 1 && dy <= 1`.
- **RNG Resource:** Use a deterministic RNG resource (like `GlobalRng`) rather than `rand::random()` for simulation determinism.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Geothermal vents alternate between high and low power phases accurately.
- [ ] Lava surges only happen during high power phases and damage adjacent structures.

## 7. Technical Guidance

- Implement `GeothermalVent` and its systems in `src/layer1/power/geothermal.rs`.
- Ensure `process_geothermal_vents` runs before the global power grid calculations so the output is accurate for the current tick.
- The `LavaSurgeChance` could be tied to Planetary Quirks or modified by specific technologies later, so make it a configurable resource.

## 8. Questions

*Builder: add questions here if spec is unclear.*
