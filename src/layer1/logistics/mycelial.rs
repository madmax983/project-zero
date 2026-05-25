use crate::layer1::map::GridPosition;
use crate::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
use bevy_ecs::prelude::*;
use std::collections::HashSet;

#[derive(Component, Debug, Clone)]
pub struct MycelialNode {
    pub is_contaminated: bool,
}

#[derive(Resource, Default)]
pub struct MycelialNetwork {
    pub nodes: HashSet<GridPosition>,
}

#[derive(Event, Debug, Clone)]
pub struct ContaminationEvent {
    pub source: GridPosition,
}

#[allow(clippy::type_complexity, clippy::cast_sign_loss)]
pub fn mycelial_transport_system(
    network: Res<MycelialNetwork>,
    mut resources: ResMut<ColonyResources>,
    items: Query<(Entity, &GridPosition, &ResourceItem)>,
    mut commands: Commands,
) {
    if network.nodes.is_empty() {
        return;
    }

    let mut consumed_items = Vec::new();
    for (entity, pos, item) in &items {
        if network.nodes.contains(pos) {
            consumed_items.push((entity, item.resource_type, item.amount));
        }
    }

    for (entity, res_type, amount) in consumed_items {
        resources.add_resource(&res_type, amount);
        commands.entity(entity).despawn();
    }
}

pub fn mycelial_upkeep_system(
    network: Res<MycelialNetwork>,
    mut resources: ResMut<ColonyResources>,
) {
    if network.nodes.is_empty() {
        return;
    }
    let cost = network.nodes.len() as f32 * 0.1;

    if resources.get_amount(ResourceType::Waste) >= cost {
        resources.consume(ResourceType::Waste, cost);
    } else if resources.get_amount(ResourceType::Food) >= cost {
        resources.consume(ResourceType::Food, cost);
    }
}

pub fn mycelial_contamination_system(
    network: Res<MycelialNetwork>,
    mut nodes: Query<(&GridPosition, &mut MycelialNode)>,
    mut events: EventReader<ContaminationEvent>,
) {
    let mut infected = false;
    for event in events.read() {
        if network.nodes.contains(&event.source) {
            infected = true;
            break;
        }
    }

    if infected {
        for (_, mut node) in &mut nodes {
            node.is_contaminated = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_mycelial_network_transports_resources() {
        let mut app = App::new();
        app.init_resource::<ColonyResources>();

        let mut network = MycelialNetwork::default();
        network.nodes.insert(GridPosition { x: 0, y: 0 });
        app.insert_resource(network);

        app.world_mut().spawn((
            GridPosition { x: 0, y: 0 },
            ResourceItem {
                resource_type: ResourceType::Ore,
                amount: 10.0,
            },
        ));

        app.update();
        app.world_mut()
            .run_system_once(mycelial_transport_system)
            .unwrap();

        let resources = app.world().get_resource::<ColonyResources>().unwrap();
        assert_eq!(resources.get_amount(ResourceType::Ore), 10.0);
    }

    #[test]
    fn test_mycelial_network_upkeep() {
        let mut app = App::new();
        let mut resources = ColonyResources {
            max_waste: 100.0,
            ..Default::default()
        };
        resources.add_resource(&ResourceType::Waste, 10.0);
        app.insert_resource(resources);

        let mut network = MycelialNetwork::default();
        network.nodes.insert(GridPosition { x: 0, y: 0 });
        network.nodes.insert(GridPosition { x: 1, y: 0 });
        app.insert_resource(network);

        app.update();
        app.world_mut()
            .run_system_once(mycelial_upkeep_system)
            .unwrap();

        let res = app.world().get_resource::<ColonyResources>().unwrap();
        assert!((res.get_amount(ResourceType::Waste) - 9.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mycelial_network_spreads_disease() {
        let mut app = App::new();
        app.add_event::<ContaminationEvent>();

        let mut network = MycelialNetwork::default();
        network.nodes.insert(GridPosition { x: 0, y: 0 });
        network.nodes.insert(GridPosition { x: 5, y: 5 });
        app.insert_resource(network);

        let node1 = app
            .world_mut()
            .spawn((
                GridPosition { x: 0, y: 0 },
                MycelialNode {
                    is_contaminated: false,
                },
            ))
            .id();
        let node2 = app
            .world_mut()
            .spawn((
                GridPosition { x: 5, y: 5 },
                MycelialNode {
                    is_contaminated: false,
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<ContaminationEvent>>()
            .send(ContaminationEvent {
                source: GridPosition { x: 0, y: 0 },
            });

        app.update();
        app.world_mut()
            .run_system_once(mycelial_contamination_system)
            .unwrap();

        let n1 = app.world().get::<MycelialNode>(node1).unwrap();
        let n2 = app.world().get::<MycelialNode>(node2).unwrap();

        assert!(n1.is_contaminated);
        assert!(n2.is_contaminated);
    }
}
