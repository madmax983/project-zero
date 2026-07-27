use crate::layer1::architecture::building::{Building, BuildingType};
use crate::layer1::core::map::GridPosition;
use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::entities::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;
use rand::Rng;

pub fn landfill_rummaging_system(
    mut pops: Query<(&GridPosition, &mut Needs, &Traits), With<Pop>>,
    buildings: Query<(&GridPosition, &Building)>,
    mut resources: ResMut<ColonyResources>,
) {
    let mut rng = rand::thread_rng();

    for (pop_pos, mut needs, traits) in pops.iter_mut() {
        if traits.has(Trait::Hoarder) {
            let is_near_landfill = buildings.iter().any(|(b_pos, building)| {
                building.building_type == BuildingType::Landfill
                    && pop_pos.distance_chebyshev(*b_pos) <= 2
            });

            if is_near_landfill {
                // Passively regenerate leisure at the cost of hygiene
                needs.leisure = (needs.leisure + 0.05).min(1.0);
                needs.hygiene = (needs.hygiene - 0.05).max(0.0);

                // Small chance to find scrap or metal
                if rng.gen_bool(0.01) {
                    // 1% chance per tick
                    if rng.gen_bool(0.5) {
                        resources.scrap += 1.0;
                    } else {
                        resources.metal += 1.0;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_landfill_rummaging() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn a Landfill
        world.spawn((
            Building {
                building_type: BuildingType::Landfill,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn a Hoarder near the Landfill
        let mut hoarder_traits = Traits::default();
        hoarder_traits.add(Trait::Hoarder);
        let hoarder = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.5,
                    hygiene: 0.5,
                    hunger: 1.0,
                    rest: 1.0,
                },
                hoarder_traits,
                GridPosition { x: 6, y: 5 }, // distance 1
            ))
            .id();

        // Spawn a normal pop near the Landfill
        let normal = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.5,
                    hygiene: 0.5,
                    hunger: 1.0,
                    rest: 1.0,
                },
                Traits::default(),
                GridPosition { x: 6, y: 5 },
            ))
            .id();

        use bevy_ecs::system::RunSystemOnce;
        world
            .run_system_once(super::landfill_rummaging_system)
            .unwrap();

        // Check Hoarder near Landfill
        let hoarder_needs = world.get::<Needs>(hoarder).unwrap();
        assert!((hoarder_needs.leisure - 0.55).abs() < 0.001);
        assert!((hoarder_needs.hygiene - 0.45).abs() < 0.001);

        // Check normal pop near Landfill
        let normal_needs = world.get::<Needs>(normal).unwrap();
        assert!((normal_needs.leisure - 0.5).abs() < 0.001);
        assert!((normal_needs.hygiene - 0.5).abs() < 0.001);
    }
}
