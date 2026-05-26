# 882 The Architect's Guilt

**1. Overview**
Pops who build structures where other pops died accumulate "Guilt." High guilt makes them refuse to repair the building or approach it, eventually trying to sabotage it. This forces the player to deal with the psychological toll of their architectural ambition and the sacrifices made for it.

**2. Dependencies**
- `src/layer1/pop.rs` (Pop component, traits, traits like `Guilt`)
- `src/layer1/buildings.rs` (Building structures, repair logic)
- `src/layer1/memory.rs` (Pop memory system, tracking deaths on specific coordinates/buildings)
- `src/layer1/utility_ai.rs` (Utility AI to handle job scoring, e.g., refusing to repair or approaching a building)

**3. RED Phase: Tests First**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_architects_guilt_accumulation() {
        // Arrange: Setup a world with a building where a pop died during construction
        let mut app = App::new();
        // Setup mock components and systems

        let building_entity = app.world_mut().spawn(Building::new()).id();
        let builder_entity = app.world_mut().spawn((Pop::new(), Builder::new())).id();

        // Simulate a death at the building's location during construction by this builder
        app.world_mut().send_event(PopDiedEvent { location: building_entity, during_construction_by: builder_entity });
        app.update();

        // Act: The builder should now have accumulated "Guilt"
        let builder = app.world().get::<Pop>(builder_entity).unwrap();

        // Assert: Guilt trait is present or Guilt component is attached
        assert!(app.world().get::<ArchitectsGuilt>(builder_entity).is_some());
    }

    #[test]
    fn test_architects_guilt_refuse_repair() {
        // Arrange
        let mut app = App::new();
        let building_entity = app.world_mut().spawn((Building::new(), Damaged::new())).id();
        let builder_entity = app.world_mut().spawn((Pop::new(), Builder::new(), ArchitectsGuilt { associated_building: building_entity })).id();

        // Act: Run the utility AI scoring for repair tasks
        app.update();

        // Assert: The score to repair *this specific* building should be exactly 0.0 or heavily penalized
        let repair_score = get_repair_score(&app, builder_entity, building_entity);
        assert_eq!(repair_score, 0.0);
    }

    #[test]
    fn test_architects_guilt_sabotage() {
        // Arrange
        let mut app = App::new();
        let building_entity = app.world_mut().spawn((Building::new(), Health::new(100.0))).id();
        let builder_entity = app.world_mut().spawn((Pop::new(), Builder::new(), ArchitectsGuilt { associated_building: building_entity, severity: 100.0 })).id();

        // Act: Run the sabotage evaluation system (high guilt triggers sabotage)
        app.update();

        // Assert: The building health should be decreased due to sabotage
        let building_health = app.world().get::<Health>(building_entity).unwrap();
        assert!(building_health.current < 100.0);
    }
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct ArchitectsGuilt {
    pub associated_building: Entity,
    pub severity: f32,
}

pub fn accumulate_guilt_system(
    mut commands: Commands,
    mut events: EventReader<PopDiedEvent>,
) {
    for event in events.read() {
        if let Some(builder_entity) = event.during_construction_by {
            commands.entity(builder_entity).insert(ArchitectsGuilt {
                associated_building: event.location,
                severity: 10.0,
            });
        }
    }
}

pub fn calculate_repair_score(
    builder_entity: Entity,
    building_entity: Entity,
    guilt_query: &Query<&ArchitectsGuilt>,
) -> f32 {
    if let Ok(guilt) = guilt_query.get(builder_entity) {
        if guilt.associated_building == building_entity {
            return 0.0; // Refuse to repair
        }
    }
    1.0 // Normal score
}

pub fn evaluate_sabotage_system(
    mut guilt_query: Query<(&ArchitectsGuilt, Entity)>,
    mut health_query: Query<&mut Health>,
) {
    for (guilt, _) in guilt_query.iter_mut() {
        if guilt.severity >= 100.0 {
            if let Ok(mut health) = health_query.get_mut(guilt.associated_building) {
                health.current -= 10.0; // Sabotage
            }
        }
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- Consider integrating `ArchitectsGuilt` into a broader `Trauma` or `Memory` component rather than a standalone component, depending on how `src/layer1/memory.rs` is structured.
- The `PopDiedEvent` might need to be expanded to carry `during_construction_by` information if it doesn't already.
- Refactor the hardcoded `severity` thresholds into tunable constants or a game configuration resource.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Pops correctly receive `ArchitectsGuilt` when a pop dies at a building they are constructing.
- [ ] Pops with `ArchitectsGuilt` correctly refuse to repair the associated building.

**7. Technical Guidance**
- Review `src/layer1/utility_ai.rs` to see how action scores are calculated and injected. You will need to add a modifier for the `Repair` action.
- Ensure the `evaluate_sabotage_system` is hooked into the correct schedule (e.g., a slow-ticking simulation schedule, not every frame).

**8. Questions**
*Builder: add questions here if spec is unclear.*

*Builder questions:*
1. The RED phase uses `Builder::new()` and `Pop::new()`, but these components usually require `Default` or specific initialization in our current architecture. Is there a specific implementation expected for `Builder`?
   - *Architect:* Treat `Builder` as a standard component (e.g., `struct Builder;` or using `Default`). The `ArchitectsGuilt` component should be attached to the entity that performed the construction job when the death occurred.
