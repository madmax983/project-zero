use crate::layer1::biology::health::Dead;
use crate::layer1::economy::inventory::Inventory;
use crate::layer1::economy::inventory::InventoryItem;
use crate::layer1::economy::items::ItemType;
use crate::layer1::entities::pop::Pop;
use crate::layer1::law::justice::Inmate;
use crate::layer1::social::morale::Morale;
use bevy_ecs::prelude::*;

#[derive(Event, Debug)]
pub struct OrganHarvestEvent {
    pub target: Entity,
    pub extractor: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct BiomassExtractor {
    pub processing: Option<Entity>,
}

pub fn handle_biomass_extraction_system(
    mut events: EventReader<OrganHarvestEvent>,
    mut extractors: Query<&mut Inventory>,
    mut pops: Query<&mut Morale, With<Pop>>,
    targets: Query<(Has<Dead>, Has<Inmate>)>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok(mut inv) = extractors.get_mut(event.extractor) {
            if let Ok((is_dead, is_inmate)) = targets.get(event.target) {
                // Determine stress penalty
                let penalty = if is_dead {
                    25.0
                } else if is_inmate {
                    60.0
                } else {
                    0.0
                };

                // Add organ to inventory
                let _ = inv.try_add(InventoryItem {
                    item_type: ItemType::VitalOrgans,
                    entity: None,
                });

                // Panic the colony (apply penalty to morale where 1.0 is max morale)
                // Assuming morale is bounded 0.0 to 1.0 based on tests checking if < 0.8
                // We'll scale penalty so 25.0 corresponds to 0.25 morale drop
                let morale_drop = penalty / 100.0;
                for mut morale in pops.iter_mut() {
                    morale.value = (morale.value - morale_drop).max(0.0);
                }

                // Consume the target
                commands.entity(event.target).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::biology::health::Dead;
    use crate::layer1::economy::inventory::Inventory;
    use crate::layer1::economy::items::ItemType;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::law::justice::Inmate;
    use crate::layer1::social::morale::Morale;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_app() -> World {
        let mut world = World::new();
        world.insert_resource(Events::<OrganHarvestEvent>::default());
        world
    }

    #[test]
    fn test_harvesting_organ_generates_organs_and_causes_colony_stress() {
        let mut world = setup_app();

        let pop_id_1 = world
            .spawn((
                Pop,
                Morale {
                    value: 1.0,
                    modifiers: vec![],
                },
            ))
            .id();
        let pop_id_2 = world
            .spawn((
                Pop,
                Morale {
                    value: 1.0,
                    modifiers: vec![],
                },
            ))
            .id();

        let dead_pop_id = world
            .spawn((
                Pop, // Dead flag would go here
                Dead,
            ))
            .id();

        let extractor_id = world
            .spawn((
                BiomassExtractor {
                    processing: Some(dead_pop_id),
                },
                Inventory::default(), // Standard capacity 20
            ))
            .id();

        world
            .resource_mut::<Events<OrganHarvestEvent>>()
            .send(OrganHarvestEvent {
                target: dead_pop_id,
                extractor: extractor_id,
            });

        let _ = world.run_system_once(handle_biomass_extraction_system);

        let extractor_inventory = world.get::<Inventory>(extractor_id).unwrap();
        // The dead pop was processed and an organ was added
        let organ_count = extractor_inventory
            .items
            .iter()
            .filter(|i| i.item_type == ItemType::VitalOrgans)
            .count();
        assert_eq!(organ_count, 1);

        // The living pops are horrified (-25 morale from base 1.0 -> 0.75, which we check is < 0.8)
        let morale_1 = world.get::<Morale>(pop_id_1).unwrap();
        let morale_2 = world.get::<Morale>(pop_id_2).unwrap();
        assert!(morale_1.value < 0.8);
        assert!(morale_2.value < 0.8);

        // The dead pop was consumed
        assert!(world.get_entity(dead_pop_id).is_err() || world.get::<Pop>(dead_pop_id).is_none());
    }

    #[test]
    fn test_harvesting_living_prisoner_causes_extreme_stress() {
        let mut world = setup_app();

        let pop_id_1 = world
            .spawn((
                Pop,
                Morale {
                    value: 1.0,
                    modifiers: vec![],
                },
            ))
            .id();

        let prisoner_id = world
            .spawn((
                Pop,
                Inmate {
                    sentence_ticks: 100,
                }, // Needs Justice System
            ))
            .id();

        let extractor_id = world
            .spawn((
                BiomassExtractor {
                    processing: Some(prisoner_id),
                },
                Inventory::default(),
            ))
            .id();

        world
            .resource_mut::<Events<OrganHarvestEvent>>()
            .send(OrganHarvestEvent {
                target: prisoner_id,
                extractor: extractor_id,
            });

        let _ = world.run_system_once(handle_biomass_extraction_system);

        // Living pop should have even more stress (-60 morale, leaving 0.4)
        let morale_1 = world.get::<Morale>(pop_id_1).unwrap();
        assert!(morale_1.value < 0.5); // Extreme horror penalty
    }
}
