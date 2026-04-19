# Sub-light Tourism

## 1. Overview
**Layer:** Layer 2
Wealthy individuals travel for decades in cryo-sleep just to witness rare planetary events or historical sites, arriving as "Sub-light Tourists."

## 2. Dependencies
- `001-project-scaffold`

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    #[derive(Component)]
    struct SubLightTourist {
        wealth: u32,
        target_event_id: String,
    }

    #[test]
    fn test_tourist_creation() {
        let mut app = App::new();

        let tourist_entity = app.world_mut().spawn((
            SubLightTourist {
                wealth: 5000,
                target_event_id: "Supernova_A".to_string(),
            },
        )).id();

        let tourist = app.world().get::<SubLightTourist>(tourist_entity).unwrap();
        assert_eq!(tourist.wealth, 5000);
        assert_eq!(tourist.target_event_id, "Supernova_A");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct SubLightTourist {
    pub wealth: u32,
    pub target_event_id: String,
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration:** Link with the economy to allow colonies to profit from tourism.

## 6. Acceptance Criteria
- [ ] `SubLightTourist` component exists.
- [ ] Tests verify creation.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code

## 7. Technical Guidance
- MVP is just the data structure.

## 8. Questions
*Builder: add questions here if spec is unclear.*
