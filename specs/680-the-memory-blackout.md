# 680 - The Memory Blackout

## 1. Overview
The Memory Blackout is a Layer 1 mechanic where a rare atmospheric event or experimental tech failure triggers a "Memory Blackout." All Pops lose memories formed within a specific time window. Relationships reset, grudges disappear, but vital learned skills or known danger zones are also forgotten. It forces the player to weigh whether intentionally triggering blackouts to erase catastrophic morale penalties (like a massacre) is worth losing crucial survival knowledge.

## 2. Dependencies
- Layer 1 `Pop` entity.
- Layer 1 `Memory` system (to be modified).
- Layer 1 `Relationships` component.
- Layer 1 `Skills` component (or equivalent knowledge tracking).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::memory::{Memory, MemoryEntry};
    use crate::layer1::relationships::Relationships;
    use crate::layer1::skills::Skills;

    #[test]
    fn test_memory_blackout_removes_recent_memories() {
        let mut app = App::new();
        app.add_systems(Update, process_memory_blackout);

        let pop = app.world_mut().spawn((
            Pop {},
            Memory {
                entries: vec![
                    MemoryEntry { timestamp: 100, data: "Old Grudge".into() },
                    MemoryEntry { timestamp: 500, data: "Recent Trauma".into() },
                ]
            }
        )).id();

        // Trigger a memory blackout covering time window 400 to 600
        app.world_mut().send_event(MemoryBlackoutEvent {
            start_time: 400,
            end_time: 600,
        });

        app.update();

        let memory = app.world().get::<Memory>(pop).unwrap();
        assert_eq!(memory.entries.len(), 1, "Recent memory should be erased");
        assert_eq!(memory.entries[0].data, "Old Grudge".to_string(), "Old memory should be retained");
    }

    #[test]
    fn test_memory_blackout_resets_recent_relationships() {
        let mut app = App::new();
        app.add_systems(Update, process_memory_blackout);

        let pop1 = app.world_mut().spawn(Pop {}).id();
        let pop2 = app.world_mut().spawn(Pop {}).id();

        app.world_mut().entity_mut(pop1).insert(Relationships {
            bonds: vec![(pop2, 50, 450)], // 450 is timestamp formed
        });

        app.world_mut().send_event(MemoryBlackoutEvent {
            start_time: 400,
            end_time: 600,
        });

        app.update();

        let relationships = app.world().get::<Relationships>(pop1).unwrap();
        assert!(relationships.bonds.is_empty(), "Recent relationships should be reset");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop {}

pub struct MemoryEntry {
    pub timestamp: u32,
    pub data: String,
}

#[derive(Component, Default)]
pub struct Memory {
    pub entries: Vec<MemoryEntry>,
}

#[derive(Component, Default)]
pub struct Relationships {
    // (Other Pop Entity, Affinity Score, Timestamp Formed)
    pub bonds: Vec<(Entity, i32, u32)>,
}

#[derive(Event)]
pub struct MemoryBlackoutEvent {
    pub start_time: u32,
    pub end_time: u32,
}

pub fn process_memory_blackout(
    mut events: EventReader<MemoryBlackoutEvent>,
    mut query: Query<(Option<&mut Memory>, Option<&mut Relationships>), With<Pop>>,
) {
    for event in events.read() {
        for (mut memory_opt, mut relations_opt) in query.iter_mut() {
            if let Some(mut memory) = memory_opt {
                memory.entries.retain(|e| e.timestamp < event.start_time || e.timestamp > event.end_time);
            }
            if let Some(mut relations) = relations_opt {
                relations.bonds.retain(|&(_, _, ts)| ts < event.start_time || ts > event.end_time);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** Ensure that skills acquired within the specified time window are also correctly rolled back if applicable.
- **Code Smells:** Iterating over the memory vectors for every single Pop on the map can be slow. `retain` is generally fast, but profiling might be necessary for massive colonies.
- **Performance:** Store memories using an interval tree or bucketed by time to allow `O(log N)` or `O(1)` deletion of time ranges.
- **API Improvements:** Add an `Event` that announces the blackout to the player via the Chronicle, so they know *why* their Pops are acting strangely.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops successfully lose memories and relationships formed inside the blackout window.

## 7. Technical Guidance
- **Code Structure:** Integrate into `src/layer1/memory.rs` and `src/layer1/relationships.rs` or create `src/layer1/quirks/memory_blackout.rs`.
- **Integration Points:** The blackout could be manually triggered via `Colony Edicts` (from `specs/054-colony-edicts.md`) or spontaneously by `Planetary Quirks` (`specs/080-planetary-quirks.md`).

## 8. Questions
*Builder: add questions here if spec is unclear.*
- **Architectural Contradiction:** The spec assumes that `Relationships` contains timestamps (`pub bonds: Vec<(Entity, i32, u32)>`) and `Skills` tracks XP with timestamps. However, checking `src/layer1/social/mod.rs` shows `Relationships` is a `HashMap<Entity, f32>` without timestamps, and `src/layer1/skills/mod.rs` shows `Skills` uses `HashMap<SkillType, f32>` which also lacks timestamps. Furthermore, `Memories` uses `ActiveMemory` which has an `added_at` field of type `u64`, rather than `u32`. It is impossible to erase *recent* relationships or skills without adding timestamps to all XP gains and relationship modifications, which is a major architectural change. Should I proceed to refactor `Relationships` and `Skills` to include timestamps, or should the blackout only affect `Memories`?
