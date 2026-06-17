// src/layer2/blind_jump.rs

use crate::layer2::fleet::{FleetHealth, InOrbit};
use crate::layer2::system::SystemBody;
use bevy::prelude::*;
use rand::Rng;

/// Component indicating a fleet intends to perform an emergency blind jump.
#[derive(Component)]
pub struct BlindJumpAction;

/// System to process blind jump actions.
pub fn blind_jump_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FleetHealth, Option<&InOrbit>), With<BlindJumpAction>>,
    system_bodies: Query<Entity, With<SystemBody>>,
) {
    let mut rng = rand::thread_rng();

    for (entity, mut health, in_orbit) in query.iter_mut() {
        let possible_destinations: Vec<Entity> = system_bodies.iter().collect();
        if possible_destinations.is_empty() {
            continue;
        }

        let mut dest = possible_destinations[rng.gen_range(0..possible_destinations.len())];

        // Ensure we don't jump to the same place if there are other destinations
        if let Some(current_orbit) = in_orbit {
            if possible_destinations.len() > 1 && dest == current_orbit.parent {
                dest = possible_destinations
                    .into_iter()
                    .find(|&d| d != current_orbit.parent)
                    .unwrap_or(dest);
            }
        }

        commands.entity(entity).remove::<InOrbit>();
        commands.entity(entity).insert(InOrbit { parent: dest });

        // Apply damage
        health.current -= 25.0; // Flat damage for MVP

        commands.entity(entity).remove::<BlindJumpAction>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::fleet::{Fleet, FleetHealth, InOrbit};
    use crate::layer2::system::SystemBody;

    #[test]
    fn test_blind_jump_changes_location_and_causes_damage() {
        let mut app = App::new();
        app.add_systems(Update, blind_jump_system);

        let initial_node = app.world_mut().spawn(SystemBody).id();
        let destination_node = app.world_mut().spawn(SystemBody).id();

        // Needs a valid destination graph, simplified for test
        // app.world_mut().insert_resource(NodeGraph {
        //     nodes: vec![initial_node, destination_node],
        // });

        let fleet = app
            .world_mut()
            .spawn((
                Fleet,
                InOrbit {
                    parent: initial_node,
                },
                // InCombat,
                BlindJumpAction,
                FleetHealth {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        app.update();

        // Check if moved
        let orbit = app.world().get::<InOrbit>(fleet);
        assert!(
            orbit.is_none() || orbit.unwrap().parent == destination_node,
            "Fleet should be at destination node."
        );

        // Check if damaged
        let health = app.world().get::<FleetHealth>(fleet).unwrap();
        assert!(
            health.current < 100.0,
            "Fleet should take damage from blind jump."
        );

        // Action removed
        assert!(app.world().get::<BlindJumpAction>(fleet).is_none());
    }
}
