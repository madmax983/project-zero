# Generational Amnesia

## 1. Overview
**Layer:** Layer 1
As generations pass, historical knowledge is lost unless actively preserved. Pops suffer from "Generational Amnesia," slowly losing skill bonuses or lore knowledge derived from past events unless "Memorial" buildings or "Archivist" pops actively refresh their memories. This creates a tension between focusing on present survival and preserving past knowledge.

## 2. Dependencies
- `001-project-scaffold`
- `890-pop-memories` (For the base memory system)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::memory::{Memory, MemoryType};
    use scale::layer1::pops::Pop;

    #[derive(Component)]
    struct GenerationalAmnesia {
        decay_rate: f32,
        current_amnesia: f32,
    }

    #[test]
    fn test_amnesia_accumulation() {
        let mut app = App::new();

        let pop_entity = app.world_mut().spawn((
            Pop::default(),
            GenerationalAmnesia {
                decay_rate: 0.1,
                current_amnesia: 0.0,
            },
        )).id();

        // Simulate time passing (mock update)
        let mut amnesia = app.world_mut().get_mut::<GenerationalAmnesia>(pop_entity).unwrap();
        amnesia.current_amnesia += amnesia.decay_rate;

        assert_eq!(amnesia.current_amnesia, 0.1);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct GenerationalAmnesia {
    pub decay_rate: f32,
    pub current_amnesia: f32,
}

pub fn apply_amnesia_system(
    mut query: Query<&mut GenerationalAmnesia>,
) {
    for mut amnesia in query.iter_mut() {
        amnesia.current_amnesia += amnesia.decay_rate;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration:** Link `GenerationalAmnesia` to the `Memory` system so that high amnesia removes or weakens memories.
- **Counterplay:** Introduce a mechanism (like proximity to a Monument) that reduces `current_amnesia`.
- **Scaling:** Ensure `decay_rate` scales with time realistically.

## 6. Acceptance Criteria
- [ ] `GenerationalAmnesia` component exists and tracks amnesia levels.
- [ ] Tests verify amnesia accumulation over time.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code

## 7. Technical Guidance
- Keep the MVP simple: just accumulate a float value representing amnesia.
- Tie it into the utility AI later to affect job performance if amnesia is too high.

## 8. Questions
*Builder: add questions here if spec is unclear.*
