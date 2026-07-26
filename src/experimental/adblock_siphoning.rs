//! Adblock Siphoning (Nova Feature).
//!
//! # The Spark
//! We have the `Trait::Hacker` personality trait, the `ColonyResources` (credits), and the `BuildingType::Billboard` which represents corporate sponsorship.
//!
//! # The Feature
//! Pops with the `Trait::Hacker` can skim money off corporate ads.
//! When they are standing within a 3-tile radius (Chebyshev distance) of a `Billboard`,
//! they passively generate `credits` for the colony by intercepting micro-transactions, but the mental focus required causes their `rest` need to decay.
//!
//! # The Potential
//! This turns a purely cosmetic/vanity building (Billboard) into a small but consistent
//! economic generator, provided the player clusters Hackers around commercial zones, creating emergent 'cyberpunk' districts.

use crate::layer1::architecture::building::{Building, BuildingType};
use crate::layer1::core::map::GridPosition;
use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

const SIPHON_RADIUS: u32 = 3;
const CREDITS_GENERATED: f32 = 0.5;
const REST_DECAY: f32 = 0.02;

pub fn adblock_siphoning_system(
    mut pops: Query<(&GridPosition, &mut Needs, &Traits), With<Pop>>,
    billboards: Query<(&GridPosition, &Building)>,
    mut colony_resources: ResMut<ColonyResources>,
) {
    for (pop_pos, mut needs, traits) in pops.iter_mut() {
        if traits.has(Trait::Hacker) {
            let mut is_near_billboard = false;
            for (billboard_pos, building) in billboards.iter() {
                if building.building_type == BuildingType::Billboard
                    && pop_pos.distance_chebyshev(*billboard_pos) <= SIPHON_RADIUS
                {
                    is_near_billboard = true;
                    break;
                }
            }

            if is_near_billboard {
                colony_resources.add_credits(CREDITS_GENERATED);
                needs.rest = (needs.rest - REST_DECAY).max(0.0);
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(adblock_siphoning_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_hacker_near_billboard_generates_credits_and_loses_rest() {
        let mut world = World::new();

        world.insert_resource(ColonyResources {
            credits: 0.0,
            max_credits: 1000.0,
            ..Default::default()
        });

        world.spawn((
            GridPosition { x: 5, y: 5 },
            Building {
                building_type: BuildingType::Billboard,
            },
        ));

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Hacker);
            t
        };

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 6 }, // Within radius 3
                traits,
                Needs {
                    rest: 1.0,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(adblock_siphoning_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        let resources = world.resource::<ColonyResources>();

        assert!(
            resources.credits > 0.0,
            "Hacker should generate credits near Billboard"
        );
        assert!(needs.rest < 1.0, "Hacker should lose rest near Billboard");
    }
}
