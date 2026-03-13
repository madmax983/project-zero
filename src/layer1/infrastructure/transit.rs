use bevy_ecs::prelude::*;
use crate::layer1::stress::StressTracker;
use crate::layer1::economy::Wallet;
use crate::layer1::map::GridPosition;

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

use std::collections::HashMap;

/// System to deduct wealth or apply stress for pops using toll transit infrastructure.
pub fn transit_toll_system(
    mut pops: Query<(&GridPosition, &mut Wallet, &mut StressTracker)>,
    roads: Query<(&GridPosition, &Toll), With<TransitInfrastructure>>,
) {
    let mut toll_map = HashMap::new();
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
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::stress::StressTracker;
    use crate::layer1::economy::Wallet;
    use crate::layer1::map::GridPosition;
    use crate::layer1::infrastructure::transit::{TransitInfrastructure, Toll, transit_toll_system};

    #[test]
    fn test_pop_pays_toll_on_transit() {
        let mut world = World::new();

        // 1. Create a road with a toll
        let road_pos = GridPosition { x: 5, y: 5 };
        world.spawn((
            TransitInfrastructure { speed_multiplier: 2.0 },
            Toll { cost: 1.0 },
            road_pos,
        ));

        // 2. Create a Pop with wealth moving onto the road
        let pop = world.spawn((
            Pop,
            road_pos, // Pop is on the road
            Wallet { credits: 10.0 },
            StressTracker::default(),
        )).id();

        // 3. Run the toll system
        let mut schedule = Schedule::default();
        schedule.add_systems(transit_toll_system);
        schedule.run(&mut world);

        // 4. Assert Pop lost credits
        let pop_wealth = world.get::<Wallet>(pop).unwrap();
        assert_eq!(pop_wealth.credits, 9.0, "Pop should have paid 1.0 credit for the toll");
    }

    #[test]
    fn test_pop_morale_penalty_if_cannot_pay() {
        let mut world = World::new();

        // 1. Create an expensive road
        let road_pos = GridPosition { x: 5, y: 5 };
        world.spawn((
            TransitInfrastructure { speed_multiplier: 2.0 },
            Toll { cost: 5.0 },
            road_pos,
        ));

        // 2. Create a broke Pop
        let pop = world.spawn((
            Pop,
            road_pos,
            Wallet { credits: 0.0 },
            StressTracker { accumulated_stress: 10.0 },
        )).id();

        // 3. Run the toll system
        let mut schedule = Schedule::default();
        schedule.add_systems(transit_toll_system);
        schedule.run(&mut world);

        // 4. Assert Pop is broke but highly stressed
        let pop_wealth = world.get::<Wallet>(pop).unwrap();
        assert_eq!(pop_wealth.credits, 0.0);
        let pop_mood = world.get::<StressTracker>(pop).unwrap();
        assert!(pop_mood.accumulated_stress > 10.0, "Pop should incur stress if they cannot afford the toll");
    }
}
