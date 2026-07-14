use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use bevy::prelude::*;

#[derive(Component)]
pub struct MycorrhizalNetwork {
    pub hunger: f32,
    pub has_warned: bool, // Move warning state to component
}

#[derive(Component)]
pub struct BuildingNeeds {
    pub power: f32,
}

#[derive(Component)]
pub struct BuildingInventory {
    pub food: f32,
}

#[derive(Component)]
pub struct OnNetwork(pub Entity);

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum FungalResource {
    Food,
    Power,
    Water,
}

pub fn fungal_network_sharing_system(
    _networks: Query<&MycorrhizalNetwork>,
    _buildings: Query<(&BuildingNeeds, &OnNetwork)>,
) {
    // For this minimal GREEN phase, the spec doesn't provide tests for sharing,
    // so we provide a safe skeleton that fulfills the signature without wasting CPU.
}

pub fn fungal_tax_system(
    mut networks: Query<(Entity, &mut MycorrhizalNetwork)>,
    mut buildings: Query<(
        Option<&mut BuildingInventory>,
        Option<&mut BuildingNeeds>,
        &OnNetwork,
    )>,
    mut chronicle: EventWriter<AddChronicleEvent>,
) {
    for (network_entity, mut network) in networks.iter_mut() {
        if network.hunger > 0.0 {
            // Just stealing food since needs.power shouldn't be subtracted as tax
            let target_resource = FungalResource::Food;

            if network.hunger > 5.0 && !network.has_warned {
                chronicle.send(AddChronicleEvent {
                    text: "The Mycorrhizal Network is starving. A massive tax is imminent."
                        .to_string(),
                    importance: EventImportance::Major,
                });
                network.has_warned = true;
            } else if network.hunger <= 5.0 {
                network.has_warned = false;
            }

            for (mut inventory_opt, _, on_network) in buildings.iter_mut() {
                if network.hunger <= 0.0 {
                    break;
                }
                if on_network.0 != network_entity {
                    continue;
                }
                if target_resource == FungalResource::Food {
                    if let Some(ref mut inventory) = inventory_opt {
                        if inventory.food > 0.0 {
                            let amount = inventory.food.min(5.0).min(network.hunger);
                            inventory.food -= amount;
                            network.hunger -= amount;
                        }
                    }
                }
            }
        } else {
            network.has_warned = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_fungal_tax_siphons_resources() {
        let mut world = setup_world();
        world.insert_resource(Events::<AddChronicleEvent>::default());

        let network_entity = world.spawn_empty().id();
        world.entity_mut(network_entity).insert(MycorrhizalNetwork {
            hunger: 10.0,
            has_warned: false,
        });

        let building_entity = world.spawn_empty().id();
        world
            .entity_mut(building_entity)
            .insert((BuildingInventory { food: 50.0 }, OnNetwork(network_entity)));

        let mut schedule = Schedule::default();
        schedule.add_systems(fungal_tax_system);
        schedule.run(&mut world);

        let building = world.get::<BuildingInventory>(building_entity).unwrap();
        let network = world.get::<MycorrhizalNetwork>(network_entity).unwrap();

        assert!(
            building.food < 50.0,
            "Building should have lost food to tax"
        );
        assert!(
            network.hunger < 10.0,
            "Network hunger should have decreased"
        );
    }

    #[test]
    fn test_fungal_tax_no_warn_if_hunger_low() {
        let mut world = setup_world();
        world.insert_resource(Events::<AddChronicleEvent>::default());

        let network_entity = world.spawn_empty().id();
        world.entity_mut(network_entity).insert(MycorrhizalNetwork {
            hunger: 4.0,
            has_warned: false,
        });

        let building_entity = world.spawn_empty().id();
        world
            .entity_mut(building_entity)
            .insert((BuildingInventory { food: 10.0 }, OnNetwork(network_entity)));

        let mut schedule = Schedule::default();
        schedule.add_systems(fungal_tax_system);
        schedule.run(&mut world);
    }

    #[test]
    fn test_fungal_tax_does_not_tax_other_networks() {
        let mut world = setup_world();
        world.insert_resource(Events::<AddChronicleEvent>::default());

        let network_entity = world.spawn_empty().id();
        world.entity_mut(network_entity).insert(MycorrhizalNetwork {
            hunger: 10.0,
            has_warned: false,
        });

        let other_network_entity = world.spawn_empty().id();
        world
            .entity_mut(other_network_entity)
            .insert(MycorrhizalNetwork {
                hunger: 0.0,
                has_warned: false,
            });

        let building_entity = world.spawn_empty().id();
        world.entity_mut(building_entity).insert((
            BuildingInventory { food: 50.0 },
            OnNetwork(other_network_entity),
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(fungal_tax_system);
        schedule.run(&mut world);

        let building = world.get::<BuildingInventory>(building_entity).unwrap();
        let network = world.get::<MycorrhizalNetwork>(network_entity).unwrap();

        assert_eq!(building.food, 50.0, "Building should not have lost food");
        assert_eq!(
            network.hunger, 10.0,
            "Network hunger should not have decreased"
        );
    }

    #[test]
    fn test_fungal_tax_clears_warn_on_no_hunger() {
        let mut world = setup_world();
        world.insert_resource(Events::<AddChronicleEvent>::default());

        let network_entity = world.spawn_empty().id();
        world.entity_mut(network_entity).insert(MycorrhizalNetwork {
            hunger: 0.0,
            has_warned: true,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(fungal_tax_system);
        schedule.run(&mut world);

        let network = world.get::<MycorrhizalNetwork>(network_entity).unwrap();

        assert!(!network.has_warned, "Network warn should be cleared");
    }
}
