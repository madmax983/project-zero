//! Upcycled Art (Nova Feature).
//!
//! # The Spark
//! We have the `Trait::Artistic`, the `ClutterGrid`, and `GraffitiType::Mural`.
//!
//! # The Feature
//! Pops with `Trait::Artistic` who have high `leisure` (e.g., > 0.8) feel inspired.
//! If they are on a tile with high `Clutter` (e.g., > 5.0), they have a chance to convert
//! the trash into a beautiful `Mural`. They consume the clutter (reducing it to 0) and
//! place a `GraffitiType::Mural` on the tile.
//!
//! # The Potential
//! Connects the personality trait (Artistic) with the maintenance system (Clutter) and the
//! beauty system (Graffiti). It turns a negative environmental factor (trash) into a
//! resource for autonomous colony beautification.

use crate::layer1::clutter::ClutterGrid;
use crate::layer1::graffiti::{Graffiti, GraffitiMap, GraffitiType};
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;
use rand::Rng;

pub fn upcycled_art_system(
    mut clutter: ResMut<ClutterGrid>,
    mut graffiti_map: ResMut<GraffitiMap>,
    mut pops: Query<(&GridPosition, &Traits, &mut Needs), With<Pop>>,
) {
    let mut rng = rand::thread_rng();

    for (pos, traits, mut needs_ptr) in pops.iter_mut() {
        if traits.has(Trait::Artistic) && needs_ptr.leisure > 0.8 {
            let x = pos.x as usize;
            let y = pos.y as usize;

            let current_clutter = clutter.get(x, y);

            if current_clutter > 5.0 && rng.gen_bool(0.10) {
                let pos_tuple = (pos.x, pos.y);
                if !graffiti_map.markings.contains_key(&pos_tuple) {
                    // Create art from trash
                    clutter.set(x, y, 0.0);

                    graffiti_map.markings.insert(
                        pos_tuple,
                        Graffiti {
                            graffiti_type: GraffitiType::Mural,
                            decay: 500.0, // Long-lasting
                            modifier: 0.2, // Very beautiful
                        },
                    );

                    // Drains a bit of rest due to creative exhaustion
                    needs_ptr.rest = (needs_ptr.rest - 0.1).max(0.0);
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(upcycled_art_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_upcycled_art_system() {
        let mut world = World::new();
        world.insert_resource(GraffitiMap {
            markings: bevy::utils::HashMap::default(),
        });

        let mut clutter = ClutterGrid::new(20, 20);
        clutter.set(5, 5, 10.0); // High clutter
        world.insert_resource(clutter);

        let mut traits = Traits(bevy::utils::HashSet::default());
        traits.add(Trait::Artistic);

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                traits,
                Needs {
                    leisure: 0.9,
                    rest: 1.0,
                    ..Default::default()
                },
            ))
            .id();

        // Run the system enough times to pass the 10% probability
        let mut mural_placed = false;
        for _ in 0..100 {
            world.run_system_once(upcycled_art_system).unwrap();
            let map = world.resource::<GraffitiMap>();
            if map.markings.contains_key(&(5, 5)) {
                mural_placed = true;
                break;
            }
        }

        assert!(
            mural_placed,
            "Artistic pop should have converted clutter into a mural"
        );

        let map = world.resource::<GraffitiMap>();
        let graffiti = map.markings.get(&(5, 5)).unwrap();
        assert_eq!(graffiti.graffiti_type, GraffitiType::Mural);
        assert_eq!(graffiti.modifier, 0.2);

        let clutter_res = world.resource::<ClutterGrid>();
        assert_eq!(
            clutter_res.get(5, 5),
            0.0,
            "Clutter should have been consumed"
        );

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            needs.rest < 1.0,
            "Pop should have lost some rest due to creative exertion"
        );
    }
}
