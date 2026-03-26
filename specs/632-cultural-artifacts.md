# Spec 632: Cultural Artifacts

## 1. Overview
A colony that remembers its history through art. Crafters create "Art" (Statues, Tapestries, Songs) tagged with recent major Colony Memories. These items radiate auras based on their memory tags (e.g., "Victory" art buffs Courage, "Tragedy" art buffs Caution but lowers Mood).

## 2. Dependencies
- `src/layer1/items.rs` (Item creation and crafting)
- `src/layer1/pop.rs` (Pop components like Mood, Courage, Caution)
- `src/layer1/memory.rs` (Colony memory events/tags to base the art on)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Mood, Courage, Caution};
    use crate::layer1::grid::GridPosition;

    #[test]
    fn test_victory_art_aura_buffs_courage() {
        let mut app = App::new();
        app.add_systems(Update, cultural_aura_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Courage { value: 50.0 },
        )).id();

        let art_entity = app.world_mut().spawn((
            CulturalArtifact {
                theme: ArtifactTheme::Victory,
                aura_radius: 5,
            },
            GridPosition { x: 5, y: 5 }, // Same tile as Pop
        )).id();

        app.update();

        let courage = app.world().get::<Courage>(pop_entity).unwrap();
        assert!(courage.value > 50.0, "Victory art should buff Courage within its aura");
    }

    #[test]
    fn test_tragedy_art_aura_buffs_caution_and_lowers_mood() {
        let mut app = App::new();
        app.add_systems(Update, cultural_aura_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Caution { value: 50.0 },
            Mood { value: 50.0 },
        )).id();

        let art_entity = app.world_mut().spawn((
            CulturalArtifact {
                theme: ArtifactTheme::Tragedy,
                aura_radius: 5,
            },
            GridPosition { x: 7, y: 7 }, // Within radius
        )).id();

        app.update();

        let caution = app.world().get::<Caution>(pop_entity).unwrap();
        let mood = app.world().get::<Mood>(pop_entity).unwrap();

        assert!(caution.value > 50.0, "Tragedy art should buff Caution");
        assert!(mood.value < 50.0, "Tragedy art should lower Mood");
    }

    #[test]
    fn test_aura_range_limit() {
        let mut app = App::new();
        app.add_systems(Update, cultural_aura_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            GridPosition { x: 15, y: 15 },
            Courage { value: 50.0 },
        )).id();

        let art_entity = app.world_mut().spawn((
            CulturalArtifact {
                theme: ArtifactTheme::Victory,
                aura_radius: 5,
            },
            GridPosition { x: 5, y: 5 }, // Out of radius
        )).id();

        app.update();

        let courage = app.world().get::<Courage>(pop_entity).unwrap();
        assert_eq!(courage.value, 50.0, "Art outside aura radius should not affect Pop");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pop::{Courage, Caution, Mood};
use crate::layer1::grid::GridPosition;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ArtifactTheme {
    Victory,
    Tragedy,
    Feast,
}

#[derive(Component)]
pub struct CulturalArtifact {
    pub theme: ArtifactTheme,
    pub aura_radius: i32,
}

pub fn cultural_aura_system(
    artifacts: Query<(&CulturalArtifact, &GridPosition)>,
    mut pops: Query<(&mut Courage, &mut Caution, &mut Mood, &GridPosition)>,
) {
    for (artifact, art_pos) in artifacts.iter() {
        for (mut courage, mut caution, mut mood, pop_pos) in pops.iter_mut() {
            let dx = (art_pos.x - pop_pos.x).abs();
            let dy = (art_pos.y - pop_pos.y).abs();

            // Chebychev distance for simplicity
            let distance = dx.max(dy);

            if distance <= artifact.aura_radius {
                match artifact.theme {
                    ArtifactTheme::Victory => {
                        courage.value = (courage.value + 1.0).min(100.0);
                    }
                    ArtifactTheme::Tragedy => {
                        caution.value = (caution.value + 1.0).min(100.0);
                        mood.value = (mood.value - 1.0).max(0.0);
                    }
                    ArtifactTheme::Feast => {
                        mood.value = (mood.value + 1.0).min(100.0);
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Buffs/debuffs are currently applied directly every tick. A better approach is to apply transient "Aura" components or status effects to Pops that decay when they leave the radius.
- Support for different art mediums (Statue vs. Tapestry) modifying the `aura_radius` and magnitude of the effect.
- Crafters should pull from a global `ColonyChronicle` or memory ledger to decide which theme to use when producing art.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops within the aura radius receive appropriate stat modifications.
- [ ] Pops outside the aura radius are unaffected.

## 7. Technical Guidance
- Distance calculation can be tweaked depending on Grid specifications (e.g., Euclidean or Manhattan).
- Consider making the `cultural_aura_system` run on a periodic timer (e.g., once per day) rather than every tick to prevent rapid stat inflation/deflation.

## 8. Questions
*Builder: add questions here if spec is unclear.*
