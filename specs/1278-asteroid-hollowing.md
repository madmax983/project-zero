# 1278: Asteroid Hollowing

## 1. Overview
**Layer:** 1

**Fantasy:** Living inside the rock. The ultimate bunker.

**Mechanic:** On Asteroid maps, you don't build *on* the surface; you mine *into* it. The asteroid shell provides massive Armor and Radiation shielding. However, mining too close to the edge weakens the "Hull Integrity".

## 2. Dependencies
- Base simulation framework (`App`, `World`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_asteroid_hollowing_basic_behavior() {
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
