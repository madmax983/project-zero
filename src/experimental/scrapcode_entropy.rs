//! Scrapcode Entropy (Nova Feature).
//!
//! # The Spark
//! We have `Scrapcode` (an infection that increases building costs) and `BuildingAge` from `temporal_echoes.rs` (which ages buildings).
//!
//! # The Feature
//! If `Scrapcode` is active, it actively accelerates the `BuildingAge` of all existing buildings every tick based on its severity, causing them to decay and collapse faster over time.
//!
//! # The Potential
//! Turns the `Scrapcode` infection from a simple economic penalty into an existential threat for the colony's infrastructure. If the player ignores purging the Scrapcode, their base will literally crumble around them due to accelerated temporal entropy.

use crate::layer1::anomalies::temporal_echoes::BuildingAge;
use crate::layer1::architecture::building::Building;
use crate::layer1::scrapcode::Scrapcode;
use bevy_ecs::prelude::*;

/// System that accelerates building aging while Scrapcode is active.
pub fn scrapcode_entropy_system(
    scrapcode: Res<Scrapcode>,
    mut buildings: Query<&mut BuildingAge, With<Building>>,
) {
    if scrapcode.active {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // If severity is 1.5, we add 1 extra tick of age (total 2). If 2.0, we add 2 extra.
        let extra_aging = (scrapcode.severity * 1.0).max(0.0) as u32;

        if extra_aging > 0 {
            for mut age in buildings.iter_mut() {
                age.ticks += extra_aging;
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(scrapcode_entropy_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_scrapcode_accelerates_building_age() {
        let mut world = World::new();

        world.insert_resource(Scrapcode {
            active: true,
            severity: 2.5, // High severity
            duration: 100,
        });

        let building = world
            .spawn((
                Building {
                    building_type: crate::layer1::architecture::building::BuildingType::Housing,
                },
                BuildingAge { ticks: 100 },
            ))
            .id();

        world.run_system_once(scrapcode_entropy_system).unwrap();

        let age = world.get::<BuildingAge>(building).unwrap();
        // Base was 100. Extra aging should be 2.5 * 1.0 = 2.
        assert_eq!(age.ticks, 102);
    }

    #[test]
    fn test_inactive_scrapcode_does_not_accelerate_age() {
        let mut world = World::new();

        world.insert_resource(Scrapcode {
            active: false,
            severity: 2.5,
            duration: 0,
        });

        let building = world
            .spawn((
                Building {
                    building_type: crate::layer1::architecture::building::BuildingType::Housing,
                },
                BuildingAge { ticks: 100 },
            ))
            .id();

        world.run_system_once(scrapcode_entropy_system).unwrap();

        let age = world.get::<BuildingAge>(building).unwrap();
        assert_eq!(age.ticks, 100);
    }
}
