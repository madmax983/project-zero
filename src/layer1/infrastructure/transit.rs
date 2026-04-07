use crate::layer1::economy::Wallet;
use crate::layer1::map::GridPosition;
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;

/// Component representing transit infrastructure on a tile.
#[derive(Component)]
pub struct TransitInfrastructure {
    /// Multiplier to movement speed applied by this infrastructure.
    pub speed_multiplier: f32,
}

/// Component representing a toll cost assigned to infrastructure.
#[derive(Component)]
pub struct Toll {
    /// Cost in credits to use this transit.
    pub cost: f32,
}

use bevy::utils::HashMap;

/// System to deduct wealth or apply stress for pops using toll transit infrastructure.
/// ⚡ Bolt Optimization: Uses `bevy::utils::HashMap` (AHash) and pre-allocates capacity to reduce hashing overhead and heap allocations.
pub fn transit_toll_system(
    mut pops: Query<(&GridPosition, &mut Wallet, &mut StressTracker), Changed<GridPosition>>,
    roads: Query<(&GridPosition, &Toll), With<TransitInfrastructure>>,
) {
    let mut toll_map = HashMap::with_capacity(roads.iter().len());
    for (pos, toll) in roads.iter() {
        toll_map.insert(*pos, toll.cost);
    }

    for (pop_pos, mut wallet, mut stress) in pops.iter_mut() {
        if let Some(&cost) = toll_map.get(pop_pos) {
            if wallet.credits >= cost {
                wallet.credits -= cost;
            } else {
                stress.accumulated_stress += cost * 2.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::economy::Wallet;
    use crate::layer1::infrastructure::transit::{
        transit_toll_system, Toll, TransitInfrastructure,
    };
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::stress::StressTracker;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_pop_pays_toll_on_transit() {
        let mut world = World::new();

        // 1. Create a road with a toll
        let road_pos = GridPosition { x: 5, y: 5 };
        world.spawn((
            TransitInfrastructure {
                speed_multiplier: 2.0,
            },
            Toll { cost: 1.0 },
            road_pos,
        ));

        // 2. Create a Pop with wealth moving onto the road
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 4 }, // Start off road
                Wallet { credits: 10.0 },
                StressTracker::default(),
            ))
            .id();

        // Simulate movement onto the road
        world
            .get_mut::<GridPosition>(pop)
            .expect("Missing GridPosition")
            .y = 5;

        // 3. Run the toll system
        let mut schedule = Schedule::default();
        schedule.add_systems(transit_toll_system);
        schedule.run(&mut world);

        // 4. Assert Pop lost credits
        let pop_wealth = world.get::<Wallet>(pop).expect("Missing Wallet");
        assert_eq!(
            pop_wealth.credits, 9.0,
            "Pop should have paid 1.0 credit for the toll"
        );
    }

    #[test]
    fn test_transit_toll_integration() {
        use crate::layer1::erosion::ErosionGrid;
        use crate::layer1::execution::components::MovementTarget;
        use crate::layer1::execution::movement_system;
        use crate::layer1::map::GridPosition;
        use crate::layer1::pop::Speed;
        use crate::layer1::terrain::{generate_terrain, TerrainType};
        use crate::layer1::utility_types::ActionType;

        let mut world = World::new();

        world.insert_resource(ErosionGrid::new(10, 10));
        let mut terrain = generate_terrain(10, 10);
        terrain.set(5, 5, TerrainType::Grass);
        terrain.set(6, 5, TerrainType::Grass);
        world.insert_resource(terrain);

        // 1. Create a road with a toll
        world.spawn((
            TransitInfrastructure {
                speed_multiplier: 2.0,
            },
            Toll { cost: 1.0 },
            GridPosition { x: 6, y: 5 },
        ));

        // 2. Create a Pop moving onto the road
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 }, // Start off road
                Wallet { credits: 10.0 },
                StressTracker::default(),
                Speed {
                    base: 1.0,
                    current: 1.0,
                    accumulator: 1.0,
                },
                MovementTarget {
                    target_entity: Entity::PLACEHOLDER,
                    target_position: GridPosition { x: 6, y: 5 },
                    for_action: ActionType::Explore,
                },
            ))
            .id();

        // 3. Run the movement system followed by toll system
        let mut schedule = Schedule::default();
        schedule.add_systems((movement_system, transit_toll_system).chain());
        schedule.run(&mut world);

        // 4. Assert Pop moved
        let new_pos = world
            .get::<GridPosition>(pop)
            .expect("Missing GridPosition");
        assert_eq!(new_pos.x, 6);
        assert_eq!(new_pos.y, 5);

        // 5. Assert Pop lost credits
        let pop_wealth = world.get::<Wallet>(pop).expect("Missing Wallet");
        assert_eq!(
            pop_wealth.credits, 9.0,
            "Pop should have paid 1.0 credit for the toll after moving"
        );
    }

    #[test]
    fn test_pop_morale_penalty_if_cannot_pay() {
        let mut world = World::new();

        // 1. Create an expensive road
        let road_pos = GridPosition { x: 5, y: 5 };
        world.spawn((
            TransitInfrastructure {
                speed_multiplier: 2.0,
            },
            Toll { cost: 5.0 },
            road_pos,
        ));

        // 2. Create a broke Pop
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 4 },
                Wallet { credits: 0.0 },
                StressTracker {
                    accumulated_stress: 10.0,
                },
            ))
            .id();

        // Simulate movement onto the road
        world
            .get_mut::<GridPosition>(pop)
            .expect("Missing GridPosition")
            .y = 5;

        // 3. Run the toll system
        let mut schedule = Schedule::default();
        schedule.add_systems(transit_toll_system);
        schedule.run(&mut world);

        // 4. Assert Pop is broke but highly stressed
        let pop_wealth = world.get::<Wallet>(pop).expect("Missing Wallet");
        assert_eq!(pop_wealth.credits, 0.0);
        let pop_mood = world
            .get::<StressTracker>(pop)
            .expect("Missing StressTracker");
        assert!(
            pop_mood.accumulated_stress > 10.0,
            "Pop should incur stress if they cannot afford the toll"
        );
    }
}
