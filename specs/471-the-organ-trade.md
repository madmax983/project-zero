# The Organ Trade (Spec 471)

## Overview
This feature introduces a cross-layer interaction (Layer 1 -> 3) where the player can enact a "Mandatory Organ Harvesting" edict. Dead Pops or Prisoners can be processed into "Vital Organs," which trade for astronomical prices on the Layer 3 market, instantly solving economic crises. However, this creates a permanent, massive "Horror" debuff to all non-psychopathic Pops, and Layer 3 pacifist empires will embargo the colony.

## Dependencies
- `054` Colony Edicts (Implemented)
- `039` Trade System (Implemented)
- `072` Justice System (Implemented)

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use super::*;

    #[test]
    fn test_organ_harvesting_edict_produces_organs() {
        let mut world = World::new();

        // Arrange
        let mut edicts = Edicts::default();
        edicts.active.insert(EdictType::MandatoryOrganHarvesting);
        world.insert_resource(edicts);

        let mut inventory = ColonyInventory::default();
        world.insert_resource(inventory);

        let dead_pop = world.spawn((
            PopBundle::default(),
            Dead,
        )).id();

        // Act
        world.run_system_once(process_dead_pops_for_organs).unwrap();

        // Assert
        let current_inventory = world.resource::<ColonyInventory>();
        assert_eq!(current_inventory.get_amount(&ItemType::VitalOrgans), 1);
        assert!(world.get_entity(dead_pop).is_none()); // Corpse should be consumed
    }

    #[test]
    fn test_organ_harvesting_causes_horror() {
        let mut world = World::new();

        world.insert_resource(Events::<OrganHarvestedEvent>::default());

        let normal_pop = world.spawn((
            PopBundle::default(),
            Needs { morale: 100.0, ..Default::default() },
        )).id();

        let psycho_pop = world.spawn((
            PopBundle::default(),
            Trait::Psychopath,
            Needs { morale: 100.0, ..Default::default() },
        )).id();

        // Trigger harvest
        world.resource_mut::<Events<OrganHarvestedEvent>>().send(OrganHarvestedEvent);

        world.run_system_once(apply_harvesting_horror_system).unwrap();

        let normal_needs = world.get::<Needs>(normal_pop).unwrap();
        let psycho_needs = world.get::<Needs>(psycho_pop).unwrap();

        assert!(normal_needs.morale < 100.0); // Took the horror penalty
        assert_eq!(psycho_needs.morale, 100.0); // Unaffected
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct OrganHarvestedEvent;

pub fn process_dead_pops_for_organs(
    mut commands: Commands,
    edicts: Res<Edicts>,
    mut inventory: ResMut<ColonyInventory>,
    dead_pops: Query<Entity, With<Dead>>,
    mut harvest_events: EventWriter<OrganHarvestedEvent>,
) {
    if !edicts.active.contains(&EdictType::MandatoryOrganHarvesting) {
        return;
    }

    for entity in dead_pops.iter() {
        inventory.add(ItemType::VitalOrgans, 1);
        commands.entity(entity).despawn_recursive();
        harvest_events.send(OrganHarvestedEvent);
    }
}

pub fn apply_harvesting_horror_system(
    mut events: EventReader<OrganHarvestedEvent>,
    mut pops: Query<(&mut Needs, Option<&Trait>)>,
) {
    if events.read().next().is_none() {
        return;
    }

    for (mut needs, trait_opt) in pops.iter_mut() {
        let is_psycho = trait_opt.map_or(false, |t| *t == Trait::Psychopath);

        if !is_psycho {
            needs.morale -= 20.0; // Flat massive penalty
            needs.morale = needs.morale.max(0.0);
        }
    }
}
```

## REFACTOR Phase: Quality & Design
- **Action Evaluation**: Integrate into `Utility AI` where pops might actively refuse to work or riot if the "Horror" modifier stacks too high.
- **Layer 3 Integration**: The embargo effect needs to hook into the diplomacy/trade system (Layer 3) to physically prevent merchant ships from Pacifist factions from docking.
- **Visuals**: A "Harvesting Facility" building should be required rather than abstractly processing corpses anywhere on the map.

## Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Enacting the edict consumes dead pops to produce `VitalOrgans`.
- [ ] Harvesting triggers a massive morale penalty on all pops lacking the `Psychopath` trait.

## Technical Guidance
- `ItemType::VitalOrgans` should be given a very high base trade value in the item registry.
- For Layer 3 embargoes, emit a specific event when the edict is passed that the diplomacy system can listen for, rather than tight-coupling Layer 1 and Layer 3 directly.

## Questions
*Builder: add questions here if spec is unclear.*
