use bevy::prelude::*;
use bevy::utils::HashMap;
use crate::layer1::map::GridPosition;

#[derive(Component, Clone)]
pub struct TransitNode {
    pub id: u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RoutePreference {
    Neutral,
    Favorite,
    Disliked,
}

#[derive(Resource, Default)]
pub struct TransitNetwork {
    pub nodes: Vec<Entity>,
    pub preferences: HashMap<(Entity, Entity), RoutePreference>,
    pub traffic_history: HashMap<(Entity, Entity), u32>,
}

impl TransitNetwork {
    pub fn new(nodes: Vec<Entity>) -> Self {
        Self {
            nodes,
            preferences: HashMap::new(),
            traffic_history: HashMap::new(),
        }
    }

    pub fn set_preference(&mut self, start: Entity, end: Entity, pref: RoutePreference) {
        self.preferences.insert((start, end), pref);
    }
}

#[derive(Event)]
pub struct ManualOverrideEvent {
    pub start: Entity,
    pub end: Entity,
}

#[derive(Component)]
pub struct TransitCrash;

pub fn update_transit_sentience_system(mut network: ResMut<TransitNetwork>) {
    let mut updates = Vec::new();
    for (&route, &traffic) in network.traffic_history.iter() {
        if traffic > 1000 {
            updates.push((route, RoutePreference::Disliked));
        } else if traffic < 100 {
            updates.push((route, RoutePreference::Favorite));
        } else {
            updates.push((route, RoutePreference::Neutral));
        }
    }
    for (route, pref) in updates {
        network.preferences.insert(route, pref);
    }
}

pub fn route_pops_with_bias_system(
    network: Res<TransitNetwork>,
    mut pops: Query<(&GridPosition, &mut crate::layer1::execution::components::MovementTarget)>,
    nodes: Query<(Entity, &GridPosition), With<TransitNode>>,
) {
    for (pos, mut target) in pops.iter_mut() {
        let current_target = target.target_entity;

        // Find which transit node the pop is currently at (if any)
        let mut current_node = None;
        for (node_entity, node_pos) in nodes.iter() {
            if pos.x == node_pos.x && pos.y == node_pos.y {
                current_node = Some(node_entity);
                break;
            }
        }

        if let Some(start_node) = current_node {
            if let Some(pref) = network.preferences.get(&(start_node, current_target)) {
                if *pref == RoutePreference::Disliked {
                    if let Some(&alt_node) = network.nodes.iter().find(|&&n| n != current_target && n != start_node) {
                        target.target_entity = alt_node;

                        // Also update the target position to match the new alt_node's position
                        if let Ok((_, alt_pos)) = nodes.get(alt_node) {
                            target.target_position = *alt_pos;
                        }
                    }
                }
            }
        }
    }
}

pub fn handle_manual_override_system(
    mut commands: Commands,
    mut events: EventReader<ManualOverrideEvent>,
    mut network: ResMut<TransitNetwork>,
) {
    for ev in events.read() {
        network.preferences.insert((ev.start, ev.end), RoutePreference::Neutral);
        commands.entity(ev.start).insert(TransitCrash);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::mind::utility_types::{PopAction, ActionType};
    use crate::layer1::pop::Pop;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_transit_network_develops_preferences_over_time() {
        let mut app = App::new();
        app.add_systems(Update, update_transit_sentience_system);

        let node_a = app.world_mut().spawn(TransitNode { id: 1 }).id();
        let node_b = app.world_mut().spawn(TransitNode { id: 2 }).id();

        let mut network = TransitNetwork::new(vec![node_a, node_b]);
        network.traffic_history.insert((node_a, node_b), 1500); // High traffic
        app.world_mut().insert_resource(network);

        // Act: Run simulation for many ticks with high traffic on route A->B
        app.update();

        // Assert: Network should develop a preference (e.g., 'favorite' or 'disliked') for this route
        let network = app.world().resource::<TransitNetwork>();
        assert_eq!(network.preferences.get(&(node_a, node_b)), Some(&RoutePreference::Disliked));
    }

    #[test]
    fn test_sentient_transit_reroutes_pops() {
        let mut app = App::new();
        app.add_systems(Update, route_pops_with_bias_system);

        let start_node = app.world_mut().spawn((TransitNode { id: 1 }, GridPosition { x: 0, y: 0 })).id();
        let end_node = app.world_mut().spawn((TransitNode { id: 2 }, GridPosition { x: 10, y: 10 })).id();
        let alt_node = app.world_mut().spawn((TransitNode { id: 3 }, GridPosition { x: 5, y: 5 })).id();

        let mut network = TransitNetwork::new(vec![start_node, end_node, alt_node]);
        network.set_preference(start_node, end_node, RoutePreference::Disliked);
        app.world_mut().insert_resource(network);

        let pop = app.world_mut().spawn((
            Pop,
            PopAction { current: ActionType::Explore, ..Default::default() },
            GridPosition { x: 0, y: 0 },
            crate::layer1::execution::components::MovementTarget {
                target_entity: end_node,
                target_position: GridPosition { x: 10, y: 10 },
                for_action: ActionType::Explore,
            }
        )).id();

        // Act: Run pathfinding/transit system
        app.update();

        // Assert: Pop's path should be forced through alt_node instead of direct route due to network bias
        let target = app.world().get::<crate::layer1::execution::components::MovementTarget>(pop).unwrap();
        // Just checking that it's NOT the disliked end_node
        assert_ne!(target.target_entity, end_node);
        assert_eq!(target.target_entity, alt_node);
        assert_eq!(target.target_position, GridPosition { x: 5, y: 5 });
    }

    #[test]
    fn test_overriding_sentient_network_causes_crashes() {
        let mut app = App::new();
        app.add_event::<ManualOverrideEvent>();
        app.add_systems(Update, handle_manual_override_system);

        let start_node = app.world_mut().spawn(TransitNode { id: 1 }).id();
        let end_node = app.world_mut().spawn(TransitNode { id: 2 }).id();

        let mut network = TransitNetwork::new(vec![start_node, end_node]);
        network.set_preference(start_node, end_node, RoutePreference::Disliked);
        app.world_mut().insert_resource(network);

        // Act: Player manually forces route override
        app.world_mut().resource_mut::<Events<ManualOverrideEvent>>().send(ManualOverrideEvent {
            start: start_node,
            end: end_node,
        });
        app.update();

        // Assert: Network experiences downtime / maintenance spike
        let network = app.world().resource::<TransitNetwork>();
        assert_eq!(network.preferences.get(&(start_node, end_node)), Some(&RoutePreference::Neutral));

        // Assert: Event creates a crash
        assert!(app.world().get::<TransitCrash>(start_node).is_some());
    }
}
