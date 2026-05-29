//! Protest Graffiti (Nova Feature).
//!
//! # The Spark
//! We have the `Protest` action (`ActionType::Protest`) and the `GraffitiMap` from `graffiti.rs`.
//!
//! # The Feature
//! Pops participating in a `Protest` passively leave `GraffitiType::Propaganda` on their current tile
//! as they protest, spreading their message and influencing the colony's morale.
//!
//! # The Potential
//! Connects the protest system with the beauty/vandalism system (Graffiti).
//! Protesters don't just stand around; they actively spread propaganda that affects the morale
//! of other pops who walk by it.

use crate::layer1::graffiti::{Graffiti, GraffitiMap, GraffitiType};
use crate::layer1::map::GridPosition;
use crate::layer1::utility_types::{ActionType, PopAction};
use bevy_ecs::prelude::*;
use rand::Rng;

/// System that causes Protesting pops to randomly leave Propaganda graffiti on their tile.
pub fn protest_graffiti_system(
    mut graffiti_map: ResMut<GraffitiMap>,
    pops: Query<(&GridPosition, &PopAction)>,
) {
    let mut rng = rand::thread_rng();

    for (pos, action) in pops.iter() {
        if action.current == ActionType::Protest {
            // Give a 5% chance per tick to drop propaganda
            if rng.gen_bool(0.05) {
                // Ensure there's not already graffiti here
                let pos_tuple = (pos.x, pos.y);
                if !graffiti_map.markings.contains_key(&pos_tuple) {
                    graffiti_map.markings.insert(
                        pos_tuple,
                        Graffiti {
                            graffiti_type: GraffitiType::Propaganda,
                            decay: 300.0,  // Long decay to represent strong messaging
                            modifier: 0.1, // Positive modifier for those agreeing with propaganda (or negative depending on interpretation, here using positive as "inspiring" for the cause)
                        },
                    );
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(protest_graffiti_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_protest_graffiti_system() {
        let mut world = World::new();
        world.insert_resource(GraffitiMap::default());

        let pop = world
            .spawn((
                GridPosition { x: 5, y: 5 },
                PopAction {
                    current: ActionType::Protest,
                    ..Default::default()
                },
            ))
            .id();

        // Run the system many times to overcome RNG
        let mut placed = false;
        for _ in 0..1000 {
            world.run_system_once(protest_graffiti_system).unwrap();

            let map = world.resource::<GraffitiMap>();
            if map.markings.contains_key(&(5, 5)) {
                placed = true;
                break;
            }
        }

        assert!(placed, "Protesting pop should eventually leave graffiti");

        let map = world.resource::<GraffitiMap>();
        let graffiti = map.markings.get(&(5, 5)).unwrap();
        assert_eq!(graffiti.graffiti_type, GraffitiType::Propaganda);
        assert_eq!(graffiti.modifier, 0.1);

        // Ensure another pop not protesting doesn't leave graffiti
        world.get_mut::<PopAction>(pop).unwrap().current = ActionType::Idle;
        world.get_mut::<GridPosition>(pop).unwrap().x = 10;

        for _ in 0..1000 {
            world.run_system_once(protest_graffiti_system).unwrap();
        }

        let map = world.resource::<GraffitiMap>();
        assert!(
            !map.markings.contains_key(&(10, 5)),
            "Idle pop should not leave graffiti"
        );
    }
}
