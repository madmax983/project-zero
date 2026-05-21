// Stubs for RED phase to make it compile

use bevy::prelude::*;

#[derive(Component)]
pub struct SolarPowerGrid {
    pub current_output: f32,
    pub max_output: f32,
}

#[derive(Component)]
pub struct Temperature {
    pub current: f32,
}

#[derive(Component)]
pub struct ColonyInfrastructure {
    pub integrity: f32,
}

#[derive(Component)]
pub struct VoidSwarm {
    pub size: u32,
}

#[derive(Component)]
pub struct EclipseTarget(pub Entity);

pub fn swarm_eclipse_system(
    swarm_query: Query<(&EclipseTarget, &VoidSwarm)>,
    mut planet_query: Query<(&mut SolarPowerGrid, &mut Temperature)>,
) {
    for (target, swarm) in swarm_query.iter() {
        if let Ok((mut grid, mut temp)) = planet_query.get_mut(target.0) {
            grid.current_output = 0.0;
            temp.current -= 5.0 * (swarm.size as f32 / 1000.0);
        }
    }
}

pub fn swarm_bio_waste_system(
    swarm_query: Query<(&EclipseTarget, &VoidSwarm)>,
    mut planet_query: Query<&mut ColonyInfrastructure>,
) {
    for (target, swarm) in swarm_query.iter() {
        if let Ok(mut infra) = planet_query.get_mut(target.0) {
            infra.integrity -= 10.0 * (swarm.size as f32 / 1000.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_eclipse_power_drop() {
        let mut app = App::new();
        app.add_systems(Update, swarm_eclipse_system);

        let planet = app
            .world_mut()
            .spawn((
                SolarPowerGrid {
                    current_output: 100.0,
                    max_output: 100.0,
                },
                Temperature { current: 20.0 },
            ))
            .id();

        app.world_mut()
            .spawn((VoidSwarm { size: 1000 }, EclipseTarget(planet)));

        app.update();

        let grid = app.world().get::<SolarPowerGrid>(planet).unwrap();
        assert_eq!(grid.current_output, 0.0);
        let temp = app.world().get::<Temperature>(planet).unwrap();
        assert!(temp.current < 20.0);
    }

    #[test]
    fn test_swarm_corrosive_rain() {
        let mut app = App::new();
        app.add_systems(Update, swarm_bio_waste_system);

        let planet = app
            .world_mut()
            .spawn((ColonyInfrastructure { integrity: 100.0 },))
            .id();

        app.world_mut()
            .spawn((VoidSwarm { size: 1000 }, EclipseTarget(planet)));

        app.update();

        let infra = app.world().get::<ColonyInfrastructure>(planet).unwrap();
        assert!(infra.integrity < 100.0);
    }
}
