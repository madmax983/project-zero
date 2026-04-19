# The Diaspora Fleet

## 1. Overview
**Layer:** Layer 2
Sub-light refugee fleets arrive periodically from other failing colonies or events. They carry diverse pops with unique traits, but also strain local resources.

## 2. Dependencies
- `001-project-scaffold`

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    #[derive(Component)]
    struct DiasporaFleet {
        refugee_count: u32,
        arrival_time: f32,
    }

    #[test]
    fn test_diaspora_fleet_creation() {
        let mut app = App::new();

        let fleet_entity = app.world_mut().spawn((
            DiasporaFleet {
                refugee_count: 100,
                arrival_time: 10.0,
            },
        )).id();

        let fleet = app.world().get::<DiasporaFleet>(fleet_entity).unwrap();
        assert_eq!(fleet.refugee_count, 100);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct DiasporaFleet {
    pub refugee_count: u32,
    pub arrival_time: f32,
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration:** Trigger population increase events upon arrival.

## 6. Acceptance Criteria
- [ ] `DiasporaFleet` component exists.
- [ ] Tests verify creation.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code

## 7. Technical Guidance
- MVP is just the data structure.

## 8. Questions
*Builder: add questions here if spec is unclear.*
