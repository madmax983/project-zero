use crate::layer1::resources::ColonyResources;
use crate::layer2::syzygy::PlanetaryGravity;
use bevy_ecs::prelude::*;

/// A single item of cargo in a trade manifest.
#[derive(Component, Clone)]
pub struct CargoItem {
    /// The mass of the cargo item in tons.
    pub mass: f32,
    /// The economic value of the cargo item in credits.
    pub value: f32,
}

/// A manifest of cargo items to be traded or launched.
#[derive(Component)]
pub struct TradeManifest {
    /// The list of cargo items in the manifest.
    pub items: Vec<CargoItem>,
}

/// Event triggered when a ship is ready to launch.
#[derive(Event)]
pub struct LaunchShipEvent {
    /// The entity representing the ship's trade manifest.
    pub manifest_entity: Entity,
}

/// Calculates the fuel cost to launch a manifest from the planet.
pub fn calculate_launch_cost(gravity: &PlanetaryGravity, manifest: &TradeManifest) -> f32 {
    let total_mass: f32 = manifest.items.iter().map(|i| i.mass).sum();
    // Base cost 100.0 fuel units + 10.0 fuel units per ton per G
    100.0 + (total_mass * gravity.current * 10.0)
}

/// Processes launch events, deducting fuel and despawning the ship if successful.
pub fn process_launch_system(
    mut commands: Commands,
    gravity: Res<PlanetaryGravity>,
    mut events: EventReader<LaunchShipEvent>,
    query: Query<(&TradeManifest, Option<&crate::layer1::culture::celestial_cemeteries::LaunchSequence>)>,
    mut resources: ResMut<ColonyResources>,
) {
    for event in events.read() {
        if let Ok((manifest, launch_seq)) = query.get(event.manifest_entity) {
            let cost = calculate_launch_cost(&gravity, manifest);
            if resources.fuel >= cost {
                resources.fuel -= cost;

                if let Some(seq) = launch_seq {
                    // Fail the launch if pseudo-random check fails
                    let failure_chance = seq.base_risk;
                    if rand::random::<f32>() < failure_chance {
                        // Launch fails! Despawn it as destroyed.
                        commands.entity(event.manifest_entity).despawn();
                        continue;
                    }
                }

                // Despawn the ship after a successful launch
                commands.entity(event.manifest_entity).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_gravity_increases_launch_cost_significantly() {
        let gravity_normal = PlanetaryGravity {
            current: 1.0,
            base: 1.0,
        };
        let gravity_high = PlanetaryGravity {
            current: 2.5,
            base: 2.5,
        };

        let manifest = TradeManifest {
            items: vec![
                CargoItem {
                    mass: 50.0,
                    value: 500.0,
                }, // 50 tons of iron ore
            ],
        };

        let cost_normal = calculate_launch_cost(&gravity_normal, &manifest);
        let cost_high = calculate_launch_cost(&gravity_high, &manifest);

        assert!(
            cost_high > cost_normal * 2.0,
            "High gravity should vastly increase launch costs"
        );
        assert_eq!(cost_normal, 600.0); // 100 + (50 * 1 * 10)
        assert_eq!(cost_high, 1350.0); // 100 + (50 * 2.5 * 10)
    }

    #[test]
    fn test_low_mass_high_value_goods_are_profitable_on_high_g() {
        let gravity_high = PlanetaryGravity {
            current: 2.5,
            base: 2.5,
        };

        let raw_ore = TradeManifest {
            items: vec![CargoItem {
                mass: 100.0,
                value: 1000.0,
            }],
        };

        let refined_chips = TradeManifest {
            items: vec![CargoItem {
                mass: 5.0,
                value: 5000.0,
            }],
        };

        let cost_ore = calculate_launch_cost(&gravity_high, &raw_ore);
        let cost_chips = calculate_launch_cost(&gravity_high, &refined_chips);

        let profit_ore = raw_ore.items[0].value - cost_ore;
        let profit_chips = refined_chips.items[0].value - cost_chips;

        assert!(
            profit_ore < 0.0,
            "Exporting heavy raw ore on High G should be unprofitable"
        );
        assert!(
            profit_chips > 0.0,
            "Exporting light refined tech on High G should be profitable"
        );
    }

    #[test]
    fn test_planetary_gravity_default() {
        let default_gravity = PlanetaryGravity::default();
        assert_eq!(default_gravity.current, 1.0);
    }

    #[test]
    fn test_process_launch_system() {
        let mut app = bevy_app::App::new();
        app.add_event::<LaunchShipEvent>();
        app.world_mut().insert_resource(PlanetaryGravity::default());
        app.world_mut().insert_resource(ColonyResources {
            fuel: 2000.0,
            ..Default::default()
        });

        let entity = app
            .world_mut()
            .spawn(TradeManifest {
                items: vec![CargoItem {
                    mass: 50.0,
                    value: 500.0,
                }],
            })
            .id();

        app.world_mut()
            .resource_mut::<Events<LaunchShipEvent>>()
            .send(LaunchShipEvent {
                manifest_entity: entity,
            });
        app.add_systems(bevy_app::Update, process_launch_system);
        app.update();

        let post_res = app.world().resource::<ColonyResources>();
        // Cost is 100 + 50 * 1.0 * 10 = 600. So 2000 - 600 = 1400.
        assert_eq!(post_res.fuel, 1400.0);

        // Ensure entity was despawned
        assert!(app.world().get_entity(entity).is_err());
    }

    #[test]
    fn test_process_launch_system_insufficient_fuel() {
        let mut app = bevy_app::App::new();
        app.add_event::<LaunchShipEvent>();
        app.world_mut().insert_resource(PlanetaryGravity::default());
        app.world_mut().insert_resource(ColonyResources {
            fuel: 100.0,
            ..Default::default()
        });

        let entity = app
            .world_mut()
            .spawn(TradeManifest {
                items: vec![CargoItem {
                    mass: 50.0,
                    value: 500.0,
                }],
            })
            .id();

        app.world_mut()
            .resource_mut::<Events<LaunchShipEvent>>()
            .send(LaunchShipEvent {
                manifest_entity: entity,
            });
        app.add_systems(bevy_app::Update, process_launch_system);
        app.update();

        let post_res = app.world().resource::<ColonyResources>();
        assert_eq!(post_res.fuel, 100.0);

        // Ensure entity was not despawned
        assert!(app.world().get_entity(entity).is_ok());
    }
}
