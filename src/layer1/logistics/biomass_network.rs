use crate::layer1::economy::resources::ResourceType;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct BiomassNetwork {
    pub hunger: f32,
    pub max_hunger: f32,
    pub consumption_rate: f32,
}

#[derive(Component)]
pub struct InTransit {
    pub network: Entity,
}

#[derive(Component)]
pub struct ResourceYield {
    pub amount: f32,
    pub resource_type: ResourceType,
}

pub fn process_biomass_network_hunger(mut query: Query<&mut BiomassNetwork>) {
    for mut network in query.iter_mut() {
        network.hunger = (network.hunger + network.consumption_rate).min(network.max_hunger);
    }
}

pub fn digest_transit_contents(
    mut commands: Commands,
    mut networks: Query<&mut BiomassNetwork>,
    transit_query: Query<(Entity, &InTransit, Option<&crate::layer1::pop::PopName>)>,
    mut pop_died_events: EventWriter<crate::layer1::pop::PopDied>,
    time: Option<Res<crate::shared::time::SimulationTime>>,
) {
    for (entity, transit, pop_name) in transit_query.iter() {
        if let Ok(mut network) = networks.get_mut(transit.network) {
            if network.hunger >= network.max_hunger {
                // Digest!
                if let Some(name) = pop_name {
                    pop_died_events.send(crate::layer1::pop::PopDied {
                        entity,
                        name: name.0.clone(),
                        reason: "digested by starving biomass network".to_string(),
                        tick: time.as_ref().map_or(0, |t| t.tick),
                    });
                }
                commands.entity(entity).despawn();
                network.hunger = (network.hunger - 20.0).max(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::{Pop, PopName};
    use bevy_app::App;

    #[test]
    fn test_biomass_network_consumes_upkeep() {
        let mut app = App::new();
        app.add_systems(bevy_app::Update, process_biomass_network_hunger);

        // Add a biomass network entity with a hunger tracker
        let network_entity = app
            .world_mut()
            .spawn((BiomassNetwork {
                hunger: 50.0,
                max_hunger: 100.0,
                consumption_rate: 10.0,
            },))
            .id();

        app.update();

        let network = app.world().get::<BiomassNetwork>(network_entity).unwrap();
        assert_eq!(network.hunger, 60.0); // Hunger increased by consumption rate
    }

    #[test]
    fn test_starving_network_digests_contents() {
        let mut app = App::new();
        app.add_systems(
            bevy_app::Update,
            (process_biomass_network_hunger, digest_transit_contents),
        );

        let network_entity = app
            .world_mut()
            .spawn((BiomassNetwork {
                hunger: 100.0, // Fully starving
                max_hunger: 100.0,
                consumption_rate: 10.0,
            },))
            .id();

        // Add an item in transit
        let item_entity = app
            .world_mut()
            .spawn((
                InTransit {
                    network: network_entity,
                },
                ResourceYield {
                    amount: 20.0,
                    resource_type: ResourceType::Food,
                },
            ))
            .id();

        app.update();

        // The item should be consumed/digested, meaning it despawns or changes state
        assert!(
            app.world().get_entity(item_entity).is_err(),
            "Item should be digested and despawned"
        );

        // Network hunger should decrease due to digestion
        let network = app.world().get::<BiomassNetwork>(network_entity).unwrap();
        assert!(
            network.hunger < 100.0,
            "Hunger should be reduced after digesting contents"
        );
    }

    #[test]
    fn test_fed_network_safely_transports() {
        let mut app = App::new();
        app.add_systems(
            bevy_app::Update,
            (process_biomass_network_hunger, digest_transit_contents),
        );

        let network_entity = app
            .world_mut()
            .spawn((BiomassNetwork {
                hunger: 0.0, // Sated
                max_hunger: 100.0,
                consumption_rate: 10.0,
            },))
            .id();

        // Add a pop in transit
        let pop_entity = app
            .world_mut()
            .spawn((
                InTransit {
                    network: network_entity,
                },
                Pop,
                PopName("Commuter".to_string()),
            ))
            .id();

        app.update();

        // The pop should survive because the network is not starving
        assert!(app.world().get_entity(pop_entity).is_ok());
    }
}
