# Spec 549: The Sleep Debt Repo Men

## 1. Overview
Corporate overlords have monetized the biological need for sleep. In a desperate bid for early capital, the colony can sign a contract with a predatory megacorp. They provide "Wakefulness Stims" that eliminate the `Rest` need, allowing a 24/7 workforce. However, the un-slept hours are tracked as "Sleep Debt". When debt reaches a critical threshold, the corp sends specialized non-lethal enforcers ("Repo Men") to forcibly drag Pops into mobile cryo-pods for multi-month comas to "reclaim" the debt.

## 2. Dependencies
- `005` Pop needs (hunger, rest)
- `415` Company Scrip (for corporate contracts)
- Combat/Event system to spawn external entities.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::needs::{Needs, NeedsConfig};
    use crate::layer1::pop::Pop;
    use bevy::prelude::*;

    #[test]
    fn test_wakefulness_stim_prevents_rest_decay_and_accrues_debt() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_sleep_debt_system);

        let pop = app.world_mut().spawn((
            Pop,
            Needs { rest: 100.0, ..Default::default() },
            SleepDebt { hours: 0.0, active_contract: true },
        )).id();

        // Act
        // Simulate time passing (1 hour)
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs_f32(3600.0));
        app.update();

        // Assert
        let debt = app.world().get::<SleepDebt>(pop).unwrap();
        let needs = app.world().get::<Needs>(pop).unwrap();

        // Rest should not decay, but debt should increase
        assert_eq!(needs.rest, 100.0);
        assert!(debt.hours > 0.0);
    }

    #[test]
    fn test_critical_sleep_debt_triggers_repo_event() {
        let mut app = App::new();
        app.add_event::<RepoManArrivalEvent>();
        app.add_systems(Update, check_critical_sleep_debt_system);

        app.world_mut().spawn((
            Pop,
            SleepDebt { hours: 500.0, active_contract: true }, // High debt
        ));

        app.update();

        let repo_events = app.world().resource::<Events<RepoManArrivalEvent>>();
        let mut reader = repo_events.get_reader();
        assert!(reader.read(repo_events).len() > 0, "Critical debt should spawn repo men");
    }

    #[test]
    fn test_repo_men_apply_coma_state() {
        let mut app = App::new();
        app.add_systems(Update, process_repo_men_action_system);

        let pop = app.world_mut().spawn((
            Pop,
            SleepDebt { hours: 500.0, active_contract: true },
        )).id();

        let repo_man = app.world_mut().spawn(RepoMan { target: pop }).id();

        app.update();

        // Target should have ForcedComa component
        assert!(app.world().get::<ForcedComa>(pop).is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use crate::layer1::needs::Needs;

#[derive(Component)]
pub struct SleepDebt {
    pub hours: f32,
    pub active_contract: bool,
}

#[derive(Event)]
pub struct RepoManArrivalEvent {
    pub target_pop: Entity,
}

#[derive(Component)]
pub struct RepoMan {
    pub target: Entity,
}

#[derive(Component)]
pub struct ForcedComa {
    pub duration_remaining: f32,
}

pub fn process_sleep_debt_system(
    time: Res<Time>,
    mut query: Query<(&mut Needs, &mut SleepDebt)>,
) {
    let dt = time.delta_secs() / 3600.0; // Simulated hours
    for (mut needs, mut debt) in query.iter_mut() {
        if debt.active_contract {
            // Prevent rest decay
            needs.rest = 100.0;
            // Accrue debt
            debt.hours += dt;
        }
    }
}

pub fn check_critical_sleep_debt_system(
    query: Query<(Entity, &SleepDebt)>,
    mut arrival_events: EventWriter<RepoManArrivalEvent>,
) {
    for (entity, debt) in query.iter() {
        if debt.hours >= 500.0 { // Critical threshold
            arrival_events.send(RepoManArrivalEvent { target_pop: entity });
        }
    }
}

pub fn process_repo_men_action_system(
    mut commands: Commands,
    repo_query: Query<(Entity, &RepoMan)>,
    pop_query: Query<Entity, With<SleepDebt>>,
) {
    for (repo_entity, repo_man) in repo_query.iter() {
        if let Ok(target_pop) = pop_query.get(repo_man.target) {
            commands.entity(target_pop).insert(ForcedComa { duration_remaining: 2160.0 }); // 3 months coma
            commands.entity(repo_entity).despawn(); // Repo man leaves
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Extract the hardcoded `500.0` debt threshold and `2160.0` coma duration into a configurable `SleepDebtConfig` resource.
- Integrate `ForcedComa` properly with the Utility AI so Pops in a coma don't try to perform actions and can't be assigned jobs.
- The `RepoManArrivalEvent` should hook into the spatial system to spawn the enforcer at the edge of the map, who then pathfinds to the target.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Pops with an active `SleepDebt` contract do not suffer `Rest` need decay.
- [ ] Pops accumulate `SleepDebt` hours over time.
- [ ] A `RepoManArrivalEvent` triggers when `SleepDebt` reaches a specific threshold.
- [ ] Pops targeted by Repo Men receive a `ForcedComa` state that disables them.

## 7. Technical Guidance
- `SleepDebt` should be applied when the colony accepts the Megacorp's "Wakefulness Stims" contract (Layer 3 -> Layer 1 interaction).
- `ForcedComa` needs to bypass standard medical triage. The Pops cannot be woken up early by colony doctors.
- Ensure the `RepoMan` uses standard pathfinding to reach the Pop, allowing for emergent scenarios (e.g., they might get attacked by wild beasts on the way, though they should be heavily armored).

## 8. Questions
*Builder: add questions here if spec is unclear.*
