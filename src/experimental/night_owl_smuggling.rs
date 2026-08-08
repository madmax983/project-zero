#![allow(clippy::type_complexity)]
//! Night Owl Smuggling (Nova Feature).
//!
//! # The Spark
//! We have the `Trait::NightOwl`, `TimeOfDay::Night`, and an underground economic entity `SmugglersCove` from the black market system. Could Night Owls do shady deals while the colony sleeps?
//!
//! # The Feature
//! Pops with the NightOwl trait passively locate and extract small amounts of credits from Smugglers Coves when active at night, draining a tiny bit of their rest in exchange for illegal profit.
//!
//! # The Potential
//! Connects a temporal trait (`NightOwl`) with an underground economic entity (`SmugglersCove`), giving a mechanical edge to a psychological trait that normally just affects mood.

use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
use crate::layer1::economy::smugglers_cove::SmugglersCove;
use crate::layer1::economy::Wallet;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

const SMUGGLING_RADIUS: u32 = 3;
const SMUGGLING_PROFIT_PER_TICK: f32 = 0.05;
const SMUGGLING_REST_DRAIN_PER_TICK: f32 = 0.01;

pub fn night_owl_smuggling_system(
    mut pops: Query<(&GridPosition, &Traits, &mut Wallet, &mut Needs), With<Pop>>,
    coves: Query<&GridPosition, With<SmugglersCove>>,
    cycle: Res<DayNightCycle>,
) {
    if cycle.time_of_day != TimeOfDay::Night {
        return;
    }

    for (pop_pos, traits, mut wallet, mut needs) in pops.iter_mut() {
        if traits.has(Trait::NightOwl) {
            let mut near_cove = false;
            for cove_pos in coves.iter() {
                if pop_pos.distance_chebyshev(*cove_pos) <= SMUGGLING_RADIUS {
                    near_cove = true;
                    break;
                }
            }

            if near_cove {
                wallet.credits += SMUGGLING_PROFIT_PER_TICK;
                needs.rest = (needs.rest - SMUGGLING_REST_DRAIN_PER_TICK).max(0.0);
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(night_owl_smuggling_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_night_owl_smuggling_at_night() {
        let mut world = World::new();
        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Night,
            day_count: 0,
            ticks_per_day: 100,
        });

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::NightOwl);
            t
        };

        let pop = world
            .spawn((
                Pop,
                traits,
                GridPosition { x: 5, y: 5 },
                Wallet { credits: 10.0 },
                Needs {
                    rest: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.spawn((
            SmugglersCove { lifespan: 100 },
            GridPosition { x: 5, y: 7 }, // Distance 2
        ));

        world.run_system_once(night_owl_smuggling_system).unwrap();

        let wallet = world.get::<Wallet>(pop).unwrap();
        assert!(wallet.credits > 10.0, "Should have gained credits");
        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.rest < 0.5, "Should have lost rest");
    }

    #[test]
    fn test_night_owl_smuggling_fails_during_day() {
        let mut world = World::new();
        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Day,
            day_count: 0,
            ticks_per_day: 100,
        });

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::NightOwl);
            t
        };

        let pop = world
            .spawn((
                Pop,
                traits,
                GridPosition { x: 5, y: 5 },
                Wallet { credits: 10.0 },
                Needs {
                    rest: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.spawn((SmugglersCove { lifespan: 100 }, GridPosition { x: 5, y: 6 }));

        world.run_system_once(night_owl_smuggling_system).unwrap();

        let wallet = world.get::<Wallet>(pop).unwrap();
        assert_eq!(wallet.credits, 10.0, "Should not gain credits during day");
    }

    #[test]
    fn test_non_night_owl_fails_smuggling() {
        let mut world = World::new();
        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Night,
            day_count: 0,
            ticks_per_day: 100,
        });

        let pop = world
            .spawn((
                Pop,
                Traits::default(),
                GridPosition { x: 5, y: 5 },
                Wallet { credits: 10.0 },
                Needs {
                    rest: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.spawn((SmugglersCove { lifespan: 100 }, GridPosition { x: 5, y: 6 }));

        world.run_system_once(night_owl_smuggling_system).unwrap();

        let wallet = world.get::<Wallet>(pop).unwrap();
        assert_eq!(
            wallet.credits, 10.0,
            "Non-night-owl should not gain credits"
        );
    }
}
