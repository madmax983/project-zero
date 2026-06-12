use bevy_ecs::prelude::*;

use crate::layer1::economy::inventory::Inventory;
use crate::layer1::economy::items::ItemType;
use crate::layer1::lifecycle::Age;
use crate::layer1::psychology::traits::{Trait, Traits};
use crate::layer1::social::morale::MoodModifier;
use crate::layer1::social::morale::Morale;

#[derive(Component, Default)]
pub struct Hoard {
    pub items: Vec<ItemType>,
}

#[derive(Event)]
pub struct ConfiscateHoardEvent {
    pub target: Entity,
}

pub fn check_for_hoarder_trait_system(mut query: Query<(&Age, &mut Traits)>) {
    for (age, mut traits) in query.iter_mut() {
        if age.ticks_alive >= 60 * 1000 && !traits.has(Trait::Hoarder) {
            traits.add(Trait::Hoarder);
        }
    }
}

pub fn hoarder_collection_system(
    mut hoarders: Query<(&Traits, &mut Hoard)>,
    mut stockpiles: Query<&mut Inventory>,
) {
    for (traits, mut hoard) in hoarders.iter_mut() {
        if traits.has(Trait::Hoarder) {
            for mut inv in stockpiles.iter_mut() {
                if let Some(pos) = inv.items.iter().position(|i| i.item_type == ItemType::Tool) {
                    inv.items.remove(pos);
                    hoard.items.push(ItemType::Tool);
                    break;
                }
            }
        }
    }
}

pub fn apply_hoard_morale_buff_system(mut query: Query<(&Hoard, &mut Morale)>) {
    for (hoard, mut morale) in query.iter_mut() {
        if !hoard.items.is_empty() {
            morale.add_modifier(MoodModifier {
                label: "Personal Hoard".to_string(),
                value: 0.05,
                duration: 1,
            });
        }
    }
}

pub fn process_confiscation_system(
    mut events: EventReader<ConfiscateHoardEvent>,
    mut query: Query<(&mut Hoard, &mut Morale)>,
) {
    for ev in events.read() {
        if let Ok((mut hoard, mut morale)) = query.get_mut(ev.target) {
            if !hoard.items.is_empty() {
                morale.add_modifier(MoodModifier {
                    label: "Confiscated Hoard".to_string(),
                    value: -0.30,
                    duration: 100,
                });
                hoard.items.clear();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer1::economy::inventory::{Inventory, InventoryItem};
    use crate::layer1::economy::items::ItemType;
    use crate::layer1::lifecycle::Age;
    use crate::layer1::psychology::traits::{Trait, Traits};
    use crate::layer1::social::morale::Morale;

    #[test]
    fn test_elder_develops_hoarder_trait() {
        let mut world = World::new();
        let elder = world.spawn((Age::new(65), Traits::default())).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(check_for_hoarder_trait_system);
        schedule.run(&mut world);

        let traits = world.get::<Traits>(elder).unwrap();
        assert!(
            traits.has(Trait::Hoarder),
            "Elder should have a chance to develop the Hoarder trait"
        );
    }

    #[test]
    fn test_hoarder_claims_scrap() {
        let mut world = World::new();
        let mut traits = Traits::default();
        traits.add(Trait::Hoarder);
        let hoarder = world
            .spawn((traits, Hoard::default(), Morale::default()))
            .id();

        let mut inventory = Inventory {
            capacity: 10,
            ..Default::default()
        };
        inventory.items.push(InventoryItem {
            item_type: ItemType::Tool,
            entity: None,
        });
        inventory.items.push(InventoryItem {
            item_type: ItemType::Tool,
            entity: None,
        });
        let stockpile = world.spawn(inventory).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(hoarder_collection_system);
        schedule.run(&mut world);

        let hoard = world.get::<Hoard>(hoarder).unwrap();
        let stockpile_inv = world.get::<Inventory>(stockpile).unwrap();

        assert!(
            hoard.items.contains(&ItemType::Tool),
            "Hoarder should have collected tool"
        );
        assert_eq!(
            stockpile_inv.items.len(),
            1,
            "Tool should be removed from stockpile"
        );
    }

    #[test]
    fn test_hoard_grants_morale_buff() {
        let mut world = World::new();
        let mut hoard = Hoard::default();
        hoard.items.push(ItemType::Tool);

        let mut traits = Traits::default();
        traits.add(Trait::Hoarder);
        let hoarder = world.spawn((traits, hoard, Morale::default())).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_hoard_morale_buff_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(hoarder).unwrap();
        assert!(
            !morale.modifiers.is_empty(),
            "Hoarder should receive a morale buff from their hoard"
        );
    }

    #[test]
    fn test_confiscate_hoard_lowers_morale() {
        let mut world = World::new();
        world.init_resource::<Events<ConfiscateHoardEvent>>();

        let mut hoard = Hoard::default();
        hoard.items.push(ItemType::Tool);

        let mut traits = Traits::default();
        traits.add(Trait::Hoarder);
        let hoarder = world.spawn((traits, hoard, Morale::default())).id();

        world.send_event(ConfiscateHoardEvent { target: hoarder });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_confiscation_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(hoarder).unwrap();
        assert!(
            morale.modifiers.iter().any(|m| m.value < 0.0),
            "Confiscating the hoard should heavily penalize morale"
        );
        let empty_hoard = world.get::<Hoard>(hoarder).unwrap();
        assert!(
            empty_hoard.items.is_empty(),
            "Hoard should be empty after confiscation"
        );
    }
}
