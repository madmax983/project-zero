# 1350: Selective Amnesia

## Overview

Eternal Sunshine of the Spotless Mind. A "Memory Wiping" medical procedure removes "Trauma" traits and memories, restoring Pop morale. However, the procedure is imprecise and also deletes linked Skills or Relationships. This creates a difficult choice between restoring mental health versus preserving a Pop's identity and value to the colony.

## Dependencies

- `034` Pop Health and Damage
- `036` Pop Memory
- `038` Relationships

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::psychology::memory::{MemoryStore, MemoryType};
    use crate::layer1::social::relationships::RelationshipTracker;
    use crate::layer1::jobs::skills::SkillSet;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, process_memory_wipe_system);
        app
    }

    #[test]
    fn test_memory_wipe_removes_trauma_and_linked_skills() {
        let mut app = setup_app();

        let mut memory_store = MemoryStore::default();
        memory_store.add_memory(MemoryType::Trauma, "Saw friend die in combat".to_string(), 100.0);

        let mut skills = SkillSet::default();
        skills.add_skill("Combat", 50.0); // Linked skill

        let mut relationships = RelationshipTracker::default();
        relationships.add_friend(Entity::from_raw(999), 100.0);

        let pop = app.world_mut().spawn((
            Pop,
            memory_store,
            skills,
            relationships,
            PendingMemoryWipe,
        )).id();

        app.update();

        // Assert component removed
        assert!(app.world().get::<PendingMemoryWipe>(pop).is_none());

        let memory = app.world().get::<MemoryStore>(pop).unwrap();
        assert!(!memory.has_memory_type(MemoryType::Trauma));

        // Assert linked attributes degraded
        let final_skills = app.world().get::<SkillSet>(pop).unwrap();
        assert!(final_skills.get_level("Combat") < 50.0, "Skill should degrade after memory wipe");

        let final_rels = app.world().get::<RelationshipTracker>(pop).unwrap();
        assert!(final_rels.get_friend_score(Entity::from_raw(999)) < 100.0, "Relationship should degrade");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::psychology::memory::{MemoryStore, MemoryType};
use crate::layer1::social::relationships::RelationshipTracker;
use crate::layer1::jobs::skills::SkillSet;

#[derive(Component)]
pub struct PendingMemoryWipe;

pub fn process_memory_wipe_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut MemoryStore,
        &mut SkillSet,
        &mut RelationshipTracker
    ), With<PendingMemoryWipe>>
) {
    for (entity, mut memories, mut skills, mut relationships) in query.iter_mut() {
        // Find and remove all Trauma
        let had_trauma = memories.remove_memories_by_type(MemoryType::Trauma);

        if had_trauma > 0 {
            // Apply arbitrary degradation to skills and relationships for MVP
            skills.degrade_all(0.5); // 50% skill loss
            relationships.degrade_all(0.5); // 50% relationship loss
        }

        // Remove the action marker
        commands.entity(entity).remove::<PendingMemoryWipe>();
    }
}
```

## REFACTOR Phase: Quality & Design

- The degradation should be targeted, perhaps linking specific memories to specific skills (e.g. a trauma during mining degrades mining skill specifically).
- Tie the `PendingMemoryWipe` to a specific medical facility building where the action must take place.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Applying a memory wipe removes trauma.
- [ ] Applying a memory wipe degrades skills and relationships.

## Technical Guidance

- Ensure modifying `SkillSet` and `RelationshipTracker` propagates necessary UI/morale updates safely.
- You might need to add `degrade_all` or similar methods to existing components if they don't exist.

## Questions

*Builder: add questions here if spec is unclear.*
