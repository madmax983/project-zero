# 1286: Ruin Integration

## 1. Overview
**Layer:** 1

**Fantasy:** Living inside the massive, incomprehensible bones of a previous civilization.

**Mechanic:** Pops can choose to settle inside pre-existing "Ancient Ruins" instead of building new housing. This grants massive defense and temperature regulation, but the ruins randomly power up ancient, unknown machinery that causes unique psychological stress.

## 2. Dependencies
- Base Layer 1 Population & Housing System
- Environmental Need / Temperature System
- Psychological Stress System
- Map/Grid System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // RED Phase Test Setup
    fn setup_app() -> App {
        let mut app = App::new();
        // Add systems
        app.add_systems(Update, (
            apply_ruin_environmental_buffs_system,
            trigger_ruin_machinery_system,
            apply_ruin_psychological_stress_system,
        ));
        app.init_resource::<SimulationTick>();
        app
    }

    #[test]
    fn test_ruin_housing_provides_temperature_buff() {
        let mut app = setup_app();

        let ruin = app.world_mut().spawn((
            AncientRuin { is_active: false },
            TemperatureRegulation { bonus: 20.0 },
        )).id();

        let pop = app.world_mut().spawn((
            Pop,
            HousedIn(ruin),
            Temperature { current: 10.0 },
        )).id();

        app.update();

        let temp = app.world().get::<Temperature>(pop).unwrap();
        // Expecting the temp to be boosted by the ruin's regulation bonus
        assert_eq!(temp.current, 30.0, "Pop temperature should be buffed by ruin housing");
    }

    #[test]
    fn test_ruin_machinery_activates_randomly() {
        let mut app = setup_app();
        app.world_mut().resource_mut::<SimulationTick>().0 = 100; // Force trigger condition for test

        let ruin = app.world_mut().spawn((
            AncientRuin { is_active: false },
            MachineryTrigger { threshold: 50 },
        )).id();

        app.update();

        let ruin_data = app.world().get::<AncientRuin>(ruin).unwrap();
        assert!(ruin_data.is_active, "Ruin machinery should activate when threshold is met");
    }

    #[test]
    fn test_active_ruin_causes_psychological_stress() {
        let mut app = setup_app();

        let active_ruin = app.world_mut().spawn((
            AncientRuin { is_active: true },
        )).id();

        let pop = app.world_mut().spawn((
            Pop,
            HousedIn(active_ruin),
            Stress { level: 0.0 },
        )).id();

        app.update();

        let stress = app.world().get::<Stress>(pop).unwrap();
        assert!(stress.level > 0.0, "Pop should gain stress when housed in an active ruin");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct SimulationTick(pub u64);

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct HousedIn(pub Entity);

#[derive(Component)]
pub struct Temperature {
    pub current: f32,
}

#[derive(Component)]
pub struct Stress {
    pub level: f32,
}

#[derive(Component)]
pub struct AncientRuin {
    pub is_active: bool,
}

#[derive(Component)]
pub struct TemperatureRegulation {
    pub bonus: f32,
}

#[derive(Component)]
pub struct MachineryTrigger {
    pub threshold: u64,
}

pub fn apply_ruin_environmental_buffs_system(
    ruins: Query<&TemperatureRegulation, With<AncientRuin>>,
    mut pops: Query<(&HousedIn, &mut Temperature), With<Pop>>,
) {
    for (housing, mut temp) in pops.iter_mut() {
        if let Ok(regulator) = ruins.get(housing.0) {
             // Minimal implementation: just add the bonus once per frame.
             // In a real system, this would likely be a calculated modifier, not continuous addition.
             // For test passing:
             temp.current += regulator.bonus;
        }
    }
}

pub fn trigger_ruin_machinery_system(
    tick: Res<SimulationTick>,
    mut ruins: Query<(&mut AncientRuin, &MachineryTrigger)>,
) {
    for (mut ruin, trigger) in ruins.iter_mut() {
        if tick.0 >= trigger.threshold && !ruin.is_active {
            ruin.is_active = true;
        }
    }
}

pub fn apply_ruin_psychological_stress_system(
    ruins: Query<&AncientRuin>,
    mut pops: Query<(&HousedIn, &mut Stress), With<Pop>>,
) {
    for (housing, mut stress) in pops.iter_mut() {
        if let Ok(ruin) = ruins.get(housing.0) {
             if ruin.is_active {
                 stress.level += 5.0; // Fixed stress penalty
             }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Modifier System**: The `apply_ruin_environmental_buffs_system` currently adds the bonus directly, which would infinitely stack if run over multiple ticks. This should be refactored to use a stat modifier system if the project has one, or ensure it only applies the difference if calculating an absolute temperature. For the tests, it passes, but the architecture needs care.
- **RNG Activation**: `trigger_ruin_machinery_system` uses a hardcoded tick threshold. Introduce an RNG component to make activation truly random as per the spec design.
- **Deactivation**: Add logic for ruins to eventually power back down (or require player intervention to silence them).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- **Housing System Compatibility**: Ensure `HousedIn` aligns with the project's standard way of tracking where pops live (e.g., standard `Residence` component pointing to an entity on the grid).
- **Events**: Consider emitting an `AncientMachineryActivatedEvent` to trigger sound effects, visual FX, and UI notifications.

## 8. Questions
*Builder: add questions here if spec is unclear.*
