# 568: The Empathic Plague

## 1. Overview
**Layer:** Layer 1
A unique pathogen sweeps through the colony. Instead of physical symptoms, it heightens the empathy of infected Pops. Infected Pops experience the negative moods (Hunger, Exhaustion, Unrest) of all Pops within a certain radius. As the infection spreads, localized pockets of misery can trigger massive, colony-wide morale collapses.

## 2. Dependencies
- `004-living-colonists`
- `016-utility-ai-system` (for Needs)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    #[derive(Component)]
    struct EmpathicPlague {
        radius: f32,
    }

    #[derive(Component)]
    struct Mood {
        value: f32, // Lower is worse
    }

    #[derive(Component)]
    struct Transform {
        translation: Vec3,
    }

    fn empathic_plague_system(
        mut q_infected: Query<(&mut Mood, &Transform, &EmpathicPlague)>,
        q_others: Query<(&Mood, &Transform), Without<EmpathicPlague>>,
    ) {
        // Implement logic to decrease infected mood based on nearby unhappy pops
    }

    #[test]
    fn test_empathic_plague_spreads_misery() {
        let mut app = App::new();
        app.add_systems(Update, empathic_plague_system);

        let infected = app.world_mut().spawn((
            Mood { value: 100.0 },
            Transform { translation: Vec3::ZERO },
            EmpathicPlague { radius: 10.0 },
        )).id();

        // Spawn a miserable neighbor nearby
        app.world_mut().spawn((
            Mood { value: 10.0 }, // Very low mood
            Transform { translation: Vec3::new(5.0, 0.0, 0.0) },
        ));

        app.update();

        // The infected pop's mood should drop because of the miserable neighbor
        let infected_mood = app.world().get::<Mood>(infected).unwrap().value;
        assert!(infected_mood < 100.0, "Infected mood should decrease due to nearby misery");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct EmpathicPlague {
    pub radius: f32,
}

// Ensure Mood component exists or use existing equivalent
```

## 5. REFACTOR Phase: Quality & Design
- **Performance:** Iterating over all nearby pops can be expensive; consider using a spatial grid or spatial hash if the population is large.
- **Integration:** Integrate this with the existing `Needs` or `Stress` system instead of a separate `Mood` component if applicable.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Infected pops' stress/mood is negatively affected by nearby stressed/unhappy pops

## 7. Technical Guidance
- Verify if `Needs` or `Stress` components exist and use those directly instead of creating a generic `Mood` component.
- The empathy transfer should ideally be a slow drain or a debuff applied over time rather than instant matching.

## 8. Questions
*Builder: add questions here if spec is unclear.*
