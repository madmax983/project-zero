# Specification: Technological Rituals (Feature 810)

## 1. Overview
The **Technological Rituals** feature represents the degradation of knowledge over time. As machines age or are jury-rigged, they develop "Quirks" (e.g., a "Hard Start"). To operate them, Pops must perform nonsensical "Rituals" (kicking the machine, praying to it, toggling switches in a specific sequence). Failure to perform these rituals decreases the machine's efficiency or triggers breakdowns. This presents a tension between investing in expensive, proper repairs versus relying on cheap but time-consuming ritual maintenance performed by specialized Pops.

## 2. Dependencies
- `Layer 1 System`
- `Machinery & Structure Aging Systems`
- `Pop Task System`
- `Job/Role System` (Specialist requirement)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_aging_machine_develops_quirks() {
        let mut app = App::new();
        app.add_systems(Update, machine_aging_system);

        // Arrange
        let entity = app.world_mut().spawn((
            Machinery { age: 100 },
            QuirkCapacity { current_quirks: 0, threshold_age: 50 }
        )).id();

        // Act
        app.update();

        // Assert
        let quirks = app.world().get::<QuirkCapacity>(entity).unwrap();
        assert!(quirks.current_quirks > 0, "Machine past its threshold age should develop quirks");
    }

    #[test]
    fn test_ritual_maintenance_restores_efficiency() {
        let mut app = App::new();
        app.add_systems(Update, perform_ritual_system);

        // Arrange
        let machine = app.world_mut().spawn((
            Machinery { age: 100 },
            QuirkCapacity { current_quirks: 1, threshold_age: 50 },
            OperatingEfficiency { value: 0.5 }
        )).id();

        let pop = app.world_mut().spawn((
            Pop,
            RitualSpecialist,
            TargetEntity(machine)
        )).id();

        // Act
        app.update();

        // Assert
        let efficiency = app.world().get::<OperatingEfficiency>(machine).unwrap();
        assert!(efficiency.value > 0.5, "Performing a ritual should restore machine efficiency");
    }

    #[test]
    fn test_missing_ritual_causes_breakdown() {
        let mut app = App::new();
        app.add_systems(Update, quirk_breakdown_system);

        // Arrange
        let machine = app.world_mut().spawn((
            Machinery { age: 100 },
            QuirkCapacity { current_quirks: 1, threshold_age: 50 },
            RitualTimer { ticks_since_last_ritual: 100, max_ticks: 50 },
            OperationalState::Running
        )).id();

        // Act
        app.update();

        // Assert
        let state = app.world().get::<OperationalState>(machine).unwrap();
        assert_eq!(*state, OperationalState::Broken, "Exceeding the ritual timer should cause a breakdown");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Machinery {
    pub age: u32,
}

#[derive(Component)]
pub struct QuirkCapacity {
    pub current_quirks: u32,
    pub threshold_age: u32,
}

#[derive(Component)]
pub struct OperatingEfficiency {
    pub value: f32,
}

#[derive(Component)]
pub struct RitualTimer {
    pub ticks_since_last_ritual: u32,
    pub max_ticks: u32,
}

#[derive(Component, PartialEq, Debug)]
pub enum OperationalState {
    Running,
    Broken,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct RitualSpecialist;

#[derive(Component)]
pub struct TargetEntity(pub Entity);

pub fn machine_aging_system(mut query: Query<(&Machinery, &mut QuirkCapacity)>) {
    for (machinery, mut capacity) in query.iter_mut() {
        if machinery.age > capacity.threshold_age && capacity.current_quirks == 0 {
            capacity.current_quirks = 1;
        }
    }
}

pub fn perform_ritual_system(
    mut pop_query: Query<(&RitualSpecialist, &TargetEntity)>,
    mut machine_query: Query<(&mut OperatingEfficiency, &mut RitualTimer)>
) {
    for (_, target) in pop_query.iter_mut() {
        if let Ok((mut efficiency, mut timer)) = machine_query.get_mut(target.0) {
            efficiency.value = 1.0;
            timer.ticks_since_last_ritual = 0;
        }
    }
}

pub fn quirk_breakdown_system(mut query: Query<(&mut OperationalState, &RitualTimer)>) {
    for (mut state, timer) in query.iter_mut() {
        if timer.ticks_since_last_ritual > timer.max_ticks {
            *state = OperationalState::Broken;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Expand the `Quirk` definition into a struct or enum rather than a raw counter so that different quirks require different specific rituals (e.g., `Quirk::HardStart` requires `Ritual::Kick`).
- Introduce a progressive aging model where the probability of a quirk forming increases non-linearly with `Machinery.age`.
- Require the Pop to be spatially adjacent to the machine to perform the ritual, adding a `Transform` and `Distance` check in the `perform_ritual_system`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Machines properly generate quirks after exceeding threshold ages.
- [ ] Efficiency is restored only by a ritual specialist targeting the machine.

## 7. Technical Guidance
- The `perform_ritual_system` should eventually be integrated into the overarching AI task pipeline to ensure Pops navigate to the machine instead of instantly triggering it from across the map.
- Consider adding an `Event` system for `MachineBreakdownEvent` to alert the UI/Player when a ritual is missed.

## 8. Questions
*Builder: add questions here if spec is unclear.*
