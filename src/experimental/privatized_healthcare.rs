//! Privatized Healthcare System (Nova Feature)
//!
//! Connects `Health`, `Wallet`, and `BuildingType::Hospital`.
//! Pops who are injured and stand near a Hospital will automatically
//! siphon credits from their `Wallet` to rapidly regenerate their `Health`.
//! If they are broke, the hospital refuses to treat them.

use crate::layer1::building::{Building, BuildingType};
use crate::layer1::economy::Wallet;
use crate::layer1::health::Health;
use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

/// Health restored per tick.
const HEAL_AMOUNT: f32 = 5.0;
/// Cost in credits per tick for the healing.
const HEAL_COST: f32 = 2.0;
/// Proximity in tiles required.
const HOSPITAL_HEAL_RADIUS: f32 = 3.0;

/// Register the system
pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(privatized_healthcare_system);
}

/// System that drains credits for rapid healing near a hospital.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn privatized_healthcare_system(
    mut pops: Query<(&GridPosition, &mut Health, &mut Wallet)>,
    buildings: Query<(&Building, &GridPosition)>,
) {
    let hospital_positions: Vec<GridPosition> = buildings
        .iter()
        .filter(|(b, _)| b.building_type == BuildingType::Hospital)
        .map(|(_, p)| *p)
        .collect();

    if hospital_positions.is_empty() {
        return;
    }

    for (pop_pos, mut health, mut wallet) in pops.iter_mut() {
        if health.current < health.max {
            let is_near_hospital = hospital_positions.iter().any(|h_pos| {
                let dx = (pop_pos.x as f32) - (h_pos.x as f32);
                let dy = (pop_pos.y as f32) - (h_pos.y as f32);
                (dx * dx + dy * dy).sqrt() <= HOSPITAL_HEAL_RADIUS
            });

            if is_near_hospital && wallet.credits >= HEAL_COST {
                wallet.credits -= HEAL_COST;
                health.current = (health.current + HEAL_AMOUNT).min(health.max);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privatized_healthcare_healing() {
        let mut world = World::new();

        world.spawn((
            Building {
                building_type: BuildingType::Hospital,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let injured_rich_pop = world
            .spawn((
                Health {
                    current: 50.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                Wallet { credits: 10.0 },
                GridPosition { x: 6, y: 5 }, // distance 1
            ))
            .id();

        let injured_poor_pop = world
            .spawn((
                Health {
                    current: 50.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                Wallet { credits: 1.0 }, // Cannot afford it
                GridPosition { x: 6, y: 5 },
            ))
            .id();

        let mut schedule = Schedule::default();
        register(&mut schedule);
        schedule.run(&mut world);

        let rich_health = world.get::<Health>(injured_rich_pop).unwrap();
        let rich_wallet = world.get::<Wallet>(injured_rich_pop).unwrap();
        assert_eq!(rich_health.current, 55.0);
        assert_eq!(rich_wallet.credits, 8.0);

        let poor_health = world.get::<Health>(injured_poor_pop).unwrap();
        let poor_wallet = world.get::<Wallet>(injured_poor_pop).unwrap();
        assert_eq!(poor_health.current, 50.0);
        assert_eq!(poor_wallet.credits, 1.0);
    }
}
