# Spec 328: Technological Regression

## 1. Overview
If a specific job type isn't performed for a long time, the colony loses the associated "Tech Level". Advanced buildings become inoperable "Black Boxes" until the knowledge is rediscovered or retaught, simulating the loss of generational knowledge.

## 2. Dependencies
- Job System
- Technology/Knowledge System
- Building System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_decay_over_time() {
        let mut app = setup_test_app();

        // Set initial knowledge
        app.world.resource_mut::<TechKnowledge>().set_level(TechField::Medicine, 5);

        // Advance time significantly without any Medicine jobs
        advance_time(&mut app, 1000.0);

        // Assert knowledge dropped
        let level = app.world.resource::<TechKnowledge>().get_level(TechField::Medicine);
        assert!(level < 5);
    }

    #[test]
    fn test_building_becomes_inoperable_on_regression() {
        let mut app = setup_test_app();

        app.world.resource_mut::<TechKnowledge>().set_level(TechField::Medicine, 1);
        let mri = app.world.spawn((Building, TechRequirement(TechField::Medicine, 3))).id();

        app.update();

        // Assert building is disabled
        assert!(app.world.get::<DisabledState>(mri).is_some());
    }

    #[test]
    fn test_working_job_maintains_knowledge() {
        let mut app = setup_test_app();

        app.world.resource_mut::<TechKnowledge>().set_level(TechField::Medicine, 5);
        app.world.spawn((Pop, JobTracker { job: JobType::Doctor, active_ticks: 100 }));

        advance_time(&mut app, 1000.0);

        // Assert knowledge didn't decay due to active worker
        let level = app.world.resource::<TechKnowledge>().get_level(TechField::Medicine);
        assert_eq!(level, 5);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Track last_worked_tick for each TechField.
// If current_tick - last_worked_tick > THRESHOLD, decrement TechLevel.
// Disable buildings whose TechRequirement > current TechLevel.
```

## 5. REFACTOR Phase: Quality & Design
- Create a `KnowledgeDecaySystem` that runs infrequently (e.g., every season).
- Fire a `KNOWLEDGE_LOST` chronicle event when a level drops.
- Give a visual indicator (like a "Black Box" icon) over disabled advanced buildings.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85%
- [ ] Tech levels decay without active workers.
- [ ] Buildings disable when required tech is lost.

## 7. Technical Guidance
- Tie `JobType` to `TechField` via a mapping resource.
- Decay should be slow—measured in in-game years, not days.

## 8. Questions
*Builder: Add any questions here.*
