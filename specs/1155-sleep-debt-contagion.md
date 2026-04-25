# 1155 Sleep Debt Contagion

## 1. Overview
When you force Pops to work extreme overtime, they accumulate "Sleep Debt." Pops with high Sleep Debt become clumsy (causing accidents) and irritable. Crucially, when an exhausted Pop interacts with a rested Pop, their erratic behavior and stress "infect" the rested Pop, slightly lowering their rest bar and increasing their stress.

This creates tension: Do you crunch your workers to meet a critical Layer 2 deadline, risking a cascading psychological collapse that could ruin the colony's productivity for months?

## 2. Dependencies
- Base `Pop` component and standard `Layer 1` simulation framework.
- Existing `Stress` and `Rest` (or equivalent mood/needs) components.
- A basic interaction or proximity system between pops (e.g. `SocialInteractionEvent` or spatial query).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_sleep_debt_accumulation_during_overtime() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_systems(Update, process_overtime_sleep_debt);

        let entity = app.world_mut().spawn((
            Pop,
            OvertimeWorker,
            SleepDebt(0.0),
        )).id();

        app.update();

        // The pop is working overtime, so sleep debt should increase.
        let debt = app.world().entity(entity).get::<SleepDebt>().unwrap();
        assert!(debt.0 > 0.0);
    }

    #[test]
    fn test_clumsy_accident_trigger_above_threshold() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_event::<IndustrialAccidentEvent>()
           .add_systems(Update, trigger_clumsy_accidents);

        // A pop with high sleep debt
        app.world_mut().spawn((
            Pop,
            SleepDebt(80.0), // Above standard threshold of 75.0
            JobStatus::Working,
        ));

        app.update();

        let accident_events = app.world().resource::<Events<IndustrialAccidentEvent>>();
        let mut reader = accident_events.get_reader();
        assert!(reader.read(accident_events).count() > 0, "High sleep debt should cause an accident while working");
    }

    #[test]
    fn test_sleep_debt_contagion_on_interaction() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_event::<SocialInteractionEvent>()
           .add_systems(Update, process_sleep_debt_contagion);

        let exhausted_pop = app.world_mut().spawn((Pop, SleepDebt(90.0))).id();
        let rested_pop = app.world_mut().spawn((Pop, Rest(100.0), Stress(0.0))).id();

        app.world_mut().resource_mut::<Events<SocialInteractionEvent>>().send(SocialInteractionEvent {
            initiator: exhausted_pop,
            target: rested_pop,
        });

        app.update();

        let target_rest = app.world().entity(rested_pop).get::<Rest>().unwrap();
        let target_stress = app.world().entity(rested_pop).get::<Stress>().unwrap();

        assert!(target_rest.0 < 100.0, "Rested pop should lose rest from interaction");
        assert!(target_stress.0 > 0.0, "Rested pop should gain stress from interaction");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct OvertimeWorker;

#[derive(Component)]
pub struct SleepDebt(pub f32);

#[derive(Component)]
pub struct Rest(pub f32);

#[derive(Component)]
pub struct Stress(pub f32);

#[derive(Component)]
pub enum JobStatus {
    Working,
    Idle,
}

#[derive(Event)]
pub struct IndustrialAccidentEvent;

#[derive(Event)]
pub struct SocialInteractionEvent {
    pub initiator: Entity,
    pub target: Entity,
}

pub fn process_overtime_sleep_debt(
    mut query: Query<&mut SleepDebt, With<OvertimeWorker>>
) {
    for mut debt in query.iter_mut() {
        debt.0 += 10.0;
    }
}

pub fn trigger_clumsy_accidents(
    query: Query<&SleepDebt, (With<Pop>, With<JobStatus>)>,
    mut accident_writer: EventWriter<IndustrialAccidentEvent>,
) {
    for debt in query.iter() {
        if debt.0 > 75.0 {
            accident_writer.send(IndustrialAccidentEvent);
        }
    }
}

pub fn process_sleep_debt_contagion(
    mut interaction_events: EventReader<SocialInteractionEvent>,
    query_exhausted: Query<&SleepDebt>,
    mut query_rested: Query<(&mut Rest, &mut Stress)>,
) {
    for event in interaction_events.read() {
        if let Ok(debt) = query_exhausted.get(event.initiator) {
            if debt.0 > 50.0 {
                if let Ok((mut rest, mut stress)) = query_rested.get_mut(event.target) {
                    rest.0 -= 5.0;
                    stress.0 += 10.0;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: The accident trigger system unconditionally spawns events every tick if debt is high. It should probably use a probability check based on debt level and only trigger occasionally.
- **Performance**: In a massive colony, iterating over all social interactions could be costly. Ensure we batch interactions or only process contagion periodically.
- **API Improvements**: `IndustrialAccidentEvent` should likely contain the Entity that caused it and the location, to propagate damage or colony notifications.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Overtime linearly increases `SleepDebt`.
- [ ] Interactions between exhausted and rested pops correctly alter `Rest` and `Stress`.

## 7. Technical Guidance
- Integrate with the existing `JobSystem` so that toggling "Overtime" on a facility automatically adds the `OvertimeWorker` component to its active workers.
- The `IndustrialAccidentEvent` should tie into the existing event log so the player sees the consequence of their crunch policies.

## 8. Questions
*Builder: add questions here if spec is unclear.*
