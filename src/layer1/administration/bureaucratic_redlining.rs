use crate::layer1::administration::zone::{ZoneGrid, ZoneType};
use crate::layer1::energy::PowerConsumer;
use crate::layer1::entities::pop::Pop;
use crate::layer1::map::GridPosition;
use crate::layer1::social::factions::{FactionId, FactionMember};
use bevy_ecs::prelude::*;

pub fn bureaucratic_redlining_system(
    zone_grid: Res<ZoneGrid>,
    mut power_consumers: Query<(&GridPosition, &mut PowerConsumer)>,
    mut pops: Query<(&GridPosition, &mut FactionMember), With<Pop>>,
) {
    // Disable power consumers in Dezoned areas
    for (pos, mut consumer) in power_consumers.iter_mut() {
        if zone_grid.get(pos.x, pos.y) == ZoneType::Dezoned {
            consumer.active = false;
        }
    }

    // Convert pops in Dezoned areas to Stateless faction
    for (pos, mut faction_member) in pops.iter_mut() {
        if zone_grid.get(pos.x, pos.y) == ZoneType::Dezoned {
            faction_member.faction_id = Some(FactionId::Stateless);
        }
    }
}

pub fn stateless_expansion_system(
    mut zone_grid: ResMut<ZoneGrid>,
    security: Option<Res<crate::layer1::edicts::ColonyPolicies>>,
) {
    let mut to_dezone = Vec::new();
    let width = zone_grid.width;
    let height = zone_grid.height;

    let spread_chance = if let Some(policies) = security {
        if policies.is_active(crate::layer1::edicts::Policy::MartialLaw) {
            0.005 // Lower chance under martial law
        } else {
            0.01
        }
    } else {
        0.01
    };

    // Simple probability approach (as instructed by GREEN phase but adapted for resource)
    for y in 0..height {
        for x in 0..width {
            if zone_grid.get(x as i32, y as i32) == ZoneType::Dezoned {
                // Check neighbors
                let neighbors = [
                    (x as i32 + 1, y as i32),
                    (x as i32 - 1, y as i32),
                    (x as i32, y as i32 + 1),
                    (x as i32, y as i32 - 1),
                ];

                for (nx, ny) in neighbors {
                    if zone_grid.get(nx, ny) != ZoneType::Dezoned
                        && zone_grid.get(nx, ny) != ZoneType::None
                        && rand::random::<f32>() < spread_chance
                    {
                        to_dezone.push((nx, ny));
                    }
                }
            }
        }
    }

    for (x, y) in to_dezone {
        zone_grid.set(x, y, ZoneType::Dezoned);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::administration::zone::{ZoneGrid, ZoneType};
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::map::GridPosition;
    use crate::layer1::social::factions::{FactionId, FactionMember};

    #[test]
    fn test_dezoning_disables_grid_and_creates_stateless_faction() {
        let mut world = World::new();

        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(5, 5, ZoneType::Dezoned);
        world.insert_resource(zone_grid);

        let node_entity = world
            .spawn((
                PowerConsumer {
                    active: true,
                    demand: 10.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let pop_entity = world
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
            &mut world,
            bureaucratic_redlining_system,
        );

        let consumer = world.get::<PowerConsumer>(node_entity).unwrap();
        assert!(
            !consumer.active,
            "PowerConsumer should be disabled in dezoned zone"
        );

        let faction = world.get::<FactionMember>(pop_entity).unwrap();
        assert_eq!(
            faction.faction_id,
            Some(FactionId::Stateless),
            "Pop faction should be 'Stateless'"
        );
    }

    #[test]
    fn test_stateless_faction_spreads_to_adjacent_zones() {
        let mut world = World::new();

        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(5, 5, ZoneType::Dezoned); // Redlined zone
        zone_grid.set(6, 5, ZoneType::Bedroom); // Adjacent normal zone
        world.insert_resource(zone_grid);

        // We use a high probability via mocking or simply run the system multiple times to trigger the spread.
        // For testing, we mock RNG or just test the logic that the system executes.
        // As rand::random is used, we can't easily mock it without rewriting the system, so we will run it
        // many times to practically guarantee a spread, or just acknowledge it's stochastic.

        // Actually, to make it deterministic, we could inject a seed or just trust the logic.
        // Let's run it 1000 times. The chance of not spreading is (0.99)^1000 = 4.3e-5, which is low.
        // But the spread system just looks for adjacent Non-Dezoned, Non-None zones.

        let mut spread = false;
        for _ in 0..1000 {
            let _ = bevy_ecs::system::RunSystemOnce::run_system_once(
                &mut world,
                stateless_expansion_system,
            );
            let grid = world.resource::<ZoneGrid>();
            if grid.get(6, 5) == ZoneType::Dezoned {
                spread = true;
                break;
            }
        }

        assert!(
            spread,
            "Stateless faction should have spread to adjacent zone"
        );
    }
}
