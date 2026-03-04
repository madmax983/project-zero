# 273: The Feral Outpost

## 1. Overview

The untamed frontier reclaims its own. Pops separated from the colony's heart forge their own miniature society. Pops assigned to work/live far from the colony core (Command Center or high-value Social Zones) slowly accumulate a "Fringe" cultural tag. If a cluster of Fringe pops is isolated for too long, they form a "Feral Outpost" sub-faction. They still work, but refuse certain orders (like moving back, or giving up their stockpiles) and may develop unique, primitive traits.

## 2. Dependencies

- `004` Pop Entity
- `056` Designated Zones
- `068` Pop Factions
- `146` Command Center & System Visibility

## 3. RED Phase: Tests First

```rust
// src/layer1/factions/feral_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::factions::{Faction, SubFaction, FeralOutpost};
    use crate::layer1::building::{CommandCenter, BuildingGrid};
    use crate::layer1::culture::CulturalTag;

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup base systems and maps
        world.insert_resource(BuildingGrid::new(100, 100));
        world
    }

    #[test]
    fn test_pop_accumulates_fringe_tag_when_far_from_core() {
        let mut world = setup_world();

        // Core at (10, 10)
        world.spawn((
            CommandCenter,
            GridPosition { x: 10, y: 10 },
        ));

        // Pop at (80, 80)
        let pop_entity = world.spawn((
            Pop,
            GridPosition { x: 80, y: 80 },
            CulturalTag::default(),
        )).id();

        // Run system over time
        for _ in 0..100 {
            update_cultural_drift_system(&mut world);
        }

        let tags = world.get::<CulturalTag>(pop_entity).unwrap();
        assert!(tags.has_tag("Fringe"), "Pop should have acquired the Fringe tag due to distance");
    }

    #[test]
    fn test_fringe_pops_form_feral_outpost_subfaction() {
        let mut world = setup_world();

        // Spawn several pops with Fringe tag clustered together
        for i in 0..5 {
            world.spawn((
                Pop,
                GridPosition { x: 80 + i, y: 80 },
                CulturalTag::new(vec!["Fringe".to_string()]),
            ));
        }

        form_feral_outposts_system(&mut world);

        // Check if FeralOutpost faction was formed
        let mut query = world.query_filtered::<&SubFaction, With<FeralOutpost>>();
        assert_eq!(query.iter(&world).count(), 1, "A Feral Outpost subfaction should have formed");
    }

    #[test]
    fn test_feral_pops_refuse_relocation() {
        let mut world = setup_world();

        let feral_faction = world.spawn((
            SubFaction::new("The Outlanders"),
            FeralOutpost,
        )).id();

        let pop_entity = world.spawn((
            Pop,
            GridPosition { x: 80, y: 80 },
            FactionMember(feral_faction),
        )).id();

        // Attempt to reassign home zone to the core
        let result = assign_home_zone(&mut world, pop_entity, GridPosition { x: 10, y: 10 });
        assert!(result.is_err(), "Feral pops should refuse relocation orders to the core");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/factions/feral.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::factions::{FactionMember, SubFaction};
use crate::layer1::building::CommandCenter;
use crate::layer1::culture::CulturalTag;

#[derive(Component)]
pub struct FeralOutpost;

pub fn update_cultural_drift_system(
    mut world: &mut World
) {
    let mut command_center_pos = None;
    let mut cc_query = world.query::<&GridPosition, With<CommandCenter>>();
    for pos in cc_query.iter(world) {
        command_center_pos = Some(*pos);
        break; // Assume 1 core for MVP
    }

    if let Some(core_pos) = command_center_pos {
        let mut pops_query = world.query::<(Entity, &GridPosition, &mut CulturalTag), With<Pop>>();
        for (entity, pos, mut tags) in pops_query.iter_mut(world) {
            let distance = core_pos.distance(pos);
            if distance > 50.0 { // Threshold distance
                tags.add_tag("Fringe");
            }
        }
    }
}

pub fn form_feral_outposts_system(
    mut world: &mut World
) {
    // Collect fringe pops and their positions
    let mut fringe_pops = Vec::new();
    let mut pops_query = world.query::<(Entity, &GridPosition, &CulturalTag), (With<Pop>, Without<FactionMember>)>();
    for (entity, pos, tags) in pops_query.iter(world) {
        if tags.has_tag("Fringe") {
            fringe_pops.push((entity, *pos));
        }
    }

    // Simplified clustering: if we have >= 3 fringe pops unassigned, form one faction
    if fringe_pops.len() >= 3 {
        let faction_entity = world.spawn((
            SubFaction::new("Feral Outpost"),
            FeralOutpost,
        )).id();

        for (pop, _) in fringe_pops {
            world.entity_mut(pop).insert(FactionMember(faction_entity));
        }
    }
}

pub fn assign_home_zone(
    world: &mut World,
    pop_entity: Entity,
    target_pos: GridPosition,
) -> Result<(), &'static str> {
    if let Some(faction_member) = world.get::<FactionMember>(pop_entity) {
        if world.get::<FeralOutpost>(faction_member.0).is_some() {
            // Feral pops refuse orders far from their current home/outpost
            return Err("Pop is feral and refuses relocation");
        }
    }

    // Default success for MVP
    Ok(())
}
```

## 5. REFACTOR Phase: Quality & Design

- The cultural drift logic (`update_cultural_drift_system`) should gradually increment a `FringeExposure` value instead of an instant threshold, making the process take time (years/seasons).
- The clustering logic (`form_feral_outposts_system`) needs a proper spatial query to ensure the pops are actually grouped together before forming an outpost.
- `CulturalTag` needs to be defined in `src/layer1/culture.rs` if it does not exist, managing strings or enums for tag types.
- The `assign_home_zone` should check if the `target_pos` is within the Outpost's claimed territory, allowing local movement but refusing core relocation.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/factions/feral.rs`.
- [ ] Pops at a distance > 50 tiles from the Command Center gain the "Fringe" tag over time.
- [ ] Groups of 3+ "Fringe" Pops form a `FeralOutpost` subfaction.
- [ ] Feral Pops return an Error when assigned a home zone at the colony core.

## 7. Technical Guidance

- Use `bevy_ecs` spatial queries or simply a nested loop for the distance checks initially, but migrate to the spatial grid index (`map.rs`) for performance.
- When expanding this, feral outposts could eventually demand independence, linking into Layer 3 diplomacy systems (Spec `209`).

## 8. Questions

*Builder: add questions here if spec is unclear. Architect will address.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
