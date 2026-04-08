# 890 - Pop Memories

## 1. Overview
**What:** Pops accumulate "memories" from significant events they witness. These memories influence their behavior, mood, and potentially the culture of the colony.
**Why:** To add depth and history to individual Pops. A Pop who survived a famine will react differently to food shortages than a newly grown Pop. This creates emergent narrative and colony identity.

## 2. Dependencies
- Layer 1 Core (Pops, StressTracker)
- A system to emit events (like `FamineEvent` or `ShipArrivalEvent`) that Pops can "witness".

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::pre::App;

    #[test]
    fn test_pop_gains_memory_from_event() {
        let mut app = App::new();
        // Setup systems and event streams
        // ...

        let pop_entity = app.world_mut().spawn((Pop, MemoryTracker::default())).id();

        // Emit a significant event
        app.world_mut().send_event(SignificantEvent::Famine);
        app.update();

        // Pop should now have a memory of the famine
        let tracker = app.world().get::<MemoryTracker>(pop_entity).unwrap();
        assert!(tracker.has_memory(MemoryType::SurvivedFamine));
    }

    #[test]
    fn test_memory_influences_stress_reaction() {
        let mut app = App::new();
        // Setup systems
        // ...

        let mut tracker = MemoryTracker::default();
        tracker.add_memory(MemoryType::SurvivedFamine);

        let pop_entity = app.world_mut().spawn((
            Pop,
            StressTracker::new(0.0),
            tracker
        )).id();

        // Simulate a minor food shortage
        app.world_mut().send_event(FoodShortageEvent);
        app.update();

        // Pop with famine memory should react with more stress
        let stress = app.world().get::<StressTracker>(pop_entity).unwrap().current_stress;
        assert!(stress > 10.0); // Normal pop might only get 5.0 stress
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct MemoryTracker {
    pub memories: Vec<MemoryType>,
}

impl MemoryTracker {
    pub fn has_memory(&self, memory: MemoryType) -> bool {
        self.memories.contains(&memory)
    }

    pub fn add_memory(&mut self, memory: MemoryType) {
        if !self.has_memory(memory.clone()) {
            self.memories.push(memory);
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum MemoryType {
    SurvivedFamine,
    // Other memories...
}

// ... systems to process events and add memories
```

## 5. REFACTOR Phase: Quality & Design
- Consider a generic `EventWitnessed` system rather than hardcoding handlers for every single event type.
- Memories could fade over time or be replaced if a cap is introduced.
- Extract the "reaction modifier" logic out of `MemoryTracker` into a more generic `TraitModifier` system if one exists.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops correctly gain memories from designated events.

## 7. Technical Guidance
- Integrate with the existing `StressTracker` to apply modifiers.
- Use Bevy's event system to broadcast significant happenings.

## 8. Questions
*Builder: add questions here if spec is unclear.*
