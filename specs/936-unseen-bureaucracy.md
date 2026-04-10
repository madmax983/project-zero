# 936: Unseen Bureaucracy

## Overview

A shadow economy run by desperate pops. Under conditions of extreme systemic inefficiency (e.g., massive job backlogs, neglected infrastructure), a 'Phantom Shift' emerges. Pops with the 'Fringe' cultural tag or high desperation secretly work during the night cycle. They fix things and complete jobs but use up colony resources without logging them and slowly build an invisible, untaxable shadow economy.

## Dependencies

- None

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_phantom_shift_activation() {
        // Arrange: Setup world with high job backlog and neglected infrastructure
        let mut app = App::new();
        // Insert resources and components simulating high backlog and a Fringe pop

        // Act: Run the phantom shift system
        app.update();

        // Assert: Verify jobs completed and resources used but not logged, shadow economy increased
    }

    #[test]
    fn test_phantom_shift_requires_fringe_or_desperate_pop() {
        // Arrange: Setup world with high job backlog but NO Fringe or desperate pops
        let mut app = App::new();
        // Insert resources and components

        // Act: Run system
        app.update();

        // Assert: No jobs completed, no phantom shift occurred
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
// Minimal implementation to pass the tests
// Define components and systems for Phantom Shift
```

## REFACTOR Phase: Quality & Design

- Ensure `PhantomShift` integrates cleanly with the existing job system without creating duplicate logic.
- Consider performance implications of checking for "neglected infrastructure" across many entities.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Phantom shift correctly identifies Fringe/desperate pops and completes jobs secretly.

## Technical Guidance

- Use existing tag components for 'Fringe' or add new components for 'Desperation' if needed.
- Hook into the night cycle event or time system to trigger the shift.

## Questions

*Builder: add questions here if spec is unclear.*
