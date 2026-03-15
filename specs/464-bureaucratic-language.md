# 464: The Bureaucratic Language

## Overview

As the colony's administration level rises, a constructed "High Speech" is developed for official edicts and technical documentation. This language is designed to obfuscate and control. Pops without sufficient education or the "Bureaucrat" trait suffer massive delays and error rates when trying to follow orders or build advanced structures, as they literally cannot understand the instructions. This creates a tension between the security of an elite-only operational language and the catastrophic inefficiency it causes among the general populace.

## Dependencies

- `054` — Colony Edicts (Implemented)
- `051` — Pop Skills and Experience (Implemented)

## RED Phase: Tests First

```rust
// tests/bureaucratic_language_tests.rs

use bevy::prelude::*;
use scale_core::layer1::pop::{PopBundle, Traits, EducationLevel};
use scale_core::layer1::bureaucracy::{AdministrationLevel, HighSpeechEnabled, translate_instruction_system, WorkDelay};

#[test]
fn test_high_speech_causes_delay_for_uneducated_pops() {
    let mut app = App::new();
    app.insert_resource(AdministrationLevel(10));
    app.insert_resource(HighSpeechEnabled(true));
    app.add_systems(Update, translate_instruction_system);

    // Spawn uneducated pop
    let uneducated_pop = app.world_mut().spawn((
        PopBundle::default(),
        EducationLevel(1),
        // no Bureaucrat trait
    )).id();

    app.update();

    // Uneducated pop should have a WorkDelay component added due to High Speech
    assert!(app.world().entity(uneducated_pop).contains::<WorkDelay>());
}

#[test]
fn test_high_speech_no_delay_for_bureaucrats() {
    let mut app = App::new();
    app.insert_resource(AdministrationLevel(10));
    app.insert_resource(HighSpeechEnabled(true));
    app.add_systems(Update, translate_instruction_system);

    // Spawn bureaucrat pop
    let mut traits = Traits::default();
    traits.add(Trait::Bureaucrat);
    let bureaucrat_pop = app.world_mut().spawn((
        PopBundle::default(),
        EducationLevel(5),
        traits,
    )).id();

    app.update();

    // Bureaucrat should NOT have a WorkDelay component
    assert!(!app.world().entity(bureaucrat_pop).contains::<WorkDelay>());
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer1/bureaucracy.rs

use bevy::prelude::*;
use crate::layer1::pop::{Traits, Trait, EducationLevel};

#[derive(Resource, Default)]
pub struct AdministrationLevel(pub u32);

#[derive(Resource, Default)]
pub struct HighSpeechEnabled(pub bool);

#[derive(Component)]
pub struct WorkDelay {
    pub multiplier: f32,
}

pub fn translate_instruction_system(
    mut commands: Commands,
    high_speech: Res<HighSpeechEnabled>,
    query: Query<(Entity, &EducationLevel, Option<&Traits>), Without<WorkDelay>>,
) {
    if !high_speech.0 {
        return;
    }

    for (entity, education, traits) in query.iter() {
        let is_bureaucrat = traits.map_or(false, |t| t.has(Trait::Bureaucrat));

        if !is_bureaucrat && education.0 < 4 {
            commands.entity(entity).insert(WorkDelay { multiplier: 0.5 });
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Refactoring Opportunities**:
    - The `WorkDelay` component should integrate with the existing work execution system, reducing the progress made per tick.
    - We might want to add a gradual learning curve, where pops exposed to High Speech slowly increase their comprehension over time.
- **Code Smells**:
    - Hardcoded education thresholds (e.g., `education.0 < 4`). This should be a configurable constant or data-driven based on the complexity of the specific edict or building.
- **Performance**:
    - Querying all pops every tick to apply delays could be expensive. Consider applying the `WorkDelay` only when a pop receives a new task or when an edict is broadcast.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Uneducated pops without the Bureaucrat trait experience a work delay when High Speech is enabled.

## Technical Guidance

- Add `HighSpeechEnabled` and `AdministrationLevel` to the main simulation resources.
- Ensure `WorkDelay` is consumed or factored into the work execution system logic (e.g., in `work_execution_system`).
- Consider generating Chronicle events (e.g., "Confusion over new mandate") when High Speech causes severe delays.

## Questions

*Builder: add questions here if spec is unclear.*
