# Specification 309: The Bureaucratic Strike

## 1. Overview
This feature simulates the malicious compliance of administrative staff ("Clerks", "Managers"). When the morale of Admin Pops drops too low, instead of rioting destructively, they trigger a "Red Tape" event. This strike massively inflates the time required to process building designations, job reassignments, and trade deals, paralyzing the colony's logistics.

## 2. Dependencies
- `Designation` system (`src/layer1/designation.rs`)
- `Job` system / Assignments (`src/layer1/jobs.rs`)
- `Morale` system (`src/layer1/morale.rs`)
- `Grievance` / Unrest system (`src/layer1/grievance.rs`)

## 3. RED Phase: Tests First

```rust
// src/layer1/social/bureaucratic_strike.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::designation::DesignationType;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            update_bureaucratic_strike_status_system,
            process_red_tape_designations_system,
        ));
        app.insert_resource(RedTapeEvent { active: false, severity: 1 });
        app
    }

    #[test]
    fn test_red_tape_increases_designation_cost() {
        let mut app = setup_app();

        let designation = app.world_mut().spawn(RedTapeCost { current_cost: 0.0, max_cost: 100.0, designation_type: DesignationType::Build }).id();

        app.world_mut().resource_mut::<RedTapeEvent>().active = true;
        app.world_mut().resource_mut::<RedTapeEvent>().severity = 10;

        app.update();

        // Cost should be multiplied by severity
        let cost = app.world().get::<RedTapeCost>(designation).unwrap();
        assert_eq!(cost.max_cost, 1000.0);
    }

    #[test]
    fn test_red_tape_prevents_instant_job_reassignment() {
        let mut app = setup_app();

        // This simulates a pop trying to take a job but being delayed by paperwork
        let job = app.world_mut().spawn(PaperworkDelay { ticks_remaining: 0 }).id();

        app.world_mut().resource_mut::<RedTapeEvent>().active = true;

        app.update();

        let delay = app.world().get::<PaperworkDelay>(job).unwrap();
        assert!(delay.ticks_remaining > 0);
    }

    #[test]
    fn test_strike_ends_when_admin_morale_recovers() {
        let mut app = setup_app();

        // Add a happy Admin Pop
        app.world_mut().spawn((
            AdminPop,
            Morale { value: 90.0 }, // High morale
        ));

        app.world_mut().resource_mut::<RedTapeEvent>().active = true;

        app.update();

        // The active state should flip back to false
        let event = app.world().resource::<RedTapeEvent>();
        assert_eq!(event.active, false);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/social/bureaucratic_strike.rs

use bevy::prelude::*;
use crate::layer1::designation::DesignationType;

#[derive(Resource, Clone, Debug)]
pub struct RedTapeEvent {
    pub active: bool,
    pub severity: u32,
}

#[derive(Component, Clone, Debug)]
pub struct AdminPop;

#[derive(Component, Clone, Debug)]
pub struct Morale {
    pub value: f32,
}

#[derive(Component, Clone, Debug)]
pub struct RedTapeCost {
    pub current_cost: f32,
    pub max_cost: f32,
    pub designation_type: DesignationType,
    pub penalized: bool,
}

#[derive(Component, Clone, Debug)]
pub struct PaperworkDelay {
    pub ticks_remaining: u32,
}

pub fn update_bureaucratic_strike_status_system(
    mut red_tape: ResMut<RedTapeEvent>,
    query: Query<&Morale, With<AdminPop>>,
) {
    let mut total_morale = 0.0;
    let mut admin_count = 0;

    for morale in query.iter() {
        total_morale += morale.value;
        admin_count += 1;
    }

    if admin_count > 0 {
        let avg_morale = total_morale / admin_count as f32;
        if avg_morale < 30.0 { // Low morale threshold
            red_tape.active = true;
            red_tape.severity = 10;
        } else if avg_morale > 60.0 {
            red_tape.active = false;
        }
    }
}

pub fn process_red_tape_designations_system(
    red_tape: Res<RedTapeEvent>,
    mut designation_query: Query<&mut RedTapeCost>,
    mut job_query: Query<&mut PaperworkDelay>,
) {
    if red_tape.active {
        for mut cost in designation_query.iter_mut() {
            if !cost.penalized {
                cost.max_cost *= red_tape.severity as f32;
                cost.penalized = true;
            }
        }

        for mut delay in job_query.iter_mut() {
            if delay.ticks_remaining == 0 {
                delay.ticks_remaining = 500 * red_tape.severity; // Add massive delay to jobs
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: The `RedTapeCost` should integrate cleanly with the existing `ConstructionCost` and `Designation` logic. We need to intercept standard assignments.
- **Morale mapping**: Ensure `AdminPop` maps directly to actual `JobType::Clerk` or `JobType::Manager` components used in `src/layer1/jobs.rs`.
- **Chronicle**: Trigger `StrikeStarted` and `StrikeEnded` events to let the player know why their colony has stalled.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for `bureaucratic_strike.rs`.
- [ ] `RedTapeEvent` active state toggles correctly based on average Admin morale.
- [ ] Designation costs multiply correctly when active.
- [ ] Job assignment paperwork delays are applied correctly when active.

## 7. Technical Guidance
- The delay to job assignments (`PaperworkDelay`) means Pops trying to switch roles will stand around in a "Waiting on Forms" state. This requires inserting a new Action into their Utility AI that ranks highest when they have an assigned but delayed job.
- **Optimization**: Bevy ECS makes finding `AdminPop` easy, but caching the `active` status in a Resource prevents running expensive queries every tick for designations.

## 8. Questions
*Builder: add questions here if spec is unclear.*
