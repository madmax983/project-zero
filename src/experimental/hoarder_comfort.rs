//! Hoarder's Comfort (Nova Feature).
//!
//! # The Spark
//! We have `ClutterGrid` which normally represents a negative environment (trash/mess).
//! We also have `Trait::Hoarder`.
//!
//! # The Feature
//! Pops with `Trait::Hoarder` passively regenerate `leisure` when standing on a tile with high clutter (> 20.0).
//! They find comfort in being surrounded by "stuff".
//!
//! # The Potential
//! Turns a negative environmental factor (clutter) into a positive for a specific subset of pops.
//! Players might intentionally create "messy zones" to keep their hoarders happy.

use crate::layer1::clutter::ClutterGrid;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

pub fn hoarder_comfort_system(
    clutter_grid: Res<ClutterGrid>,
    mut pops: Query<(&GridPosition, &Traits, &mut Needs), With<Pop>>,
) {
    for (pos, traits, mut needs) in pops.iter_mut() {
        if traits.has(Trait::Hoarder) {
            if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
                let clutter = clutter_grid.get(x, y);
                if clutter > 20.0 {
                    // Passively regenerate leisure
                    needs.leisure = (needs.leisure + 0.05).min(1.0);
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(hoarder_comfort_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_hoarder_comfort_system() {
        let mut world = World::new();

        let mut clutter_grid = ClutterGrid::new(10, 10);
        clutter_grid.set(5, 5, 50.0); // High clutter
        clutter_grid.set(2, 2, 0.0); // No clutter
        world.insert_resource(clutter_grid);

        let mut hoarder_traits = Traits::default();
        hoarder_traits.add(Trait::Hoarder);

        // Hoarder in high clutter
        let hoarder_happy = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                hoarder_traits.clone(),
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        // Hoarder in no clutter
        let hoarder_neutral = world
            .spawn((
                Pop,
                GridPosition { x: 2, y: 2 },
                hoarder_traits,
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        // Normal pop in high clutter
        let normal_pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Traits::default(),
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(hoarder_comfort_system).unwrap();

        let happy_needs = world.get::<Needs>(hoarder_happy).unwrap();
        assert!(
            happy_needs.leisure > 0.5,
            "Hoarder in clutter should gain leisure"
        );

        let neutral_needs = world.get::<Needs>(hoarder_neutral).unwrap();
        assert_eq!(
            neutral_needs.leisure, 0.5,
            "Hoarder in clean area should not gain leisure"
        );

        let normal_needs = world.get::<Needs>(normal_pop).unwrap();
        assert_eq!(
            normal_needs.leisure, 0.5,
            "Normal pop should not gain leisure from clutter"
        );
    }
}
