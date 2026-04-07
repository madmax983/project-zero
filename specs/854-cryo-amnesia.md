# 854 - Cryo-Amnesia

## 1. Overview
Pops waking from long-term cryosleep suffer from "Memory Gaps". They might temporarily lose Skill XP or gain "False Memories" (traits/relationships that aren't real). This adds a risk to waking up new pops quickly.

## 2. Dependencies
- `src/layer1/memory.rs`
- `src/layer1/skills.rs` (or wherever XP is stored)
- `src/layer1/components.rs` (Cryosleep states)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        // Setup minimal systems
        app
    }

    #[test]
    fn test_thawing_pop_gains_amnesia() {
        let mut app = setup_app();
        // Arrange: Pop in Cryosleep
        // Act: Trigger thaw event
        // Assert: Pop gains Amnesia component/status
    }

    #[test]
    fn test_amnesia_reduces_effective_skill() {
        let mut app = setup_app();
        // Arrange: Thawed Pop with Amnesia and Level 5 Engineering
        // Act: Calculate effective skill level
        // Assert: Effective skill is treated as Level 1 or 2 temporarily
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct CryoAmnesia {
    pub severity: f32, // Decays over time
    pub false_memory: Option<String>,
}

pub fn cryo_thaw_system(
    // Thawing pops
) {
    // Add CryoAmnesia component
}

pub fn amnesia_skill_penalty_system(
    // Query pops with CryoAmnesia and Skills
) {
    // Temporarily reduce effective skill or add a failure chance to tasks
}

pub fn amnesia_recovery_system(
    // mut amnesiacs: Query<&mut CryoAmnesia>
) {
    // Decay severity over time, remove component when 0
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate False Memories directly into the `Memories` component structure from `src/layer1/memory.rs`.
- Allow medical buildings or social interactions to speed up amnesia recovery.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Waking pops receive the CryoAmnesia effect.
- [ ] Amnesia temporarily debuffs skills or introduces false memories.

## 7. Technical Guidance
- Be careful not to permanently delete the pop's base skill XP; use a temporary modifier or an "effective skill" getter.

## 8. Questions
*Builder: add questions here if spec is unclear.*
