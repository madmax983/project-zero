use crate::layer1::administration::edicts::{ColonyPolicies, Policy};
use crate::layer1::biology::health::Dead;
use crate::layer1::items::ItemType;
use crate::layer1::psychology::traits::{Trait, Traits};
use crate::layer1::social::morale::{MoodModifier, Morale};
use bevy::prelude::DespawnRecursiveExt;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct ColonyInventory {
    pub items: HashMap<ItemType, usize>,
}

impl ColonyInventory {
    pub fn add(&mut self, item_type: ItemType, amount: usize) {
        *self.items.entry(item_type).or_insert(0) += amount;
    }

    pub fn get_amount(&self, item_type: &ItemType) -> usize {
        self.items.get(item_type).copied().unwrap_or(0)
    }
}

#[derive(Event)]
pub struct OrganHarvestedEvent;

pub fn process_dead_pops_for_organs(
    mut commands: Commands,
    edicts: Res<ColonyPolicies>,
    mut inventory: Option<ResMut<ColonyInventory>>,
    dead_pops: Query<Entity, With<Dead>>,
    mut harvest_events: Option<ResMut<Events<OrganHarvestedEvent>>>,
) {
    if !edicts.is_active(Policy::MandatoryOrganHarvesting) {
        return;
    }

    for entity in dead_pops.iter() {
        if let Some(ref mut inv) = inventory {
            inv.add(ItemType::VitalOrgans, 1);
        }
        commands.entity(entity).despawn_recursive();
        if let Some(ref mut events) = harvest_events {
            events.send(OrganHarvestedEvent);
        }
    }
}

pub fn apply_harvesting_horror_system(
    events: Option<Res<Events<OrganHarvestedEvent>>>,
    mut pops: Query<(&mut Morale, Option<&Traits>)>,
) {
    if events.is_none() || events.as_ref().unwrap().is_empty() {
        return;
    }

    for (mut morale, traits_opt) in pops.iter_mut() {
        let is_psycho = traits_opt.is_some_and(|t| t.has(Trait::Psychopath));

        if !is_psycho {
            morale.add_modifier(MoodModifier {
                label: "Horror: Organ Harvesting".to_string(),
                value: -20.0,
                duration: 9999, // permanent/massive
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organ_harvesting_edict_produces_organs() {
        let mut world = World::new();

        // Arrange
        let mut edicts = ColonyPolicies::default();
        edicts.toggle(Policy::MandatoryOrganHarvesting);
        world.insert_resource(edicts);

        let inventory = ColonyInventory::default();
        world.insert_resource(inventory);
        world.insert_resource(Events::<OrganHarvestedEvent>::default());

        let dead_pop = world.spawn(Dead).id();

        // Act
        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            process_dead_pops_for_organs,
        );

        // Assert
        let current_inventory = world.resource::<ColonyInventory>();
        assert_eq!(current_inventory.get_amount(&ItemType::VitalOrgans), 1);
        assert!(world.get_entity(dead_pop).is_err()); // Corpse should be consumed
    }

    #[test]
    fn test_organ_harvesting_causes_horror() {
        let mut world = World::new();

        world.insert_resource(Events::<OrganHarvestedEvent>::default());

        let normal_pop = world.spawn(Morale::default()).id();

        let mut psycho_traits = Traits::default();
        psycho_traits.add(Trait::Psychopath);

        let psycho_pop = world.spawn((psycho_traits, Morale::default())).id();

        // Trigger harvest
        world
            .resource_mut::<Events<OrganHarvestedEvent>>()
            .send(OrganHarvestedEvent);

        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            apply_harvesting_horror_system,
        );

        let normal_morale = world.get::<Morale>(normal_pop).unwrap();
        let psycho_morale = world.get::<Morale>(psycho_pop).unwrap();

        assert!(!normal_morale.modifiers.is_empty()); // Took the horror penalty
        assert!(psycho_morale.modifiers.is_empty()); // Unaffected
    }
}
