# Ancestral Graves

## 1. Overview
The history of the colony is written on the land itself. Dead pops leave "Grave" tiles. Relatives visit graves for mood buffs. Graves cannot be built over without a severe "Sacrilege" penalty.

## 2. Dependencies
- Core architecture (Specs 001-013)
- Pop Memory (Spec 036)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_grave_provides_mood_buff_to_visitor() {
        // Arrange
        let mut world = World::new();
        let grave = world.spawn((Grave, Position(0, 0))).id();
        let visitor = world.spawn((Pop, Needs::default(), Position(1, 0))).id();

        // Act
        world.run_system_once(grave_visit_system).unwrap();

        // Assert
        let needs = world.get::<Needs>(visitor).unwrap();
        assert!(needs.leisure > 0.8, "Visiting grave should restore leisure/mood");
    }

    #[test]
    fn test_building_over_grave_causes_sacrilege() {
        // Arrange
        let mut world = World::new();
        let grave_pos = Position(0, 0);
        world.spawn((Grave, grave_pos));
        let mut events = Events::<BuildEvent>::default();
        events.send(BuildEvent { pos: grave_pos, building: BuildingType::Wall });
        world.insert_resource(events);

        // Act
        world.run_system_once(build_system_wrapper).unwrap();

        // Assert
        // Check that a Sacrilege event/modifier was created
        let sacrilege_events = world.get_resource::<Events<SacrilegeEvent>>().unwrap();
        assert_eq!(sacrilege_events.len(), 1, "Building over a grave should trigger sacrilege");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// `grave_visit_system`: Checks proximity of pops to graves and boosts their morale/leisure.
// Update building placement to check for `Grave` entities and emit `SacrilegeEvent` if overlapping.
```

## 5. REFACTOR Phase: Quality & Design
- Hook the Sacrilege event into the global morale or colony chronicle.
- Restrict visits to relatives or friends if the relationship system supports it.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Coverage >= 85% for new code

## 7. Technical Guidance
- Consider if Graves are terrain modifiers or entities. Entities are easier to attach data (who is buried here) to.

## 8. Questions
*Builder: add questions here if spec is unclear.*
