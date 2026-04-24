# The Consultant (Spec 1149)

## 1. Overview
This specification details "The Consultant" feature for SCALE. A corporate NPC arrives who automatically alters job priorities to maximize efficiency ("Maximize Profit") at the severe expense of health and safety, leading to rapid worker burnout and stress.

The goal is to implement a system that monitors the arrival of this NPC and applies extreme efficiency buffs, while also driving up stress, burnout, and risk of critical failure, creating a tension between short-term gains and long-term disaster.

## 2. Dependencies
- Base Population/Job system.
- Stress and Morale system.
- Work Efficiency buffs/modifiers.
- The `Lore` / `Chronicle` systems for generating events.

## 3. RED Phase: Tests First

```rust
use bevy::prelude::*;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consultant_spawn_triggers_profit_override() {
        let mut app = App::new();
        // Setup initial state: Normal priorities
        app.insert_resource(JobPriorities { prioritize_safety: true, ..default() });

        // Spawn Consultant
        app.world.spawn((ConsultantMarker, Name::new("The Fixer")));

        // Run the system
        app.add_systems(Update, apply_consultant_override);
        app.update();

        // Assert: Priorities changed to Maximize Profit
        let priorities = app.world.resource::<JobPriorities>();
        assert_eq!(priorities.prioritize_safety, false);
        assert_eq!(priorities.maximize_profit, true);
    }

    #[test]
    fn test_consultant_buffs_efficiency_but_increases_stress() {
        let mut app = App::new();
        let worker_entity = app.world.spawn((
            Worker,
            Efficiency(1.0),
            Stress(0.0),
        )).id();

        // Spawn Consultant in the same sector/global
        app.world.spawn((ConsultantMarker, BuffRadius(50.0)));

        app.add_systems(Update, consultant_worker_impact);
        app.update();

        // Assert: Efficiency increased massively, but stress went up
        let efficiency = app.world.get::<Efficiency>(worker_entity).unwrap();
        let stress = app.world.get::<Stress>(worker_entity).unwrap();

        assert!(efficiency.0 > 1.5, "Efficiency should receive a massive buff");
        assert!(stress.0 > 0.5, "Stress should increase significantly");
    }

    #[test]
    fn test_consultant_causes_critical_reactor_failure() {
        let mut app = App::new();
        let reactor_entity = app.world.spawn((
            Reactor,
            MaintenanceLevel(0.0), // Low maintenance
            CriticalCondition(false),
        )).id();

        // With consultant present, low maintenance leads to fast failure
        app.world.spawn(ConsultantMarker);

        app.add_systems(Update, consultant_hazard_escalation);
        app.update();

        let condition = app.world.get::<CriticalCondition>(reactor_entity).unwrap();
        assert!(condition.0, "Reactor should reach critical condition under Consultant's watch");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct ConsultantMarker;

#[derive(Resource, Default)]
pub struct JobPriorities {
    pub prioritize_safety: bool,
    pub maximize_profit: bool,
}

#[derive(Component)]
pub struct Worker;

#[derive(Component)]
pub struct Efficiency(pub f32);

#[derive(Component)]
pub struct Stress(pub f32);

#[derive(Component)]
pub struct BuffRadius(pub f32);

#[derive(Component)]
pub struct Reactor;

#[derive(Component)]
pub struct MaintenanceLevel(pub f32);

#[derive(Component)]
pub struct CriticalCondition(pub bool);


pub fn apply_consultant_override(
    consultants: Query<&ConsultantMarker>,
    mut priorities: ResMut<JobPriorities>,
) {
    if !consultants.is_empty() {
        priorities.prioritize_safety = false;
        priorities.maximize_profit = true;
    }
}

pub fn consultant_worker_impact(
    consultants: Query<(), With<ConsultantMarker>>,
    mut workers: Query<(&mut Efficiency, &mut Stress), With<Worker>>,
) {
    if !consultants.is_empty() {
        for (mut efficiency, mut stress) in workers.iter_mut() {
            efficiency.0 = 2.0; // Massive buff
            stress.0 += 1.0; // High stress
        }
    }
}

pub fn consultant_hazard_escalation(
    consultants: Query<(), With<ConsultantMarker>>,
    mut hazards: Query<(&MaintenanceLevel, &mut CriticalCondition), With<Reactor>>,
) {
    if !consultants.is_empty() {
        for (maintenance, mut condition) in hazards.iter_mut() {
            if maintenance.0 < 0.5 {
                condition.0 = true;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: The consultant should broadcast events on arrival and departure so the `Chronicle` system can record their disastrous tenure using the Lexicon term "The Consultant".
- **Refactoring Opportunities**: The current implementation checks `if !consultants.is_empty()` in every system. This should be refactored into a `RunCondition` (e.g., `run_if(any_with_component::<ConsultantMarker>)`) to avoid unnecessary iteration when the consultant isn't around.
- **Stress Scaling**: Stress shouldn't just instantly spike; it should compound over time using `Time`.
- **Targeting**: The consultant might buff only the sector they are currently residing in, rather than a global modifier, to create spatial dynamics (e.g., move them to the mines, stress spikes there).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] The Consultant increases both productivity and stress.

## 7. Technical Guidance
- **Gotchas**: Watch out for runaway stress causing immediate colony collapse. Give players a way to handle the Consultant (bribes, "accidents").
- **Integration Points**: Tie the critical failure logic into the existing `Hazard` and `Explosion` systems. Add templates for "Consultant Arrives", "Consultant Blames Workers", and "Consultant Departs".

## 8. Questions
*Builder: add questions here if spec is unclear.*
