use crate::layer1::edicts::{ColonyPolicies, Policy};
use crate::layer1::health::Dead;
use crate::layer1::items::ItemType;
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::traits::Traits;
use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct ColonyInventory {
    pub items: std::collections::HashMap<ItemType, u32>,
}

impl ColonyInventory {
    pub fn add(&mut self, item: ItemType, amount: u32) {
        *self.items.entry(item).or_insert(0) += amount;
    }
    pub fn get_amount(&self, item: &ItemType) -> u32 {
        *self.items.get(item).unwrap_or(&0)
    }
}

pub type Edicts = ColonyPolicies;
#[allow(non_snake_case)]
pub mod EdictType {
    #[allow(non_upper_case_globals)]
    pub const MandatoryOrganHarvesting: super::Policy = super::Policy::MandatoryOrganHarvesting;
}

#[derive(Event, Default)]
pub struct OrganHarvestedEvent;

pub fn process_dead_pops_for_organs(
    mut commands: Commands,
    edicts: Option<Res<Edicts>>,
    inventory: Option<ResMut<ColonyInventory>>,
    dead_pops: Query<Entity, With<Dead>>,
    mut harvest_events: EventWriter<OrganHarvestedEvent>,
) {
    let Some(edicts) = edicts else { return };
    let Some(mut inventory) = inventory else {
        return;
    };

    if !edicts
        .active_policies
        .contains(&EdictType::MandatoryOrganHarvesting)
    {
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
    let mut harvest_count = 0;
    for _ in events.read() {
        harvest_count += 1;
    }

    if harvest_count == 0 {
        return;
    }

    for (mut morale, traits_opt) in pops.iter_mut() {
        let is_psycho = traits_opt.is_some_and(|t| t.has(crate::layer1::traits::Trait::Psychopath));

        if !is_psycho {
            for _ in 0..harvest_count {
                morale.modifiers.push(MoodModifier {
                    label: "Organ Harvesting Horror".to_string(),
                    value: -20.0,
                    duration: 9999999, // Permanent as per spec
                });
                morale.value -= 20.0;
                if morale.value < 0.0 {
                    morale.value = 0.0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::PopBundle;
    use crate::layer1::traits::Trait;
    use rand::thread_rng;

    #[test]
    fn test_organ_harvesting_edict_produces_organs() {
        let mut world = World::new();

        // Arrange
        let mut edicts = Edicts::default();
        edicts
            .active_policies
            .insert(EdictType::MandatoryOrganHarvesting);
        world.insert_resource(edicts);

        let inventory = ColonyInventory::default();
        world.insert_resource(inventory);
        world.init_resource::<Events<OrganHarvestedEvent>>();

        let mut rng = thread_rng();
        let dead_pop = world.spawn((PopBundle::random(0, 0, &mut rng), Dead)).id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(process_dead_pops_for_organs);
        schedule.run(&mut world);

        // Assert
        let current_inventory = world.resource::<ColonyInventory>();
        assert_eq!(current_inventory.get_amount(&ItemType::VitalOrgans), 1);
        assert!(world.get_entity(dead_pop).is_err()); // Corpse should be consumed
    }

    #[test]
    fn test_organ_harvesting_causes_horror() {
        let mut world = World::new();

        world.init_resource::<Events<OrganHarvestedEvent>>();

        let mut rng = thread_rng();

        // Spawn normal pop
        let mut normal_bundle = PopBundle::random(0, 0, &mut rng);
        normal_bundle.morale = Morale {
            value: 100.0,
            ..Default::default()
        };
        let normal_pop = world.spawn(normal_bundle).id();

        // Spawn psycho pop
        let mut psycho_bundle = PopBundle::random(0, 0, &mut rng);
        psycho_bundle.traits.add(Trait::Psychopath);
        psycho_bundle.morale = Morale {
            value: 100.0,
            ..Default::default()
        };
        let psycho_pop = world.spawn(psycho_bundle).id();

        // Trigger harvest
        world
            .resource_mut::<Events<OrganHarvestedEvent>>()
            .send(OrganHarvestedEvent);

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_harvesting_horror_system);
        schedule.run(&mut world);

        let normal_morale = world.get::<Morale>(normal_pop).unwrap();
        let psycho_morale = world.get::<Morale>(psycho_pop).unwrap();

        assert!(normal_morale.value < 100.0); // Took the horror penalty
        assert_eq!(psycho_morale.value, 100.0); // Unaffected
    }
}
