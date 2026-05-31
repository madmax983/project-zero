# 1279: Weaponized Tourism

## 1. Overview
**Layer:** Cross-layer

**Fantasy:** Killing them with kindness.

**Mechanic:** You send your own "Tourist" pops to rival colonies. They pay well but are programmed to be "Difficult" (complain, break things, spread contrary Ethics). If the rival harms them, you get a Casus Belli.

## 2. Dependencies
- Base simulation framework (`App`, `World`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_weaponized_tourism_basic_behavior() {
        // Arrange
        let mut app = App::new();
        // Act
        // Assert
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Components and systems here
```

## 5. REFACTOR Phase: Quality & Design
- TBD

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Implement MVP

## 8. Questions
*Builder: add questions here if spec is unclear.*
