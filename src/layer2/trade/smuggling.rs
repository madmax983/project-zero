use crate::layer1::economy::black_market::ColonyStats;
use crate::layer1::economy::items::ItemType;
use crate::layer1::psychology::needs::Needs;
use crate::layer2::moon_hermits::AsteroidNode;
use bevy_app::{App, Plugin, Update};
use bevy_ecs::prelude::*;

pub struct SmugglerEcosystemPlugin;

impl Plugin for SmugglerEcosystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_smuggler_hubs, process_smuggler_trade));
    }
}

#[derive(Component)]
pub struct SmugglerHub {
    pub active: bool,
    pub target_colony: Entity, // Can just track the entity for context
    pub supplied_item: ItemType,
}

#[derive(Component)]
pub struct Unmonitored;

#[derive(Component)]
pub struct EmbargoPolicy {
    pub embargoes: Vec<ItemType>,
}

#[allow(clippy::type_complexity)]
fn spawn_smuggler_hubs(
    mut commands: Commands,
    colonies: Query<(Entity, &Needs, &EmbargoPolicy)>,
    unmonitored_nodes: Query<Entity, (With<AsteroidNode>, With<Unmonitored>, Without<SmugglerHub>)>,
) {
    let mut available_nodes: Vec<Entity> = unmonitored_nodes.iter().collect();

    for (colony_ent, needs, policy) in colonies.iter() {
        for embargoed_item in &policy.embargoes {
            // Treat leisure < 0.2 as an unmet luxury need for contraband like VoidAle
            let is_unmet =
                if *embargoed_item == ItemType::VoidAle || *embargoed_item == ItemType::Alcohol {
                    needs.leisure < 0.2
                } else {
                    needs.hunger < 0.2
                };

            if is_unmet {
                if let Some(node_ent) = available_nodes.pop() {
                    commands.entity(node_ent).insert(SmugglerHub {
                        active: true,
                        target_colony: colony_ent,
                        supplied_item: *embargoed_item,
                    });
                }
            }
        }
    }
}

fn process_smuggler_trade(
    mut commands: Commands,
    hubs: Query<(Entity, &SmugglerHub)>,
    mut colonies: Query<&mut Needs>,
    mut stats: ResMut<ColonyStats>,
) {
    for (entity, hub) in hubs.iter() {
        if hub.active {
            if let Ok(mut needs) = colonies.get_mut(hub.target_colony) {
                // Fulfill need
                if hub.supplied_item == ItemType::VoidAle || hub.supplied_item == ItemType::Alcohol
                {
                    needs.leisure = 1.0;
                } else {
                    needs.hunger = 1.0;
                }

                // Increase corruption (simulates crime going up)
                stats.corruption += 0.1;

                // Deactivate the hub to prevent infinite looping
                commands.entity(entity).remove::<SmugglerHub>();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::black_market::ColonyStats;
    use crate::layer1::economy::items::ItemType;
    use crate::layer1::psychology::needs::Needs;
    use crate::layer2::moon_hermits::AsteroidNode;
    use bevy_app::App;

    #[test]
    fn test_smuggler_hub_spawns_on_unmet_embargoed_need() {
        let mut app = App::new();
        app.add_plugins(SmugglerEcosystemPlugin);

        app.world_mut().insert_resource(ColonyStats::default());

        // Arrange: Create a colony with an unmet need and an embargo on the item
        // Note: For testing, we use leisure to represent a luxury need
        let _colony_entity = app
            .world_mut()
            .spawn((
                Needs {
                    leisure: 0.0, // Unmet need
                    ..Default::default()
                },
                EmbargoPolicy {
                    embargoes: vec![ItemType::VoidAle], // Item mapped to luxury
                },
            ))
            .id();

        // Create an unmonitored asteroid belt
        let asteroid_node = app.world_mut().spawn((AsteroidNode, Unmonitored)).id();

        // Act: Run the simulation to trigger smuggler hub formation
        app.update();

        // Assert: A Smuggler Hub should spawn on the unmonitored node
        let has_hub = app.world().get::<SmugglerHub>(asteroid_node).is_some();
        assert!(
            has_hub,
            "Smuggler Hub should spawn when embargoed goods are demanded"
        );
    }

    #[test]
    fn test_smugglers_fulfill_needs_and_increase_crime() {
        let mut app = App::new();
        app.add_plugins(SmugglerEcosystemPlugin);

        app.world_mut().insert_resource(ColonyStats {
            corruption: 0.0,
            unmet_luxury: 0,
        });

        let colony_entity = app
            .world_mut()
            .spawn((
                Needs {
                    leisure: 0.0,
                    ..Default::default()
                },
                EmbargoPolicy {
                    embargoes: vec![ItemType::VoidAle],
                },
            ))
            .id();

        let _hub_entity = app
            .world_mut()
            .spawn((
                AsteroidNode,
                SmugglerHub {
                    active: true,
                    target_colony: colony_entity,
                    supplied_item: ItemType::VoidAle,
                },
            ))
            .id();

        // Act: Smugglers supply goods
        app.update();

        // Assert: Needs are met, but corruption goes up
        let colony_needs = app.world().get::<Needs>(colony_entity).unwrap();
        assert!(
            colony_needs.leisure > 0.0,
            "Smugglers should fulfill the embargoed need"
        );

        let stats = app.world().resource::<ColonyStats>();
        assert!(
            stats.corruption > 0.0,
            "Smuggler activity should increase colony corruption"
        );
    }
}
