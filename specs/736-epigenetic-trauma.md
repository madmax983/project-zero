# 736 - Epigenetic Trauma

## 1. Overview
The sins of the father are visited upon the son. A generation raised in fear is different. Children born to parents with high Stress or Trauma traits inherit "Phobias" or "Resiliences". The "Starvation Generation" produces children who are "Hoarders" (eat less, store more). This creates tension between protecting the parents to protect the future, or burning them out for survival now.

## 2. Dependencies
- Layer 1 Pop generation and reproduction system.
- Stress/Trauma tracking for Pops.
- Trait inheritance mechanisms.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_offspring_inherits_trauma_as_hoarder_trait() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, apply_epigenetic_trauma_system);

        let parent = app.world_mut().spawn((
            Pop,
            Stress { level: 90.0 },
            StarvationTrauma,
        )).id();

        let child = app.world_mut().spawn((
            Pop,
            OffspringOf(parent),
        )).id();

        // Act
        app.update();

        // Assert
        // Child should have developed the Hoarder trait due to parent's StarvationTrauma and high stress
        assert!(app.world().get::<HoarderTrait>(child).is_some());
    }

    #[test]
    fn test_offspring_without_traumatized_parent_does_not_inherit() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, apply_epigenetic_trauma_system);

        let parent = app.world_mut().spawn((
            Pop,
            Stress { level: 10.0 }, // Low stress, no trauma
        )).id();

        let child = app.world_mut().spawn((
            Pop,
            OffspringOf(parent),
        )).id();

        // Act
        app.update();

        // Assert
        // Child should NOT have the Hoarder trait
        assert!(app.world().get::<HoarderTrait>(child).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Stress {
    pub level: f32,
}

#[derive(Component)]
pub struct StarvationTrauma;

#[derive(Component)]
pub struct HoarderTrait;

#[derive(Component)]
pub struct OffspringOf(pub Entity);

pub fn apply_epigenetic_trauma_system(
    mut commands: Commands,
    child_query: Query<(Entity, &OffspringOf), (With<Pop>, Without<HoarderTrait>)>,
    parent_query: Query<(&Stress, Option<&StarvationTrauma>), With<Pop>>,
) {
    for (child_entity, offspring) in child_query.iter() {
        if let Ok((stress, starvation_trauma)) = parent_query.get(offspring.0) {
            if stress.level >= 80.0 && starvation_trauma.is_some() {
                commands.entity(child_entity).insert(HoarderTrait);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Abstract the specific `StarvationTrauma` -> `HoarderTrait` mapping into a data-driven configuration or a broader `EpigeneticMapping` resource so more trauma-to-trait conversions can be easily added (e.g., Agoraphobia from overcrowding).
- Ensure the system only runs once upon the child's birth/spawning, possibly by listening to a `PopBornEvent` instead of running a query every frame on all children.
- Integrate `HoarderTrait` into the actual resource consumption logic so Hoarders eat less but try to cache food in their private stashes.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Offspring correctly inherit specific traits based on parent's high stress and trauma markers.

## 7. Technical Guidance
- Hook this logic into the existing reproduction or pop spawning pipeline. A command or an event observer is preferable to a continuously running system.
- `HoarderTrait` should interact with Layer 1's logistics and inventory. Hoarders might have a personal inventory or just apply a modifier to their food consumption rate.

## 8. Questions
*Builder: add questions here if spec is unclear.*
