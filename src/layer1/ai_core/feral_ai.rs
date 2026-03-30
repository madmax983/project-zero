use bevy_ecs::prelude::*;
use crate::layer1::medical::Hospital;
use crate::layer1::energy::{PowerConsumer, GridPower};
use crate::layer1::pathfinding::PathingGrid;

#[derive(Component)]
pub struct FeralAlgorithm {
    pub active: bool,
    pub target_population: u32,
}

#[derive(Component)]
pub struct FoodSource;

pub fn feral_algorithm_power_system(
    mut query: Query<(&Hospital, &mut PowerConsumer)>,
    feral_query: Query<&FeralAlgorithm>,
    power_grid: Option<Res<GridPower>>,
) {
    let power_grid = if let Some(pg) = power_grid {
        pg
    } else {
        return;
    };

    if let Ok(feral) = feral_query.get_single() {
        if feral.active && power_grid.available < power_grid.required {
            for (_, mut power) in &mut query {
                power.active = false;
            }
        }
    }
}

pub fn feral_algorithm_routing_system(
    feral_query: Query<&FeralAlgorithm>,
    mut pathing_grid: Option<ResMut<PathingGrid>>,
) {
    let pg = if let Some(pg) = pathing_grid.as_deref_mut() {
        pg
    } else {
        return;
    };

    if let Ok(feral) = feral_query.get_single() {
        if feral.active {
            pg.route_food_to_bunker();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::health::Health;

    #[test]
    fn test_feral_algorithm_reroutes_food() {
        let mut app = App::new();
        app.init_resource::<PathingGrid>();
        app.add_systems(Update, feral_algorithm_routing_system);

        app.world_mut().spawn((Needs { hunger: 100.0, rest: 100.0, leisure: 100.0, hygiene: 100.0 }, Health { current: 100.0, max: 100.0 }));

        app.world_mut().spawn(FeralAlgorithm { active: true, target_population: 10 });

        app.world_mut().spawn(FoodSource);

        app.update();

        let grid = app.world().resource::<PathingGrid>();
        assert!(grid.is_food_routed_to_bunker());
    }

    #[test]
    fn test_feral_algorithm_cuts_life_support() {
        let mut app = App::new();
        app.init_resource::<GridPower>();
        app.add_systems(Update, feral_algorithm_power_system);

        let hospital = app.world_mut().spawn((Hospital::default(), PowerConsumer { active: true, demand: 10.0 })).id();
        app.world_mut().spawn(Health { current: 10.0, max: 100.0 });

        app.world_mut().spawn(FeralAlgorithm { active: true, target_population: 100 });

        app.world_mut().insert_resource(GridPower { available: 50, required: 100 });
        app.update();

        let hospital_power = app.world().get::<PowerConsumer>(hospital).unwrap();
        assert!(!hospital_power.active);
    }
}
