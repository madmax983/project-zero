use bevy_ecs::prelude::*;
use crate::layer1::entities::pop::{Pop, Job};
use crate::layer1::social::morale::Morale;
use crate::layer1::mind::utility_types::{AssignmentType};

// 1. Define FeralCultist component
#[derive(Component)]
pub struct FeralCultist;

// 2. System to convert low-morale pops
pub fn cult_conversion_system(
    mut commands: Commands,
    query: Query<(Entity, &Morale), (With<Pop>, Without<FeralCultist>)>
) {
    for (entity, morale) in query.iter() {
        if morale.value < 20.0 { // Threshold
            commands.entity(entity).insert(FeralCultist);
            // Force them into cult activity instead of letting them idle
            commands.entity(entity).insert(Job {
                workplace: Entity::PLACEHOLDER,
                job_type: AssignmentType::CultActivity,
            });
        }
    }
}

// 3. Define HolySite and boosting logic
#[derive(Component)]
pub struct HolySite {
    pub cultists_present: u32,
}

#[derive(Component)]
pub struct EnergyPlant {
    pub output: u32,
}

pub fn cult_boosting_system(
    mut query: Query<(&HolySite, &mut EnergyPlant)>
) {
    for (site, mut plant) in query.iter_mut() {
        if site.cultists_present > 0 {
            // Only apply a flat boost instead of exponential growth
            plant.output = plant.output.saturating_add(50); // Simple +50 flat boost per tick
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::{Pop, Job};
    use crate::layer1::social::morale::Morale;
    use crate::layer1::mind::utility_types::AssignmentType;
    use crate::layer1::architecture::building::Building;

    fn setup_test_app() -> bevy::app::App {
        let mut app = bevy::app::App::new();
        app.add_systems(bevy::app::Update, (cult_conversion_system, cult_boosting_system));
        app
    }

    #[test]
    fn test_pop_joins_feral_cult_on_prolonged_low_morale() {
        // Arrange
        let mut app = setup_test_app();
        let pop = app.world_mut().spawn((
            Pop,
            Morale { value: 10.0, modifiers: vec![] }, // Critically low
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().entity(pop).contains::<FeralCultist>());
    }

    #[test]
    fn test_feral_cultist_boosts_building_output() {
        // Arrange
        let mut app = setup_test_app();
        let building = app.world_mut().spawn((
            Building::default(),
            EnergyPlant { output: 100 },
            HolySite { cultists_present: 5 },
        )).id();

        // Act
        app.update(); // Run cultist boost system

        // Assert
        let plant = app.world().entity(building).get::<EnergyPlant>().unwrap();
        assert!(plant.output > 100, "Cultists should over-boost output");
    }

    #[test]
    fn test_feral_cultist_ignores_normal_jobs() {
        // Arrange
        let mut app = setup_test_app();
        // Spawning with low morale so they convert
        let pop = app.world_mut().spawn((
            Pop,
            Morale { value: 10.0, modifiers: vec![] }, // Critically low
            Job { workplace: Entity::PLACEHOLDER, job_type: AssignmentType::DeepMining },
        )).id();

        // Act
        app.update(); // Run utility AI / job assignment

        // Assert
        let job = app.world().entity(pop).get::<Job>();
        assert!(job.is_none() || job.unwrap().job_type == AssignmentType::CultActivity);
    }
}
