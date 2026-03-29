# 732 - Xeno-Plague Variants

## 1. Overview
Diseases in SCALE shouldn't just be "take damage". They should be weird, offering both positive and negative traits. For example, "Crystal-Skin" might slow a pop down but give them high defense. Players might intentionally infect pops with "Stone-Lung" to make them immune to toxic gas, at the cost of a shortened lifespan. The tension lies in whether to cure the plague for safety or weaponize it for utility.

## 2. Dependencies
- Needs the underlying disease/plague system, which should handle modifiers to stats.
- Traits system.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use bevy::utils::Duration;

    // You would use standard bevy testing patterns.

    #[test]
    fn test_disease_applies_positive_and_negative_traits() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // add disease systems

        let pop_entity = app.world_mut().spawn((
            Pop,
            Health { current: 100.0, max: 100.0 },
            MovementSpeed(1.0),
            Defense(10.0),
        )).id();

        // Act
        // Apply "Crystal-Skin" plague
        app.world_mut().entity_mut(pop_entity).insert(Disease::CrystalSkin);
        app.update();

        // Assert
        let speed = app.world().get::<MovementSpeed>(pop_entity).unwrap();
        let defense = app.world().get::<Defense>(pop_entity).unwrap();

        // Crystal-Skin should slow down but increase defense
        assert!(speed.0 < 1.0);
        assert!(defense.0 > 10.0);
    }

    #[test]
    fn test_disease_shortens_lifespan() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Lifespan { remaining_years: 50 },
        )).id();

        // Act
        // Apply "Stone-Lung" plague
        app.world_mut().entity_mut(pop_entity).insert(Disease::StoneLung);
        app.update();

        // Assert
        let lifespan = app.world().get::<Lifespan>(pop_entity).unwrap();
        // Stone-Lung should shorten lifespan
        assert!(lifespan.remaining_years < 50);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component)]
pub struct Defense(pub f32);

#[derive(Component)]
pub struct Lifespan {
    pub remaining_years: u32,
}

#[derive(Component)]
pub enum Disease {
    CrystalSkin,
    StoneLung,
}

pub fn process_diseases(
    mut query: Query<(&Disease, &mut MovementSpeed, &mut Defense, &mut Lifespan), Added<Disease>>
) {
    for (disease, mut speed, mut defense, mut lifespan) in query.iter_mut() {
        match disease {
            Disease::CrystalSkin => {
                speed.0 *= 0.5; // slow
                defense.0 += 20.0; // high defense
            }
            Disease::StoneLung => {
                lifespan.remaining_years = lifespan.remaining_years.saturating_sub(10); // shortened lifespan
                // Immune to toxic gas logic would go elsewhere, handled via markers
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Currently hardcoded disease effects on components `MovementSpeed`, `Defense`, etc. We should use a more generic modifier system that is robust against base stat changes.
- `Disease` enum could be a struct with data-driven definitions loaded from config files to allow easier addition of new plagues.
- Immune to toxic gas should be implemented via a `ToxicImmunity` marker component added to the pop when `StoneLung` is present.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified (plague gives positive and negative effects)

## 7. Technical Guidance
- Integrate with the existing disease spread mechanics. These plagues should spread through the colony like normal diseases.
- Consider adding a `Cure` component/system to remove the disease and revert the modifiers.

## 8. Questions
*Builder: add questions here if spec is unclear.*
