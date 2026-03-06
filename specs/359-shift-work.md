# Spec 359: Shift Work

## 1. Overview
The factory never sleeps, but people must. Buildings can be assigned a "Shift" (Day/Night). Pops working a Night shift suffer mood penalties unless they have adapted or possess specific traits (like `Nocturnal`).

## 2. Dependencies
- `004-pop-entity.md` (for Pops)
- `006-building-placement.md` (for Buildings and shift settings)
- `009-job-system.md` (for Job assignments)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::building::Building;
    use crate::layer1::needs::Needs;
    use crate::layer1::job::Job;

    #[test]
    fn test_building_has_default_day_shift() {
        let mut app = App::new();

        let building = app.world_mut().spawn(Building { building_type: BuildingType::Farm }).id();

        // Should default to Day Shift
        assert_eq!(app.world().get::<Shift>(building).unwrap().shift_type, ShiftType::Day);
    }

    #[test]
    fn test_pop_gains_night_shift_debuff() {
        let mut app = App::new();
        app.add_systems(Update, apply_shift_effects_system);

        // Setup Night Shift Building
        let building = app.world_mut().spawn((
            Building { building_type: BuildingType::Mine },
            Shift { shift_type: ShiftType::Night }
        )).id();

        // Setup Pop working the building
        let pop = app.world_mut().spawn((
            Pop,
            Needs { stress: 10.0, ..default() },
            Job { target: building },
        )).id();

        app.update();

        // Stress should increase due to working Night Shift
        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(needs.stress > 10.0);
    }

    #[test]
    fn test_nocturnal_pop_ignores_night_shift_debuff() {
        let mut app = App::new();
        app.add_systems(Update, apply_shift_effects_system);

        let building = app.world_mut().spawn((
            Building { building_type: BuildingType::Mine },
            Shift { shift_type: ShiftType::Night }
        )).id();

        let pop = app.world_mut().spawn((
            Pop,
            Needs { stress: 10.0, ..default() },
            Job { target: building },
            Trait::Nocturnal,
        )).id();

        app.update();

        // Stress should NOT increase because pop is Nocturnal
        let needs = app.world().get::<Needs>(pop).unwrap();
        assert_eq!(needs.stress, 10.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::building::Building;
use crate::layer1::needs::Needs;
use crate::layer1::job::Job;
use crate::layer1::traits::Trait;

#[derive(Component)]
pub struct Pop;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ShiftType {
    Day,
    Night,
}

#[derive(Component)]
pub struct Shift {
    pub shift_type: ShiftType,
}

// Ensure new buildings get a Day shift by default if not specified
pub fn initialize_building_shifts(
    mut commands: Commands,
    query: Query<Entity, (With<Building>, Without<Shift>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(Shift { shift_type: ShiftType::Day });
    }
}

pub fn apply_shift_effects_system(
    mut pop_query: Query<(&mut Needs, &Job, Option<&Trait>), With<Pop>>,
    building_query: Query<&Shift, With<Building>>,
) {
    for (mut needs, job, pop_trait) in pop_query.iter_mut() {
        if let Ok(shift) = building_query.get(job.target) {
            if shift.shift_type == ShiftType::Night {
                if pop_trait != Some(&Trait::Nocturnal) {
                    needs.stress += 0.5; // Apply flat stress penalty per tick
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Time of Day Integration:** Ensure Shift only applies its debuff during actual working hours. `Night` shift workers sleeping during the day shouldn't be penalized for sleeping.
- **Gradual Adaptation:** Instead of instantly being Nocturnal, Pops should build up an `AdaptationLevel` tracker. Working 30 Night shifts in a row slowly makes them `Nocturnal`.
- **Global Clock:** The Shift logic needs to tie into `SimulationTime` (Day/Night cycle) to accurately reflect when the Pop is actively on shift vs off shift.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Buildings without a `Shift` component default to `ShiftType::Day`.
- [ ] A Pop with a `Job` targeting a `Night` shift building gains `Stress` continuously.
- [ ] A Pop with `Trait::Nocturnal` does NOT gain `Stress` from `Night` shift.

## 7. Technical Guidance
- Integrate with `Job` target checking.
- Place `initialize_building_shifts` in a `PostUpdate` or initialization step so new spawned buildings are covered.
- Place `apply_shift_effects_system` in `Layer1SystemSet::Observation` or `Execution`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
