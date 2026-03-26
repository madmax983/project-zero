# 614: Solar Cycles

## 1. Overview
The local star dictates survival on Layer 1. The star cycles between "Solar Maximum" and "Solar Minimum" over time. During Solar Maximum, colonies benefit from high solar power generation but suffer from increased radiation (sickness/mutation) and communication interference. During Solar Minimum, temperatures drop drastically and solar power generation falls significantly, but radiation is low and comms are clear. This introduces cyclical crises that players must prepare for, balancing between clean solar power and reliable but dirty alternatives.

## 2. Dependencies
- Layer 1 Time/Tick System
- Layer 1 Weather/Atmosphere system (for temperature/radiation effects)
- Layer 1 Power Grid system
- Layer 1 Pop Health/Sickness system

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use layer1::power::{SolarPanel, PowerOutput};
    use layer1::health::{RadiationExposure, Temperature};

    #[test]
    fn test_solar_maximum_increases_power_and_radiation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, apply_solar_cycle_system);

        let cycle_entity = app.world_mut().spawn(SolarCycle { phase: SolarPhase::Maximum }).id();
        let panel_entity = app.world_mut().spawn((SolarPanel { base_output: 100 }, PowerOutput(0))).id();
        let pop_entity = app.world_mut().spawn(RadiationExposure(0)).id();

        app.update();

        let power = app.world().get::<PowerOutput>(panel_entity).unwrap().0;
        assert!(power > 100, "Solar power output should be boosted during Maximum");

        let radiation = app.world().get::<RadiationExposure>(pop_entity).unwrap().0;
        assert!(radiation > 0, "Pops should accumulate radiation during Maximum");
    }

    #[test]
    fn test_solar_minimum_decreases_power_and_temperature() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, apply_solar_cycle_system);

        let cycle_entity = app.world_mut().spawn(SolarCycle { phase: SolarPhase::Minimum }).id();
        let panel_entity = app.world_mut().spawn((SolarPanel { base_output: 100 }, PowerOutput(0))).id();
        let env_entity = app.world_mut().spawn(Temperature(20)).id(); // Base temp 20C

        app.update();

        let power = app.world().get::<PowerOutput>(panel_entity).unwrap().0;
        assert!(power < 100, "Solar power output should be reduced during Minimum");

        let temp = app.world().get::<Temperature>(env_entity).unwrap().0;
        assert!(temp < 20, "Global temperature should drop during Minimum");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct SolarCycle {
    pub phase: SolarPhase,
}

#[derive(PartialEq)]
pub enum SolarPhase {
    Minimum,
    Maximum,
    Normal,
}

pub fn apply_solar_cycle_system(
    cycle_query: Query<&SolarCycle>,
    mut panel_query: Query<(&SolarPanel, &mut PowerOutput)>,
    mut pop_query: Query<&mut RadiationExposure>,
    mut env_query: Query<&mut Temperature>,
) {
    let Ok(cycle) = cycle_query.get_single() else { return };

    match cycle.phase {
        SolarPhase::Maximum => {
            for (panel, mut output) in panel_query.iter_mut() {
                output.0 = panel.base_output * 2; // Boost power
            }
            for mut radiation in pop_query.iter_mut() {
                radiation.0 += 1; // Increase radiation
            }
        }
        SolarPhase::Minimum => {
            for (panel, mut output) in panel_query.iter_mut() {
                output.0 = panel.base_output / 2; // Reduce power
            }
            for mut temp in env_query.iter_mut() {
                temp.0 -= 10; // Drop temperature
            }
        }
        SolarPhase::Normal => {
            for (panel, mut output) in panel_query.iter_mut() {
                output.0 = panel.base_output;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Extract configuration multipliers (e.g. `2.0`, `0.5`, `-10`) into a `SolarCycleConfig` resource.
- Optimize component access by making queries orthogonal where possible.
- Instead of raw addition to radiation/temperature per tick, use delta time to ensure framerate independence.
- Trigger Bevy Events upon cycle transitions (e.g., `SolarCycleChangedEvent`) to notify the UI or audio systems without polling.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Solar Maximum effectively doubles solar power generation and applies a continuous radiation debuff.
- [ ] Solar Minimum halves solar power generation and significantly reduces global map temperature.

## 7. Technical Guidance
- **ECS Design:** The solar cycle should likely be a global `Resource` rather than a component on an entity, though testing with a singleton entity is fine for simplicity. Refactor to use a `Resource` if appropriate for global environmental variables.
- **Timing:** Integrate the cycle phase transition with the existing layer 1 clock (e.g., ticking years or seasons).
- **Feedback:** Ensure that UI warnings are broadcast when entering extreme phases (Maximum/Minimum).

## 8. Questions
*Builder: add questions here if spec is unclear.*
