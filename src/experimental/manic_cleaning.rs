#![allow(clippy::type_complexity)]
//! Manic Cleaning (Nova Feature).
//!
//! # The Spark
//! We have `Catharsis` (a buff after a mental break) and `ClutterGrid`.
//!
//! # The Feature
//! Pops in `Catharsis` enter a state of "Manic Cleaning". As they move around,
//! they rapidly destroy `Clutter` on the tiles they step on.
//!
//! # The Potential
//! Turns a post-breakdown recovery state into a hyper-productive cleaning frenzy.
//! The colony becomes spotless after a collective breakdown.

use crate::layer1::clutter::ClutterGrid;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::stress::Catharsis;
use bevy_ecs::prelude::*;

const MANIC_CLEANING_AMOUNT: f32 = 10.0;

pub fn manic_cleaning_system(
    clutter_grid: Option<ResMut<ClutterGrid>>,
    pops: Query<&GridPosition, (With<Pop>, With<Catharsis>)>,
) {
    let Some(mut grid) = clutter_grid else {
        return;
    };

    for pos in pops.iter() {
        grid.remove_clutter(pos.x as usize, pos.y as usize, MANIC_CLEANING_AMOUNT);
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(manic_cleaning_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_manic_cleaning_reduces_clutter() {
        let mut world = World::new();

        let mut grid = ClutterGrid::new(10, 10);
        grid.add_clutter(5, 5, 20.0);
        world.insert_resource(grid);

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Catharsis {
                duration_remaining: 10,
                morale_bonus: 0.5,
            },
        ));

        world.run_system_once(manic_cleaning_system).unwrap();

        let grid = world.resource::<ClutterGrid>();
        assert!(
            grid.get(5, 5) < 20.0,
            "Catharsis pop should have cleaned some clutter"
        );
    }
}
