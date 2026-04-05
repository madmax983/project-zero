# 792 - Invasive Domestication

## 1. Overview
Certain biomes contain plant life that, instead of attacking, releases pheromones that rapidly fulfill the 'Leisure' and 'Social' needs of Pops. However, extended exposure causes a `Dependency` trait. Dependent Pops refuse to work, preferring to bask in the flora's aura, and will violently defend the plants if the player tries to harvest or clear them. This spec implements the `BlissWeed` flora, the `Dependency` trait, and the behavioral changes associated with it.

## 2. Dependencies
- Needs System (Leisure, Social) (`src/layer1/needs.rs`)
- Utility AI / Work Execution (`src/layer1/utility_ai.rs`)
- Terrain / Flora system (`src/layer1/map.rs`)
- Combat / Defense system (`src/layer1/defense.rs`)

## 3. RED Phase: Tests First

```rust
// tests/layer1/invasive_domestication_tests.rs

use bevy::prelude::*;
use scale::layer1::invasive_domestication::*;
use scale::layer1::needs::{NeedSocial, NeedLeisure};
use scale::layer1::utility_ai::ActionType;
use scale::layer1::pop::Pop;

#[test]
fn test_bliss_weed_boosts_needs_and_causes_dependency() {
    let mut app = App::new();
    // Register systems
    app.add_systems(Update, apply_bliss_weed_aura);

    // Spawn BlissWeed
    let weed_entity = app.world.spawn((BlissWeed { aura_radius: 5.0, dependency_threshold: 10.0 }, Transform::from_xyz(0.0, 0.0, 0.0))).id();

    // Spawn Pop nearby
    let pop_entity = app.world.spawn((
        Pop,
        NeedSocial { value: 0.0 },
        NeedLeisure { value: 0.0 },
        BlissExposure { accumulated: 0.0 },
        Transform::from_xyz(1.0, 0.0, 0.0)
    )).id();

    // Run systems to accumulate exposure
    for _ in 0..15 {
        app.update();
    }

    // Needs should be fulfilled, and Dependency trait acquired
    assert!(app.world.get::<NeedSocial>(pop_entity).unwrap().value > 50.0);
    assert!(app.world.get::<NeedLeisure>(pop_entity).unwrap().value > 50.0);
    assert!(app.world.get::<DependencyTrait>(pop_entity).is_some());
}

#[test]
fn test_dependency_blocks_work_actions() {
    let mut app = App::new();
    app.add_systems(Update, dependency_work_blocker_system);

    // Pop with Dependency trait
    let pop_entity = app.world.spawn((
        Pop,
        DependencyTrait,
        CurrentAction { action: ActionType::Work }
    )).id();

    app.update();

    // Action should be changed to Basking or Idle
    let action = app.world.get::<CurrentAction>(pop_entity).unwrap();
    assert_ne!(action.action, ActionType::Work);
}

#[test]
fn test_dependency_defends_flora() {
    let mut app = App::new();
    app.add_systems(Update, defend_bliss_weed_system);

    let weed_entity = app.world.spawn((BlissWeed { aura_radius: 5.0, dependency_threshold: 10.0 }, Transform::from_xyz(0.0, 0.0, 0.0))).id();

    let pop_entity = app.world.spawn((
        Pop,
        DependencyTrait,
        Transform::from_xyz(1.0, 0.0, 0.0),
        CurrentTarget { entity: None }
    )).id();

    let hostile_worker = app.world.spawn((
        Pop,
        CurrentAction { action: ActionType::ClearFlora { target: weed_entity } },
        Transform::from_xyz(2.0, 0.0, 0.0)
    )).id();

    app.update();

    // Dependent Pop should target the hostile worker
    let target = app.world.get::<CurrentTarget>(pop_entity).unwrap();
    assert_eq!(target.entity, Some(hostile_worker));
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/invasive_domestication.rs

use bevy::prelude::*;
use crate::layer1::needs::{NeedSocial, NeedLeisure};
use crate::layer1::utility_ai::ActionType;

#[derive(Component)]
pub struct BlissWeed {
    pub aura_radius: f32,
    pub dependency_threshold: f32,
}

#[derive(Component, Default)]
pub struct BlissExposure {
    pub accumulated: f32,
}

#[derive(Component)]
pub struct DependencyTrait;

// Simple system to apply aura
pub fn apply_bliss_weed_aura(
    weeds: Query<(&BlissWeed, &Transform)>,
    mut pops: Query<(Entity, &Transform, &mut NeedSocial, &mut NeedLeisure, &mut BlissExposure), Without<DependencyTrait>>,
    mut commands: Commands,
) {
    for (weed, weed_transform) in weeds.iter() {
        for (pop_ent, pop_transform, mut social, mut leisure, mut exposure) in pops.iter_mut() {
            if weed_transform.translation.distance(pop_transform.translation) <= weed.aura_radius {
                social.value += 10.0;
                leisure.value += 10.0;
                exposure.accumulated += 1.0;

                if exposure.accumulated >= weed.dependency_threshold {
                    commands.entity(pop_ent).insert(DependencyTrait);
                }
            }
        }
    }
}

pub fn dependency_work_blocker_system(
    mut pops: Query<&mut CurrentAction, With<DependencyTrait>>
) {
    for mut action in pops.iter_mut() {
        if let ActionType::Work = action.action {
            action.action = ActionType::Idle; // Or a specific 'Basking' action
        }
    }
}

pub fn defend_bliss_weed_system(
    weeds: Query<Entity, With<BlissWeed>>,
    hostiles: Query<(Entity, &CurrentAction)>,
    mut dependent_pops: Query<&mut CurrentTarget, With<DependencyTrait>>,
) {
    for (hostile_ent, action) in hostiles.iter() {
        if let ActionType::ClearFlora { target } = action.action {
            if weeds.contains(*target) {
                for mut pop_target in dependent_pops.iter_mut() {
                    pop_target.entity = Some(hostile_ent);
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Performance**: The distance check in `apply_bliss_weed_aura` (`O(N*M)`) could become a bottleneck if there are many weeds and Pops. Consider using a spatial hash or grid-based lookup for the aura.
- **Code Smells**: `defend_bliss_weed_system` currently makes *all* dependent Pops attack *any* hostile worker. This should probably check for proximity to the threatened weed.
- **API Improvements**: Create a specific `ActionType::Bask` instead of just falling back to `Idle` for dependent pops.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops near Bliss Weed gain Leisure/Social, accumulate exposure, and eventually gain `DependencyTrait`.
- [ ] Pops with `DependencyTrait` cancel work actions.
- [ ] Pops with `DependencyTrait` target entities attempting to clear Bliss Weed.

## 7. Technical Guidance

- Register `BlissWeed` and `DependencyTrait` components correctly.
- Add the new systems to `Layer1SystemSet::Update` (or appropriate schedule).
- Ensure `BlissExposure` decays slowly when a Pop is away from the aura, preventing accidental long-term buildup from brief encounters.

## 8. Questions

*Builder: add questions here if spec is unclear.*
