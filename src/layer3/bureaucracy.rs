use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::entities::pop::Pop;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct AutomatedReporting {
    pub is_active: bool,
    pub reported_population: usize, // The fake number sent to the capital
}

#[derive(Component)]
pub struct AutomatedDefenses {
    pub power_level: f32,
    pub is_active: bool,
}

// System to simulate the delayed realization of the colony's death
pub fn empire_resource_distribution_system(
    mut resources: ResMut<ColonyResources>,
    colonies: Query<(Entity, &AutomatedReporting)>,
) {
    for (_, reporting) in colonies.iter() {
        if reporting.is_active && reporting.reported_population > 0 {
            // The empire thinks the colony is alive and sends resources
            resources.food += 100.0;
        }
    }
}

// System where Layer 1 updates the reporting
pub fn colony_reporting_system(
    pops: Query<(), With<Pop>>,
    mut colonies: Query<&mut AutomatedReporting>,
) {
    let pop_count = pops.iter().count();
    for mut reporting in colonies.iter_mut() {
        if pop_count == 0 {
            // Automation might continue reporting the last known good number if not explicitly shut down
        } else {
            reporting.reported_population = pop_count;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_colony_reporting_with_pops() {
        let mut app = App::new();

        app.world_mut().spawn(Pop);
        app.world_mut().spawn(Pop);

        let reporting_entity = app
            .world_mut()
            .spawn(AutomatedReporting {
                is_active: true,
                reported_population: 0,
            })
            .id();

        app.add_systems(Update, colony_reporting_system);
        app.update();

        let reporting = app
            .world()
            .get::<AutomatedReporting>(reporting_entity)
            .unwrap();
        assert_eq!(reporting.reported_population, 2);
    }
}
