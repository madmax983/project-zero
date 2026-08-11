# 1360 - The Automation Paradox

**1. Overview**
As more Layer 1 jobs are automated, Pops transition to a "Leisure" state. Prolonged leisure causes an "Ennui" debuff to stack, eventually leading to sabotage as Pops try to create emergencies to feel useful again.

**2. Dependencies**
- `Pop` components and Job/Leisure states.
- Sabotage event system.

**3. RED Phase: Tests First**
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    #[derive(Component)]
    struct PopState { ennui: f32, is_leisure: bool }

    #[derive(Event)]
    struct SabotageEvent(Entity);

    fn process_ennui_system(
        mut commands: Commands,
        mut query: Query<(Entity, &mut PopState)>,
        mut sabotage_events: EventWriter<SabotageEvent>,
    ) {
        for (entity, mut pop) in query.iter_mut() {
            if pop.is_leisure {
                pop.ennui += 1.0;
                if pop.ennui > 10.0 {
                    sabotage_events.send(SabotageEvent(entity));
                    pop.ennui = 0.0;
                }
            }
        }
    }

    #[test]
    fn test_prolonged_leisure_triggers_sabotage() {
        let mut app = App::new();
        app.add_event::<SabotageEvent>();
        app.add_systems(Update, process_ennui_system);

        let pop = app.world_mut().spawn(PopState { ennui: 10.0, is_leisure: true }).id();

        app.update();

        let events = app.world().resource::<Events<SabotageEvent>>();
        let mut cursor = events.get_cursor();
        assert!(cursor.read(events).next().is_some());

        let updated_pop = app.world().get::<PopState>(pop).unwrap();
        assert_eq!(updated_pop.ennui, 0.0);
    }
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
// Implementation provided in the test block for `process_ennui_system`.
```

**5. REFACTOR Phase: Quality & Design**
- Fine-tune the rate at which Ennui increases based on overall colony automation levels.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code

**7. Technical Guidance**
- Consider using a global `AutomationLevel` resource to scale the Ennui generation.

**8. Questions**
*Builder: add questions here if spec is unclear.*
