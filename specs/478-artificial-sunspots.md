# Specification 478: Artificial Sunspots

## 1. Overview
A hostile Layer 3 empire uses a megastructure to induce artificial sunspots on your system's star. This plunges your Layer 1 colony into sudden, unpredictable periods of radioactive darkness. Solar power drops to zero, and surface operations become lethal without heavy shielding. This mechanic introduces extreme tension for solar-reliant colonies, forcing them to balance clean, local stellar energy against the need for dirty, local fallback generators to survive orbital manipulation.

## 2. Dependencies
- `042` Energy System (`PowerSource` component, solar panel equivalents).
- `065` Day/Night Cycle (or the Time system governing lighting/temperature).
- Layer 3 interaction systems capable of triggering events in Layer 1.

## 3. RED Phase: Tests First

```rust
// tests/artificial_sunspots_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use scale::layer1::energy::PowerSource;
    use scale::layer1::pop::Health;

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup time, grid, and systems
        world
    }

    #[test]
    fn test_artificial_sunspot_drops_solar_power_to_zero() {
        let mut world = setup_world();

        // Spawn a solar panel that normally generates power
        let solar_panel = world.spawn((
            SolarPanel { base_output: 100.0 },
            PowerSource { output: 100.0, active: true },
        )).id();

        // Trigger the artificial sunspot event
        world.insert_resource(ArtificialSunspotActive(true));

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_sunspot_effects_system);
        schedule.run(&mut world);

        // Assert power generation is 0
        let power_source = world.get::<PowerSource>(solar_panel).unwrap();
        assert_eq!(power_source.output, 0.0);
    }

    #[test]
    fn test_artificial_sunspot_causes_radiation_damage_outdoors() {
        let mut world = setup_world();

        // Spawn a pop outdoors (no shielding)
        let pop = world.spawn((
            PopBundle::default(),
            Health { current: 100.0, max: 100.0 },
            OutdoorExposure, // Indicates the pop is outside
        )).id();

        // Trigger the artificial sunspot event
        world.insert_resource(ArtificialSunspotActive(true));

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_sunspot_radiation_system);
        schedule.run(&mut world);

        // Assert health has decreased due to radiation
        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 100.0);
    }

    #[test]
    fn test_artificial_sunspot_no_damage_indoors() {
        let mut world = setup_world();

        // Spawn a pop indoors (shielded)
        let pop = world.spawn((
            PopBundle::default(),
            Health { current: 100.0, max: 100.0 },
            // No OutdoorExposure
        )).id();

        // Trigger the artificial sunspot event
        world.insert_resource(ArtificialSunspotActive(true));

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_sunspot_radiation_system);
        schedule.run(&mut world);

        // Assert health is unchanged
        let health = world.get::<Health>(pop).unwrap();
        assert_eq!(health.current, 100.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/environment/artificial_sunspots.rs

use bevy_ecs::prelude::*;
use crate::layer1::energy::PowerSource;
use crate::layer1::pop::Health;

#[derive(Resource, Default)]
pub struct ArtificialSunspotActive(pub bool);

#[derive(Component)]
pub struct SolarPanel {
    pub base_output: f32,
}

#[derive(Component)]
pub struct OutdoorExposure;

pub fn apply_sunspot_effects_system(
    sunspot: Res<ArtificialSunspotActive>,
    mut solar_panels: Query<(&SolarPanel, &mut PowerSource)>,
) {
    for (panel, mut power) in solar_panels.iter_mut() {
        if sunspot.0 {
            power.output = 0.0;
        } else {
            power.output = panel.base_output;
        }
    }
}

pub fn apply_sunspot_radiation_system(
    sunspot: Res<ArtificialSunspotActive>,
    mut exposed_pops: Query<&mut Health, With<OutdoorExposure>>,
) {
    if sunspot.0 {
        for mut health in exposed_pops.iter_mut() {
            health.current -= 5.0; // Flat radiation damage per tick for MVP
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Visuals**: Add a screen tint or lighting change when the sunspot is active to visually communicate the danger.
- **Lore Integration**: Ensure Chronicle events are generated when the sunspot begins and ends, attributing it to Layer 3 actors if known.
- **Gradual Onset**: Instead of an instant on/off boolean, transition the `ArtificialSunspotActive` resource to a `f32` intensity value that scales the power loss and radiation damage over time.
- **Shielding Mechanics**: Implement proper shielding mechanics (e.g., `RadiationShield` component) rather than relying solely on `OutdoorExposure`.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Solar power output drops to 0 when the sunspot is active.
- [ ] Unshielded pops take damage when the sunspot is active.

## 7. Technical Guidance
- Integrate into the `Layer1SystemSet::Environment` schedule.
- Be careful with the `Health` component to ensure pops can actually die if radiation goes unchecked.
- The `OutdoorExposure` marker should be managed by the existing roofing/indoor calculation systems.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
