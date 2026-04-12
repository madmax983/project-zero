# 973: Ecosystem Mimicry

## Overview

On planets with extreme hostile fauna (e.g., swarms of apex predators), you can build "Mimicry Generators" instead of turrets. These generators emit pheromones, sounds, and visual patterns that trick the local wildlife into believing your colony is a natural part of the landscape, like a massive termite mound or a dormant predator. This provides a cheap, zero-casualty defense but removes player control over the alien ecosystem they are now integrated with. It also creates a dynamic where rival empires invading the planet will be attacked by the native fauna defending their "nest."

## Dependencies

- None (Base Layer 1 Simulation)

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_mimicry_generator_pacifies_local_fauna() {
        // Arrange: Setup world with hostile fauna and a colony building
        let mut app = App::new();
        app.add_event::<AttackEvent>();
        app.add_systems(Update, process_fauna_aggro_system);

        let colony_entity = app.world_mut().spawn(ColonyBuilding).id();
        let fauna_entity = app.world_mut().spawn(HostileFauna { aggro_target: Some(colony_entity) }).id();

        // Act: Spawn Mimicry Generator
        app.world_mut().spawn(MimicryGenerator { active: true, radius: 10.0 });
        app.update();

        // Assert: Fauna drops aggro
        let fauna = app.world().get::<HostileFauna>(fauna_entity).unwrap();
        assert!(fauna.aggro_target.is_none(), "Mimicry generator should pacify hostile fauna.");
    }

    #[test]
    fn test_mimicry_generator_redirects_fauna_to_invaders() {
        // Arrange: Setup world with pacified fauna and a Mimicry Generator
        let mut app = App::new();
        app.add_event::<AttackEvent>();
        app.add_systems(Update, process_fauna_aggro_system);

        app.world_mut().spawn(MimicryGenerator { active: true, radius: 10.0 });
        let fauna_entity = app.world_mut().spawn(HostileFauna { aggro_target: None }).id();

        // Act: Spawn Invader
        let invader_entity = app.world_mut().spawn(Invader).id();
        app.update();

        // Assert: Fauna targets invader
        let fauna = app.world().get::<HostileFauna>(fauna_entity).unwrap();
        assert_eq!(fauna.aggro_target, Some(invader_entity), "Pacified fauna should attack invaders.");
    }

    #[test]
    fn test_mimicry_generator_inactive_restores_aggro() {
         // Arrange
         let mut app = App::new();
         app.add_systems(Update, process_fauna_aggro_system);

         let colony_entity = app.world_mut().spawn(ColonyBuilding).id();
         let fauna_entity = app.world_mut().spawn(HostileFauna { aggro_target: None }).id();
         let generator_entity = app.world_mut().spawn(MimicryGenerator { active: true, radius: 10.0 }).id();
         app.update();

         // Act: Deactivate generator
         app.world_mut().get_mut::<MimicryGenerator>(generator_entity).unwrap().active = false;
         app.update();

         // Assert: Fauna regains aggro on colony
         let fauna = app.world().get::<HostileFauna>(fauna_entity).unwrap();
         assert_eq!(fauna.aggro_target, Some(colony_entity), "Inactive generator should restore aggro.");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct MimicryGenerator {
    pub active: bool,
    pub radius: f32,
}

#[derive(Component)]
pub struct HostileFauna {
    pub aggro_target: Option<Entity>,
}

#[derive(Component)]
pub struct ColonyBuilding;

#[derive(Component)]
pub struct Invader;

#[derive(Event)]
pub struct AttackEvent;

pub fn process_fauna_aggro_system(
    mut fauna_query: Query<&mut HostileFauna>,
    generator_query: Query<&MimicryGenerator>,
    invader_query: Query<Entity, With<Invader>>,
    colony_query: Query<Entity, With<ColonyBuilding>>,
) {
    let has_active_generator = generator_query.iter().any(|g| g.active);

    for mut fauna in fauna_query.iter_mut() {
        if has_active_generator {
            if let Some(invader) = invader_query.iter().next() {
                fauna.aggro_target = Some(invader);
            } else {
                fauna.aggro_target = None;
            }
        } else {
             if let Some(colony) = colony_query.iter().next() {
                 fauna.aggro_target = Some(colony);
             }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- The current implementation of `process_fauna_aggro_system` checks `has_active_generator` globally. This should be refactored to use spatial queries (checking distance against the `MimicryGenerator` radius) to only pacify fauna within the generator's area of effect.
- The aggro logic currently just picks the first available target (`.next()`). It needs a real targeting algorithm based on proximity to find the nearest valid `Invader` or `ColonyBuilding`.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/fauna/mimicry.rs` (or equivalent).
- [ ] Spatial checks are implemented so only fauna within the `radius` are affected.

## Technical Guidance

- Place this in `src/layer1/fauna/mimicry.rs`.
- You will need to extract positions (e.g. `GridPosition` or `Transform`) to calculate distances between the `MimicryGenerator`, the `HostileFauna`, and potential targets.

## Questions

*Builder: add questions here if spec is unclear.*
