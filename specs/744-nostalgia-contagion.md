# 744 The Nostalgia Contagion

## 1. Overview
**Layer:** 1
**Fantasy:** A disease of the mind where the past is so alluring it paralyzes the present.
**Mechanic:** A psychological affliction spread through conversation. Infected Pops experience massive Morale boosts but refuse to perform any "new" tasks (building new structures, researching tech). They will only maintain existing buildings or perform basic survival routines. Curing it requires aggressively isolating infected Pops or destroying the "artifacts" (old buildings, statues) they fixate on.

## 2. Dependencies
- `src/layer1/needs.rs` (Morale/Needs)
- `src/layer1/memory.rs` (Memories, Rumor Web integration)
- `src/layer1/social.rs` (Conversation mechanisms)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // A dummy component for the test environment.
    #[derive(Component)]
    struct Pop;

    #[derive(Component)]
    struct NostalgiaContagion;

    #[derive(Component)]
    struct Morale(f32);

    #[derive(Component, PartialEq, Eq)]
    enum TaskPriority {
        Maintenance,
        Survival,
        NewConstruction,
        Research,
    }

    #[derive(Component)]
    struct CurrentTask(TaskPriority);

    fn nostalgia_spread_system(
        mut commands: Commands,
        infected_query: Query<Entity, With<NostalgiaContagion>>,
        mut uninfected_query: Query<(Entity, &mut Morale), Without<NostalgiaContagion>>,
        // Simplification for conversation adjacency
        time: Res<Time>,
    ) {
        // Stub implementation for test validation
        if !infected_query.is_empty() {
            for (entity, mut morale) in uninfected_query.iter_mut() {
                commands.entity(entity).insert(NostalgiaContagion);
                morale.0 += 50.0;
            }
        }
    }

    fn nostalgia_task_restriction_system(
        mut query: Query<(&NostalgiaContagion, &mut CurrentTask)>,
    ) {
        for (_, mut task) in query.iter_mut() {
            if task.0 == TaskPriority::NewConstruction || task.0 == TaskPriority::Research {
                task.0 = TaskPriority::Maintenance;
            }
        }
    }

    #[test]
    fn test_nostalgia_contagion_spreads_and_boosts_morale() {
        let mut app = App::new();
        app.add_systems(Update, nostalgia_spread_system);
        app.init_resource::<Time>();

        let infected_pop = app.world_mut().spawn((Pop, NostalgiaContagion, Morale(50.0))).id();
        let uninfected_pop = app.world_mut().spawn((Pop, Morale(50.0))).id();

        app.update();

        assert!(app.world().entity(uninfected_pop).contains::<NostalgiaContagion>());
        assert_eq!(app.world().entity(uninfected_pop).get::<Morale>().unwrap().0, 100.0);
    }

    #[test]
    fn test_nostalgia_contagion_restricts_tasks() {
        let mut app = App::new();
        app.add_systems(Update, nostalgia_task_restriction_system);

        let infected_builder = app.world_mut().spawn((Pop, NostalgiaContagion, CurrentTask(TaskPriority::NewConstruction))).id();
        let infected_researcher = app.world_mut().spawn((Pop, NostalgiaContagion, CurrentTask(TaskPriority::Research))).id();
        let infected_maintainer = app.world_mut().spawn((Pop, NostalgiaContagion, CurrentTask(TaskPriority::Maintenance))).id();

        app.update();

        assert!(app.world().entity(infected_builder).get::<CurrentTask>().unwrap().0 == TaskPriority::Maintenance);
        assert!(app.world().entity(infected_researcher).get::<CurrentTask>().unwrap().0 == TaskPriority::Maintenance);
        assert!(app.world().entity(infected_maintainer).get::<CurrentTask>().unwrap().0 == TaskPriority::Maintenance);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Stub:
// pub struct NostalgiaContagion;
// Add system logic into `src/layer1/social.rs` that checks if an infected pop interacts with another.
// If true, apply `NostalgiaContagion` and significantly boost `Morale`.
// Add system logic into `src/layer1/utility_ai.rs` that forces `NostalgiaContagion` pops to only select `Maintenance` or `Survival` ActionTypes.
```

## 5. REFACTOR Phase: Quality & Design
- Integrate tightly with the existing `Rumor Web` (`src/layer1/social.rs`) rather than hardcoding bespoke adjacency checks.
- Balance the `Morale` boost to ensure it's not a strictly positive exploit. The restriction to maintenance/survival should be a harsh penalty.
- The cure condition (destroying old artifacts) needs integration with `Chronicle`/`Building` age to identify valid targets.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops with `NostalgiaContagion` cannot perform construction or research tasks.
- [ ] Spreads via social interaction.

## 7. Technical Guidance
- The task restriction can be implemented elegantly in `evaluate_actions_system` by artificially dropping the Utility AI score of restricted task types to 0.0 for infected Pops.
- Curing the contagion can be implemented via an event (`CureNostalgiaEvent`) triggered when a designated "old" structure is dismantled.

## 8. Questions
*Builder: add questions here if spec is unclear.*
