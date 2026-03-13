# Spec 329: Leader Ascension

## 1. Overview
Exceptional Pops in Layer 1 can be "promoted" to Layer 2/3 Agents (Governors, Admirals), carrying their L1 traits and history with them. This bridges the gap between individual colonists and empire-level leaders.

## 2. Dependencies
- Pop System (Skills/Traits)
- (Planned) Layer 2/3 Leader System
- Chronicle System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pop_eligible_for_ascension() {
        let mut app = setup_test_app();

        // Create a highly skilled Pop
        let pop = app.world.spawn((Pop, SkillLevel { level: 10 })).id();

        app.update();

        // Assert Pop gains an `AscensionCandidate` component
        assert!(app.world.get::<AscensionCandidate>(pop).is_some());
    }

    #[test]
    fn test_ascension_removes_from_l1_and_creates_leader() {
        let mut app = setup_test_app();

        let pop = app.world.spawn((Pop, Name("Hero".into()), Trait::Brave, AscensionCandidate)).id();

        // Trigger promotion
        app.world.send_event(PromotePopEvent { pop_entity: pop, new_role: LeaderRole::Admiral });
        app.update();

        // Assert L1 pop is gone
        assert!(app.world.get::<Pop>(pop).is_none());

        // Assert L2 leader exists with same traits
        let leader_query = app.world.query::<(&Leader, &Trait)>().iter(&app.world).count();
        assert_eq!(leader_query, 1);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Check for max level skills to assign `AscensionCandidate`.
// Handle `PromotePopEvent` by despawning the `Pop` and spawning a `Leader`.
```

## 5. REFACTOR Phase: Quality & Design
- Ensure traits translate correctly from L1 worker traits to L2 leader traits.
- Log `LEADER_PROMOTED` in the Chronicle.
- Add an UI action to manually trigger the promotion for candidates.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85%
- [ ] High-skill pops can be promoted, removing them from L1 labor pool.
- [ ] Promotions are logged in the Chronicle.

## 7. Technical Guidance
- Requires scaffolding the basic `Leader` component if Layer 2 isn't fully built out yet.

## 8. Questions
*Builder: Add any questions here.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
