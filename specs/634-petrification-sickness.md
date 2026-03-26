# 634: The Petrification Sickness

## Overview

A slow-creeping horror where the very environment a colony exploits begins to claim its people. Working in specific deep-crust or exotic mineral mines exposes Pops to a slow-acting pathogen or resonant frequency known as "The Petrification Sickness". Over time, an afflicted Pop's movement speed and needs decay, while their physical resistance drastically increases. Eventually, they fully "petrify," transforming from a living Pop into an unmoving statue that counts as a high-value, albeit morbid, cultural artifact. This creates a horrifying socio-economic loop where players might intentionally expose dying or unhappy Pops to harvest valuable statues to boost local room quality.

## Dependencies

- `018` Mining and Resources
- `034` Pop Health and Damage
- `061` Cultural Artifacts (Art)

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, MovementStats};
    use crate::layer1::needs::Needs;
    use crate::layer1::health::{Health, DamageResistance};
    use crate::layer1::mining::{MineType, MiningJob};
    use crate::layer1::artifacts::Artifact;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            petrification_exposure_system,
            petrification_progression_system,
            petrification_transformation_system
        ));
        app
    }

    #[test]
    fn test_mining_exotic_ore_causes_petrification_exposure() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            MiningJob { mine_type: MineType::ExoticDeepCrust }
        )).id();

        app.update();

        // Assert pop gained early stage of sickness
        let exposure = app.world().get::<PetrificationSickness>(pop);
        assert!(exposure.is_some());
        assert_eq!(exposure.unwrap().stage, 1);
    }

    #[test]
    fn test_petrification_progresses_and_alters_stats() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            MovementStats { speed: 1.0 },
            Needs { hunger: 100.0, rest: 100.0, ..Default::default() },
            DamageResistance { physical: 0.0, ..Default::default() },
            PetrificationSickness { stage: 50, max_stage: 100 }
        )).id();

        // Progress time
        app.insert_resource(Time::default());
        app.update();

        let sickness = app.world().get::<PetrificationSickness>(pop).unwrap();
        assert!(sickness.stage > 50);

        let movement = app.world().get::<MovementStats>(pop).unwrap();
        assert!(movement.speed < 1.0, "Movement speed should decrease as pop petrifies");

        let resistance = app.world().get::<DamageResistance>(pop).unwrap();
        assert!(resistance.physical > 0.0, "Physical resistance should increase as pop turns to stone");
    }

    #[test]
    fn test_full_petrification_transforms_pop_into_artifact() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            PetrificationSickness { stage: 99, max_stage: 100 }
        )).id();

        app.update();

        // Pop should no longer have the Pop component
        assert!(app.world().get::<Pop>(pop).is_none());

        // Pop should now have the Artifact component with high value
        let artifact = app.world().get::<Artifact>(pop);
        assert!(artifact.is_some());
        assert_eq!(artifact.unwrap().name, "Petrified Colonist");
        assert!(artifact.unwrap().beauty_value > 50);
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pop::{Pop, MovementStats};
use crate::layer1::needs::Needs;
use crate::layer1::health::{Health, DamageResistance};
use crate::layer1::mining::{MineType, MiningJob};
use crate::layer1::artifacts::Artifact;

#[derive(Component)]
pub struct PetrificationSickness {
    pub stage: u32,
    pub max_stage: u32,
}

pub fn petrification_exposure_system(
    mut commands: Commands,
    query: Query<(Entity, &MiningJob), Without<PetrificationSickness>>
) {
    for (entity, job) in query.iter() {
        if job.mine_type == MineType::ExoticDeepCrust {
            commands.entity(entity).insert(PetrificationSickness {
                stage: 1,
                max_stage: 100,
            });
        }
    }
}

pub fn petrification_progression_system(
    mut query: Query<(
        &mut PetrificationSickness,
        &mut MovementStats,
        &mut DamageResistance
    )>
) {
    for (mut sickness, mut movement, mut resistance) in query.iter_mut() {
        if sickness.stage < sickness.max_stage {
            sickness.stage += 1;

            // Linearly interpolate stats based on stage
            let progress = sickness.stage as f32 / sickness.max_stage as f32;
            movement.speed = 1.0 - (0.8 * progress); // Max 80% speed reduction
            resistance.physical = 50.0 * progress; // Max 50 physical resistance
        }
    }
}

pub fn petrification_transformation_system(
    mut commands: Commands,
    query: Query<(Entity, &PetrificationSickness), With<Pop>>
) {
    for (entity, sickness) in query.iter() {
        if sickness.stage >= sickness.max_stage {
            commands.entity(entity)
                .remove::<Pop>()
                .remove::<MovementStats>()
                .remove::<Needs>()
                .insert(Artifact {
                    name: "Petrified Colonist".to_string(),
                    beauty_value: 75,
                });
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Stat Modifiers**: Instead of directly mutating base `MovementStats` and `DamageResistance`, implement a modifier system or attach dynamic components so the debuffs/buffs can be recalculated cleanly.
- **Needs Decay**: Add logic to slow down the decay of `Needs` (hunger, rest) as the petrification progresses. The closer they are to stone, the less they need to eat.
- **Lore Integration**: Hook into the `Chronicle` system to generate an event when a Pop fully petrifies, noting their sacrifice/fate.
- **Cure/Treatment**: Add a late-game medical tech that can reverse the early stages of the sickness.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Working in `ExoticDeepCrust` mines exposes Pops to `PetrificationSickness`.
- [ ] The sickness correctly reduces speed and increases resistance over time.
- [ ] Reaching maximum sickness stage transforms the Pop into an `Artifact`.

## Technical Guidance

- When stripping components during the final transformation, ensure all Pop-specific active AI states and rendering tags are also removed or replaced with static artifact tags.
- Consider adding a particle effect or color tint to the Pop's sprite/model as their `stage` increases to visually indicate the petrification process to the player.

## Questions

*Builder: add questions here if spec is unclear.*
