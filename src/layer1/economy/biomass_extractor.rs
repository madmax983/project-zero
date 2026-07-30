use crate::layer1::economy::inventory::{Inventory, InventoryItem};
use crate::layer1::entities::pop::Pop;
use crate::layer1::health::Dead;
use crate::layer1::items::ItemType;
use crate::layer1::law::justice::Inmate;
use crate::layer1::social::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct OrganHarvestEvent {
    pub target: Entity,
    pub extractor: Entity,
}

#[derive(Component)]
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
            if let Ok((is_dead, is_arrested)) = targets.get(event.target) {
                let penalty = if is_arrested {
                    -0.60
                } else if is_dead {
                    -0.25
                } else {
                    0.0
                };

                let _ = inv.try_add(InventoryItem {
                    item_type: ItemType::VitalOrgans,
                    entity: None,
                });

                if penalty < 0.0 {
                    for mut morale in pops.iter_mut() {
                        morale.add_modifier(MoodModifier {
                            label: "Horror".to_string(),
                            value: penalty,
                            duration: 1000,
                        });
                    }
                }

                commands.entity(event.target).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<OrganHarvestEvent>>();
        world
    }

    #[test]
    fn test_harvesting_organ_generates_organs_and_causes_colony_stress() {
        let mut world = setup_world();

        let pop_id_1 = world.spawn((Pop, Morale::default())).id();
        let pop_id_2 = world.spawn((Pop, Morale::default())).id();

        let dead_pop_id = world.spawn((Pop, Dead)).id();

        let mut inv = Inventory::default();
        inv.capacity = 10;
        let extractor_id = world
            .spawn((
                BiomassExtractor {
                    processing: Some(dead_pop_id),
                },
                inv,
            ))
            .id();

        world.send_event(OrganHarvestEvent {
            target: dead_pop_id,
            extractor: extractor_id,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_biomass_extraction_system);
        schedule.run(&mut world);

        let extractor_inventory = world.get::<Inventory>(extractor_id).unwrap();
        assert!(extractor_inventory
            .items
            .iter()
            .any(|i| i.item_type == ItemType::VitalOrgans));

        let morale_1 = world.get::<Morale>(pop_id_1).unwrap();
        let morale_2 = world.get::<Morale>(pop_id_2).unwrap();
        assert!(morale_1.modifiers.len() > 0);
        assert!(morale_2.modifiers.len() > 0);
        assert_eq!(morale_1.modifiers[0].value, -0.25);

        assert!(world.get_entity(dead_pop_id).is_err(), "Corpse should be despawned");
    }

    #[test]
    fn test_harvesting_living_prisoner_causes_extreme_stress() {
        let mut world = setup_world();

        let pop_id_1 = world.spawn((Pop, Morale::default())).id();

        let prisoner_id = world.spawn((Pop, Inmate::default())).id();

        let mut inv = Inventory::default();
        inv.capacity = 10;
        let extractor_id = world
            .spawn((
                BiomassExtractor {
                    processing: Some(prisoner_id),
                },
                inv,
            ))
            .id();

        world.send_event(OrganHarvestEvent {
            target: prisoner_id,
            extractor: extractor_id,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_biomass_extraction_system);
        schedule.run(&mut world);

        let morale_1 = world.get::<Morale>(pop_id_1).unwrap();
        assert!(morale_1.modifiers.len() > 0);
        assert_eq!(morale_1.modifiers[0].value, -0.6);

        assert!(world.get_entity(prisoner_id).is_err(), "Prisoner should be despawned");
    }
}
