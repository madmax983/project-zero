# 754 - The Memory Smugglers

## 1. Overview
**Layer:** 1
**Fantasy:** An underground market where trauma is erased and manufactured happiness is bought and sold.
**Mechanic:** Pops can extract their worst Memories (grief, terror) and sell them to "Memory Brokers" for resources, instantly improving their Mood. These stolen memories are then spliced into other Pops as "synthetic leisure." If a Pop accumulates too many synthetic memories, they develop "Identity Drift" and abandon their assigned jobs to wander the colony looking for people who don't exist.

## 2. Dependencies
- `036-pop-memory.md` (Memory system)
- `348-the-black-market.md` (Smuggling events/Brokers)
- `016-utility-ai-system.md` (AI evaluation to buy/sell memories)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::memory::{Memory, MemoryType};
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use bevy::prelude::*;

    #[test]
    fn test_memory_sold() {
        let mut app = App::new();
        app.add_plugins(MemorySmugglerPlugin);

        let pop_id = app.world.spawn((
            Pop::new("Traumatized Pop"),
            Needs { mood: 10.0 }, // terrible mood
            Memory { memories: vec![MemoryType::Trauma] },
            WantsToSellMemory,
        )).id();

        app.update();

        let pop = app.world.get_entity(pop_id).unwrap();
        let needs = pop.get::<Needs>().unwrap();
        let memory = pop.get::<Memory>().unwrap();

        assert!(memory.memories.is_empty());
        assert!(needs.mood > 50.0); // Mood improved after sale
    }

    #[test]
    fn test_identity_drift() {
        let mut app = App::new();
        app.add_plugins(MemorySmugglerPlugin);

        let pop_id = app.world.spawn((
            Pop::new("Addicted Pop"),
            SyntheticMemories { count: 10 }, // high count causes drift
        )).id();

        app.update();

        let pop = app.world.get_entity(pop_id).unwrap();
        assert!(pop.contains::<IdentityDrift>());

        // Ensure they lost their job assignment
        assert!(pop.get::<JobAssignment>().is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::memory::{Memory, MemoryType};
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;

pub struct MemorySmugglerPlugin;

impl Plugin for MemorySmugglerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (process_memory_sales, process_identity_drift));
    }
}

#[derive(Component)]
pub struct WantsToSellMemory;

#[derive(Component)]
pub struct SyntheticMemories {
    pub count: usize,
}

#[derive(Component)]
pub struct IdentityDrift;

#[derive(Component)]
pub struct JobAssignment;

pub fn process_memory_sales(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Memory, &mut Needs), With<WantsToSellMemory>>,
) {
    for (entity, mut memory, mut needs) in query.iter_mut() {
        // remove traumas and boost mood
        memory.memories.retain(|m| *m != MemoryType::Trauma);
        needs.mood += 50.0;

        commands.entity(entity).remove::<WantsToSellMemory>();
    }
}

pub fn process_identity_drift(
    mut commands: Commands,
    query: Query<(Entity, &SyntheticMemories), Without<IdentityDrift>>,
) {
    for (entity, synth_memories) in query.iter() {
        if synth_memories.count > 5 {
            commands.entity(entity).insert(IdentityDrift);
            commands.entity(entity).remove::<JobAssignment>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**:
    - The `IdentityDrift` state should alter the Pop's utility weights so they prioritize wandering instead of fulfilling basic needs properly.
    - Tie the Memory Brokers into the existing `BlackMarket` system, so they only appear during times of high collective trauma or via Smuggler ships.
    - Chronicle integration: "A generation forgetting its pain, wandering the halls for strangers that never lived here."

## 6. Acceptance Criteria (Testable!)
- [ ] `test_memory_sold` passes.
- [ ] `test_identity_drift` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.

## 7. Technical Guidance
- **Gotchas**: Ensure that Pops under `IdentityDrift` don't instantly starve to death by abandoning *all* logic. Give them a special wandering state that occasionally pauses to eat if desperate.

## 8. Questions
*Builder: add questions here if spec is unclear.*
