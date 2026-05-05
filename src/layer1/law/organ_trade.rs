use crate::layer1::administration::edicts::{ColonyPolicies, Policy};
use crate::layer1::economy::items::ItemType;
use crate::layer1::health::Dead;
use crate::layer1::psychology::traits::{Trait, Traits};
use crate::layer1::social::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct OrganHarvestedEvent;

#[derive(Resource, Default)]
pub struct ColonyInventory {
    pub organs: usize,
}

impl ColonyInventory {
    pub fn get_amount(&self, item_type: &ItemType) -> usize {
        match item_type {
            ItemType::VitalOrgans => self.organs,
            _ => 0,
        }
    }

    pub fn add(&mut self, item_type: ItemType, amount: usize) {
        if item_type == ItemType::VitalOrgans {
            self.organs += amount;
        }
    }
}

pub fn process_dead_pops_for_organs(
    mut commands: Commands,
    edicts: Res<ColonyPolicies>,
    mut inventory: ResMut<ColonyInventory>,
    dead_pops: Query<Entity, With<Dead>>,
    mut harvest_events: EventWriter<OrganHarvestedEvent>,
) {
    if !edicts.is_active(Policy::MandatoryOrganHarvesting) {
        return;
    }

    for entity in dead_pops.iter() {
        inventory.add(ItemType::VitalOrgans, 1);
        commands.entity(entity).despawn();
        harvest_events.send(OrganHarvestedEvent);
    }
}

pub fn apply_harvesting_horror_system(
    mut events: EventReader<OrganHarvestedEvent>,
    mut pops: Query<(&mut Morale, Option<&Traits>)>,
) {
    let count = events.read().count();
    if count == 0 {
        return;
    }

    for (mut morale, trait_opt) in pops.iter_mut() {
        let is_psycho = trait_opt.is_some_and(|t| t.has(Trait::Cannibal));

        if !is_psycho {
            morale.modifiers.push(MoodModifier {
                label: "Organ Harvesting Horror".to_string(),
                value: -0.2 * count as f32, // Apply penalty multiplied by count
                duration: 1000,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_organ_harvesting_edict_produces_organs() {
        let mut world = World::new();

        let mut edicts = ColonyPolicies::default();
        edicts
            .active_policies
            .insert(Policy::MandatoryOrganHarvesting);
        world.insert_resource(edicts);

        let inventory = ColonyInventory::default();
        world.insert_resource(inventory);
        world.init_resource::<Events<OrganHarvestedEvent>>();

        let dead_pop = world.spawn((Pop, Dead)).id();

        let _ = world.run_system_once(process_dead_pops_for_organs);

        let current_inventory = world.resource::<ColonyInventory>();
        assert_eq!(current_inventory.get_amount(&ItemType::VitalOrgans), 1);
        assert!(world.get_entity(dead_pop).is_err());
    }

    #[test]
    fn test_organ_harvesting_causes_horror() {
        let mut world = World::new();
        world.init_resource::<Events<OrganHarvestedEvent>>();

        let normal_pop = world
            .spawn((
                Pop,
                Morale {
                    value: 1.0,
                    modifiers: vec![],
                },
            ))
            .id();

        let mut psycho_traits = Traits::default();
        psycho_traits.add(Trait::Cannibal);

        let psycho_pop = world
            .spawn((
                Pop,
                psycho_traits,
                Morale {
                    value: 1.0,
                    modifiers: vec![],
                },
            ))
            .id();

        world
            .resource_mut::<Events<OrganHarvestedEvent>>()
            .send(OrganHarvestedEvent);

        let _ = world.run_system_once(apply_harvesting_horror_system);

        let normal_morale = world.get::<Morale>(normal_pop).unwrap();
        let psycho_morale = world.get::<Morale>(psycho_pop).unwrap();

        assert!(!normal_morale.modifiers.is_empty());
        assert!(psycho_morale.modifiers.is_empty());
    }
}
