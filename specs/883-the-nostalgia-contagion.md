# 883 The Nostalgia Contagion

**1. Overview**
Trade ships from the Core Worlds occasionally carry "Nostalgia Artifacts" (old holovids, physical books). Pops who interact with them get a massive morale boost but a permanent "Yearning" debuff, causing them to spend work hours staring at the sky. A psychological affliction spread through conversation. Infected Pops experience massive Morale boosts but refuse to perform any "new" tasks (building new structures, researching tech). They will only maintain existing buildings or perform basic survival routines.

**2. Dependencies**
- `src/layer1/pop.rs` (Pop component, traits like `Yearning`)
- `src/layer1/utility_ai.rs` (Utility AI for scoring tasks based on the `Yearning` trait)
- `src/layer1/social.rs` (or similar, for spreading traits/afflictions through conversation)
- `src/layer1/items.rs` (Nostalgia Artifacts logic)

**3. RED Phase: Tests First**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_nostalgia_contagion_infection() {
        // Arrange
        let mut app = App::new();
        // Setup mock components and systems
        let artifact_entity = app.world_mut().spawn(NostalgiaArtifact::new()).id();
        let pop_entity = app.world_mut().spawn((Pop::new(), Position::new(0, 0))).id();
        app.world_mut().entity_mut(pop_entity).insert(Inventory::with_item(artifact_entity));

        // Act: Run the interaction system
        app.update();

        // Assert: Pop should have the Yearning trait/component
        assert!(app.world().get::<Yearning>(pop_entity).is_some());
    }

    #[test]
    fn test_nostalgia_contagion_spread() {
        // Arrange
        let mut app = App::new();
        let infected_pop = app.world_mut().spawn((Pop::new(), Yearning::new(), Position::new(0, 0))).id();
        let clean_pop = app.world_mut().spawn((Pop::new(), Position::new(0, 1))).id(); // Adjacent

        // Act: Run social interaction system
        app.update();

        // Assert: Clean pop should now be infected
        assert!(app.world().get::<Yearning>(clean_pop).is_some());
    }

    #[test]
    fn test_nostalgia_contagion_task_refusal() {
        // Arrange
        let mut app = App::new();
        let pop_entity = app.world_mut().spawn((Pop::new(), Yearning::new())).id();
        let new_building_task = app.world_mut().spawn(BuildTask::new()).id();
        let maintenance_task = app.world_mut().spawn(MaintenanceTask::new()).id();

        // Act: Run utility AI scoring
        app.update();

        // Assert: Build task score is 0.0, Maintenance task score is > 0.0
        let build_score = get_task_score(&app, pop_entity, new_building_task);
        let maint_score = get_task_score(&app, pop_entity, maintenance_task);

        assert_eq!(build_score, 0.0);
        assert!(maint_score > 0.0);
    }

    #[test]
    fn test_nostalgia_contagion_morale_boost() {
        // Arrange
        let mut app = App::new();
        let pop_entity = app.world_mut().spawn((Pop::new(), Morale::new(50.0), Yearning::new())).id();

        // Act: Run morale update system
        app.update();

        // Assert: Morale should be significantly boosted
        let morale = app.world().get::<Morale>(pop_entity).unwrap();
        assert!(morale.current > 50.0);
    }
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct NostalgiaArtifact;

#[derive(Component)]
pub struct Yearning;

pub fn interact_with_artifact_system(
    mut commands: Commands,
    query: Query<(Entity, &Inventory)>,
    artifact_query: Query<&NostalgiaArtifact>,
) {
    for (entity, inventory) in query.iter() {
        for item in &inventory.items {
            if artifact_query.get(*item).is_ok() {
                commands.entity(entity).insert(Yearning);
            }
        }
    }
}

pub fn spread_contagion_system(
    mut commands: Commands,
    infected_query: Query<(Entity, &Position, &Yearning)>,
    mut clean_query: Query<(Entity, &Position), Without<Yearning>>,
) {
    for (_, inf_pos, _) in infected_query.iter() {
        for (clean_ent, clean_pos) in clean_query.iter_mut() {
            if inf_pos.distance(clean_pos) <= 1.0 { // Adjacent
                commands.entity(clean_ent).insert(Yearning);
            }
        }
    }
}

pub fn adjust_task_scores_for_yearning(
    pop_entity: Entity,
    task_type: TaskType,
    yearning_query: &Query<&Yearning>,
) -> f32 {
    if yearning_query.get(pop_entity).is_ok() {
        match task_type {
            TaskType::BuildNew | TaskType::Research => return 0.0,
            _ => return 1.0,
        }
    }
    1.0
}

pub fn apply_yearning_morale_boost(
    mut query: Query<&mut Morale, With<Yearning>>,
) {
    for mut morale in query.iter_mut() {
        morale.current = (morale.current + 20.0).min(100.0);
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- The social spreading mechanic should probably hook into an existing `Gossip` or `Conversation` event system rather than a raw distance check every frame to be more performant and thematic.
- Ensure that `Yearning` is properly cleared if the player destroys the source artifacts or institutes isolation policies.
- Task scoring should be integrated into the existing `evaluate_actions_system` weights rather than a standalone function.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Pops with Nostalgia Artifacts in inventory gain `Yearning`.
- [ ] `Yearning` spreads to nearby pops.
- [ ] Pops with `Yearning` refuse new building/research tasks.
- [ ] Pops with `Yearning` have a massive morale boost.

**7. Technical Guidance**
- Review `src/layer1/utility_ai.rs` to correctly modify the utility scoring logic.
- Consider adding an `is_new_task()` method to tasks to easily filter them in the utility AI when checking the `Yearning` trait.

**8. Questions**
*Builder: add questions here if spec is unclear.*
