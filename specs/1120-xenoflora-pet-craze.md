# Xenoflora Pet Craze

## 1. Overview
**Layer:** Layer 1
A seemingly harmless local flora becomes a viral obsession among the colonists, disrupting work but creating immense joy.
Pops randomly discover a "cute" or "soothing" local alien plant/animal and adopt it as a pet. These pets consume tiny amounts of resources but drastically boost the Pop's mood. The trend spreads virally through the colony's social network.

## 2. Dependencies
- `001-project-scaffold`

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    #[derive(Component)]
    struct Pop;

    #[derive(Component)]
    struct XenofloraPet {
        resource_upkeep: u32,
        mood_boost: f32,
    }

    #[derive(Component)]
    struct Mood {
        value: f32,
    }

    #[test]
    fn test_pet_creation() {
        let mut app = App::new();

        let pop_entity = app.world_mut().spawn((
            Pop,
            Mood { value: 50.0 },
            XenofloraPet {
                resource_upkeep: 1,
                mood_boost: 25.0,
            },
        )).id();

        let pet = app.world().get::<XenofloraPet>(pop_entity).unwrap();
        assert_eq!(pet.resource_upkeep, 1);
        assert_eq!(pet.mood_boost, 25.0);
    }

    #[test]
    fn test_pet_mood_boost() {
        let mut app = App::new();
        // Setup initial system that applies the mood boost
        app.add_systems(Update, apply_pet_mood_boost);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Mood { value: 50.0 },
            XenofloraPet {
                resource_upkeep: 1,
                mood_boost: 25.0,
            },
        )).id();

        app.update();

        let mood = app.world().get::<Mood>(pop_entity).unwrap();
        assert_eq!(mood.value, 75.0);
    }

    // Stub for the system to make it compile in RED phase initially
    fn apply_pet_mood_boost(mut query: Query<(&mut Mood, &XenofloraPet)>) {
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Mood {
    pub value: f32,
}

#[derive(Component)]
pub struct XenofloraPet {
    pub resource_upkeep: u32,
    pub mood_boost: f32,
}

pub fn apply_pet_mood_boost(mut query: Query<(&mut Mood, &XenofloraPet)>) {
    for (mut mood, pet) in query.iter_mut() {
        mood.value += pet.mood_boost;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration:** Integrate with Layer 1 resource consumption so that the pet upkeep is actually deducted from colony stores.
- **Viral Spread:** Implement a system where pops without pets have a chance to gain one if interacting with a pop that has one.
- **Starvation Risk:** Add logic to drop mood drastically if upkeep resources are not available.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code

## 7. Technical Guidance
- MVP is the component and basic mood modifier system. The viral spread can be implemented as a separate follow-up or added to social interaction systems.

## 8. Questions
*Builder: add questions here if spec is unclear.*
