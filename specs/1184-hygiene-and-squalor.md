# 1184: Hygiene & Squalor

## Overview

A clean ship is a happy ship. Space is dirty, and disease breeds in filth. Pops accumulate "Filth" over time from work and sweat. Constructible "Showers" consume Water to remove it. High Filth increases Disease risk and causes a "Repulsive" social debuff (other pops avoid them). This forces tension between water usage (scarcity) and health/social cohesion.

## Dependencies

- `004` — Pop Entity
- `005` — Pop Needs

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::needs::{Filth, DiseaseRisk, SocialRepulsion};
    use crate::layer1::systems::hygiene::filth_accumulation_system;
    use crate::layer1::systems::hygiene::shower_system;
    use crate::layer1::resources::WaterSupply;

    #[test]
    fn test_filth_accumulation() {
        let mut app = App::new();
        app.add_systems(Update, filth_accumulation_system);

        let entity = app.world_mut().spawn(Filth(0.0)).id();

        app.update();

        let filth = app.world().get::<Filth>(entity).unwrap();
        assert!(filth.0 > 0.0, "Filth should accumulate over time");
    }

    #[test]
    fn test_high_filth_causes_repulsion_and_disease_risk() {
        let mut app = App::new();
        app.add_systems(Update, filth_accumulation_system);

        let entity = app.world_mut().spawn(Filth(90.0)).id();

        app.update();

        assert!(app.world().get::<SocialRepulsion>(entity).is_some(), "High filth should add SocialRepulsion component");
        assert!(app.world().get::<DiseaseRisk>(entity).unwrap().0 > 0.0, "High filth should increase DiseaseRisk");
    }

    #[test]
    fn test_shower_cleans_filth_and_consumes_water() {
        let mut app = App::new();
        app.world_mut().insert_resource(WaterSupply { amount: 10.0 });
        app.add_systems(Update, shower_system);

        let entity = app.world_mut().spawn(Filth(50.0)).id();

        // Simulating the pop using a shower
        app.update();

        let filth = app.world().get::<Filth>(entity).unwrap();
        let water = app.world().get_resource::<WaterSupply>().unwrap();

        // Assert that taking a shower reduces filth to 0 or lowers it significantly, and consumes water
        assert_eq!(filth.0, 0.0, "Shower should clean all filth");
        assert!(water.amount < 10.0, "Shower should consume water");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Filth(pub f32);

#[derive(Component)]
pub struct SocialRepulsion;

#[derive(Component)]
pub struct DiseaseRisk(pub f32);

#[derive(Resource)]
pub struct WaterSupply {
    pub amount: f32,
}

pub fn filth_accumulation_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Filth, Option<&mut DiseaseRisk>)>,
) {
    for (entity, mut filth, disease_risk) in query.iter_mut() {
        // Accumulate filth over time (e.g. 1 unit per tick)
        filth.0 += 1.0;

        if filth.0 >= 80.0 {
            commands.entity(entity).insert(SocialRepulsion);

            if let Some(mut risk) = disease_risk {
                risk.0 += 0.5;
            } else {
                commands.entity(entity).insert(DiseaseRisk(0.5));
            }
        }
    }
}

pub fn shower_system(
    mut water: ResMut<WaterSupply>,
    mut query: Query<&mut Filth>,
) {
    // Basic interaction implementation - iterates over all pops for now
    for mut filth in query.iter_mut() {
        if filth.0 > 0.0 && water.amount >= 1.0 {
            water.amount -= 1.0;
            filth.0 = 0.0;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Interaction Driven**: `shower_system` shouldn't instantly clean everyone. Pops should need an action (Utility AI) to walk to a `Shower` entity.
- **Gradual Removal**: Remove `SocialRepulsion` when Filth drops below the threshold.
- **Disease Scaling**: Tie `DiseaseRisk` increase directly to the amount of `Filth` above the threshold.

## Acceptance Criteria

- [ ] Pops correctly accumulate `Filth` over time.
- [ ] Pops with high `Filth` gain `SocialRepulsion` and increased `DiseaseRisk`.
- [ ] Pops can use showers to reduce `Filth` to 0.
- [ ] Using showers consumes `WaterSupply`.
- [ ] `cargo test` returns 0 failures and test coverage >= 85% for new code.
- [ ] `cargo clippy -- -D warnings` passes.

## Technical Guidance

- Integrate `Filth` accumulation into the `metabolism_system` or similar time-based update loops.
- `Shower` should be added as a building that Pops can path to when their `Filth` exceeds a certain threshold (Utility AI task).

## Questions

*Builder: add questions here if spec is unclear.*
