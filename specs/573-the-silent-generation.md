# 573 The Silent Generation

## 1. Overview
A Layer 1 generational trait mechanic where a generation of Pops born during a time of extreme trauma (prolonged famine or massive death toll) gain the "Silent" trait. These Pops are highly resilient to stress and work tirelessly, but they possess zero social needs, refuse leisure activities, and generate no cultural artifacts. This creates a highly efficient but culturally dead society.

## 2. Dependencies
- Layer 1 `Pop` entity, `Needs` (Stress, Social, Leisure)
- `BirthEvent` or Pop spawning system
- Colony-wide `TraumaTracker` (recording recent death tolls or famine duration)
- `Trait` system

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_pop_born_during_high_trauma_gets_silent_trait() {
        let mut app = App::new();
        // Setup ...
        app.world.insert_resource(TraumaTracker { recent_deaths: 50, famine_ticks: 1000 });

        let newborn = app.world.spawn(Pop).id();
        app.world.send_event(BirthEvent { entity: newborn });

        app.update(); // Run generation traits system

        assert!(app.world.get::<TraitSilent>(newborn).is_some());
    }

    #[test]
    fn test_silent_trait_prevents_social_and_leisure_needs() {
        let mut app = App::new();
        // Setup ...
        let pop = app.world.spawn((Pop, TraitSilent, Needs::default())).id();

        app.update(); // Run needs decay system

        let needs = app.world.get::<Needs>(pop).unwrap();
        // Silent pops shouldn't even track social/leisure, or it should remain 0/disabled
        assert_eq!(needs.social, 0.0);
        assert_eq!(needs.leisure, 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Resource, Default)]
pub struct TraumaTracker {
    pub recent_deaths: u32,
    pub famine_ticks: u32,
}

#[derive(Component)]
pub struct TraitSilent;

pub fn assign_generational_traits_system(
    mut events: EventReader<BirthEvent>,
    trauma: Res<TraumaTracker>,
    mut commands: Commands,
) {
    for event in events.read() {
        // Thresholds for "extreme trauma"
        if trauma.recent_deaths >= 20 || trauma.famine_ticks >= 500 {
            commands.entity(event.entity).insert(TraitSilent);
        }
    }
}

pub fn silent_needs_suppression_system(
    mut query: Query<&mut Needs, With<TraitSilent>>,
) {
    for mut needs in query.iter_mut() {
        needs.social = 0.0;
        needs.leisure = 0.0;
        // Suppress stress to simulate resilience
        needs.stress = needs.stress.min(50.0);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: `silent_needs_suppression_system` constantly overwriting needs to 0.0 is brute-force. A better approach is to remove the `SocialNeed` and `LeisureNeed` components entirely from the entity if using a fragmented component structure, or have the Utility AI completely ignore these needs if the trait is present.
- **Integration**: The `TraumaTracker` needs a system to slowly decay `recent_deaths` and `famine_ticks` over time so the era of trauma eventually ends.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops born during high trauma periods acquire the `TraitSilent` component.
- [ ] Pops with `TraitSilent` do not accumulate Social or Leisure needs and have a capped Stress response.

## 7. Technical Guidance
- Ensure the Utility AI assigns exactly 0.0 score to any `ActionType::Socialize` or `ActionType::Play` for Pops with the `TraitSilent` component to enforce the "culturally dead" aspect.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
