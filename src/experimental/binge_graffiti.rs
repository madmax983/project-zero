//! Binge Graffiti (Nova Feature).
//!
//! # The Spark
//! We have the `Binge` mental break (`ActionType::Binge`) and the `GraffitiMap` from `graffiti.rs`.
//!
//! # The Feature
//! Pops experiencing a `Binge` passively leave `GraffitiType::Vandalism` on their current tile
//! as they move around, simulating throwing food everywhere.
//!
//! # The Potential
//! Connects the mental break (Bingeing) with the beauty/vandalism system (Graffiti).
//! A bingeing pop doesn't just consume food; they actively ruin the aesthetic of the colony
//! by smearing food on the walls, requiring cleaning to restore morale.

use crate::layer1::graffiti::{Graffiti, GraffitiMap, GraffitiType};
use crate::layer1::map::GridPosition;
use crate::layer1::utility_types::{ActionType, PopAction};
use bevy_ecs::prelude::*;
use rand::Rng;

/// System that causes Bingeing pops to randomly leave Vandalism graffiti on their tile.
pub fn binge_graffiti_system(
    mut graffiti_map: ResMut<GraffitiMap>,
    pops: Query<(&GridPosition, &PopAction)>,
) {
    let mut rng = rand::thread_rng();

    for (pos, action) in pops.iter() {
        if action.current == ActionType::Binge {
            // Give a 5% chance per tick to drop food smears
            if rng.gen_bool(0.05) {
                // Ensure there's not already graffiti here
                let pos_tuple = (pos.x, pos.y);
                if !graffiti_map.markings.contains_key(&pos_tuple) {
                    graffiti_map.markings.insert(
                        pos_tuple,
                        Graffiti {
                            graffiti_type: GraffitiType::Vandalism,
                            decay: 200.0,   // Long decay to represent a mess
                            modifier: -0.1, // Disgusting food smear
                        },
                    );
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(binge_graffiti_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_binge_graffiti_system() {
        let mut world = World::new();
        world.insert_resource(GraffitiMap::default());

        let pop = world
            .spawn((
                GridPosition { x: 5, y: 5 },
                PopAction {
                    current: ActionType::Binge,
                    ..Default::default()
                },
            ))
            .id();

        // Run the system many times to overcome RNG
        let mut placed = false;
        for _ in 0..100 {
            world.run_system_once(binge_graffiti_system).unwrap();

            let map = world.resource::<GraffitiMap>();
            if map.markings.contains_key(&(5, 5)) {
                placed = true;
                break;
            }
        }

        assert!(placed, "Bingeing pop should eventually leave graffiti");

        let map = world.resource::<GraffitiMap>();
        let graffiti = map.markings.get(&(5, 5)).unwrap();
        assert_eq!(graffiti.graffiti_type, GraffitiType::Vandalism);
        assert_eq!(graffiti.modifier, -0.1);

        // Ensure another pop not bingeing doesn't leave graffiti
        world.get_mut::<PopAction>(pop).unwrap().current = ActionType::Idle;
        world.get_mut::<GridPosition>(pop).unwrap().x = 10;

        for _ in 0..100 {
            world.run_system_once(binge_graffiti_system).unwrap();
        }

        let map = world.resource::<GraffitiMap>();
        assert!(
            !map.markings.contains_key(&(10, 5)),
            "Idle pop should not leave graffiti"
        );
    }
}
