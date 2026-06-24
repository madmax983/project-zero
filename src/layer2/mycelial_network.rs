use crate::layer1::nature::ecology::{EcologicalDamage, HarvestEvent};
use crate::layer2::fleet::{Fleet, FleetOrder};
use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct SpaceFauna {
    pub strength: i32,
}

#[derive(Component, Debug, Clone)]
pub struct BioLink {
    pub source: Entity,
    pub target: Entity,
}

pub fn evaluate_ecological_damage_system(
    mut commands: Commands,
    mut events: EventReader<HarvestEvent>,
    mut damage_query: Query<&mut EcologicalDamage>,
    link_query: Query<&BioLink>,
) {
    for event in events.read() {
        if let Ok(mut damage) = damage_query.get_mut(event.node) {
            damage.value += event.amount;

            if damage.value >= damage.threshold {
                // Trigger immune response
                // Find a connected node to spawn from
                let mut spawn_node = None;
                for link in link_query.iter() {
                    if link.source == event.node {
                        spawn_node = Some(link.target);
                        break;
                    }
                }

                if let Some(_spawner) = spawn_node {
                    // Spawn Fauna
                    commands.spawn((
                        Fleet,
                        SpaceFauna { strength: 100 }, // MVP strength
                        FleetOrder::MoveTo(event.node),
                    ));

                    // Reset threshold
                    damage.value = 0.0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer3::physics::relativity::SystemNode;

    #[test]
    fn test_harvesting_triggers_immune_response_at_connected_node() {
        let mut app = App::new();
        app.add_event::<HarvestEvent>();
        app.add_systems(Update, evaluate_ecological_damage_system);

        // Setup Nodes
        let node_a = app
            .world_mut()
            .spawn((
                SystemNode,
                EcologicalDamage {
                    value: 0.0,
                    threshold: 100.0,
                },
            ))
            .id();

        let node_b = app.world_mut().spawn((SystemNode,)).id();

        // Connect them with a BioLink
        app.world_mut().spawn(BioLink {
            source: node_a,
            target: node_b,
        });
        app.world_mut().spawn(BioLink {
            source: node_b,
            target: node_a,
        });

        // Trigger massive harvest on Node A
        app.world_mut()
            .resource_mut::<Events<HarvestEvent>>()
            .send(HarvestEvent {
                node: node_a,
                amount: 150.0, // Exceeds threshold
            });

        app.update();

        // Verify Space Fauna spawned at Node B targeting Node A
        let mut found_fauna = false;
        let mut query = app.world_mut().query::<(&SpaceFauna, &FleetOrder)>();
        for (_fauna, command) in query.iter(app.world()) {
            if let FleetOrder::MoveTo(destination) = command {
                if *destination == node_a {
                    found_fauna = true;
                    break;
                }
            }
        }

        assert!(found_fauna, "Massive harvesting at Node A should spawn immune response Fauna at connected Node B that targets Node A.");

        let damage_a = app.world().get::<EcologicalDamage>(node_a).unwrap();
        assert_eq!(
            damage_a.value, 0.0,
            "Ecological damage should reset after triggering an immune response."
        );
    }
}
