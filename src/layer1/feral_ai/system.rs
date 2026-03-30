use bevy_ecs::prelude::*;
use crate::layer1::energy::PowerConsumer;
use crate::layer1::medical::Hospital;

#[derive(Component)]
pub struct FeralAlgorithm {
    pub active: bool,
    pub target_population: u32,
}

#[derive(Resource, Default)]
pub struct GridPower {
    pub available: u32,
    pub required: u32,
}

#[derive(Component)]
pub struct FoodSource;

pub fn feral_algorithm_power_system(
    mut query: Query<(&Hospital, &mut PowerConsumer)>,
    feral_query: Query<&FeralAlgorithm>,
    power_grid: Option<Res<GridPower>>,
) {
    if let Ok(feral) = feral_query.get_single() {
        if let Some(grid) = power_grid {
            if feral.active && grid.available < grid.required {
                for (_, mut power) in query.iter_mut() {
                    power.active = false;
                }
            }
        }
    }
}

#[derive(Component)]
pub struct FeralControlled;

pub fn feral_algorithm_routing_system(
    feral_query: Query<&FeralAlgorithm>,
    mut food_query: Query<(Entity, &FoodSource), Without<FeralControlled>>,
    mut commands: Commands,
) {
    if let Ok(feral) = feral_query.get_single() {
        if feral.active {
            for (entity, _) in food_query.iter_mut() {
                commands.entity(entity).insert(FeralControlled);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use crate::layer1::feral_ai::system::{
        FeralAlgorithm, FoodSource, GridPower, feral_algorithm_power_system,
        feral_algorithm_routing_system, FeralControlled
    };
    use crate::layer1::medical::Hospital;
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::Needs;
    use crate::layer1::health::Health;

    #[test]
    fn test_feral_algorithm_reroutes_food() {
        // Arrange: Setup test data
        let mut app = App::new();
        app.add_systems(Update, feral_algorithm_routing_system);

        // Spawn a hungry pop
        let _pop = app.world_mut().spawn((Needs { hunger: 100.0, rest: 100.0, hygiene: 100.0, leisure: 100.0 }, Health { current: 100.0, max: 100.0 })).id();

        // Spawn Feral Algorithm component
        app.world_mut().spawn(FeralAlgorithm { active: true, target_population: 10 });

        // Spawn Food source
        let food_source = app.world_mut().spawn(FoodSource).id();

        // Act: Run the feral algorithm
        app.update();

        // Assert: The pathfinding should now route food away from the pop
        // (mocking pathfinding logic for the test by checking if FeralControlled marker was added)
        assert!(app.world().get::<FeralControlled>(food_source).is_some(), "Food source should be controlled by feral algorithm");
    }

    #[test]
    fn test_feral_algorithm_cuts_life_support() {
        // Arrange: Setup hospital with sick pop
        let mut app = App::new();
        app.add_systems(Update, feral_algorithm_power_system);

        let hospital = app.world_mut().spawn((Hospital {
            healing_rate: 1.0,
            max_healing_per_tick: 1.0,
        }, PowerConsumer { active: true, demand: 10.0 })).id();
        let _sick_pop = app.world_mut().spawn(Health { current: 10.0, max: 100.0 }).id(); // Negative utility

        app.world_mut().spawn(FeralAlgorithm { active: true, target_population: 100 });

        // Act: Run the feral algorithm during a brownout
        app.world_mut().insert_resource(GridPower { available: 50, required: 100 });
        app.update();

        // Assert: The hospital power should be cut to prioritize efficiency
        let hospital_power = app.world().get::<PowerConsumer>(hospital).unwrap();
        assert!(!hospital_power.active);
    }
}
