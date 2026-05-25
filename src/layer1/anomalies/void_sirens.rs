//! # The Void Sirens
//!
//! Strange hypnotic signals broadcast from deep space. These signals specifically target
//! highly intelligent pops, causing them to abandon their duties and obsessively build
//! strange antenna structures to communicate back.

use crate::layer1::entities::pop::Job;
use crate::layer1::entities::pop::Pop;
use crate::layer3::diplomacy::brain_drain::Intelligence;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct SirenObsession;

#[derive(Event)]
pub struct SirenSignalEvent;

#[derive(Component)]
pub struct BuildingAntenna;

const OBSESSION_INT_THRESHOLD: u32 = 80;

pub fn apply_siren_obsession(
    mut commands: Commands,
    mut events: EventReader<SirenSignalEvent>,
    query: Query<(Entity, &Intelligence), With<Pop>>,
) {
    if !events.is_empty() {
        events.clear(); // Consume event
        for (entity, int) in query.iter() {
            if int.value >= OBSESSION_INT_THRESHOLD {
                commands.entity(entity).insert(SirenObsession);
            }
        }
    }
}

pub fn handle_obsessed_jobs(
    mut commands: Commands,
    query: Query<Entity, (With<SirenObsession>, With<Job>)>,
) {
    for entity in query.iter() {
        // Remove job and make them build an antenna
        commands
            .entity(entity)
            .remove::<Job>()
            .insert(BuildingAntenna);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_siren_signal_inflicts_obsession_on_high_int_pops() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        // Spawn normal pop
        let normal_pop = app
            .world_mut()
            .spawn((Pop, Intelligence { value: 50 }))
            .id();

        // Spawn high int pop
        let smart_pop = app
            .world_mut()
            .spawn((Pop, Intelligence { value: 95 }))
            .id();

        // Act
        // Trigger Siren Signal Event
        app.add_event::<SirenSignalEvent>();
        app.world_mut().send_event(SirenSignalEvent);
        app.add_systems(Update, apply_siren_obsession);
        app.update();

        // Assert
        assert!(
            app.world().get::<SirenObsession>(normal_pop).is_none(),
            "Normal pop should not be obsessed."
        );
        assert!(
            app.world().get::<SirenObsession>(smart_pop).is_some(),
            "High INT pop should become obsessed."
        );
    }

    #[test]
    fn test_obsessed_pop_abandons_current_job() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Intelligence { value: 90 },
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: crate::layer1::utility_types::AssignmentType::Administrator,
                }, // Use existing variant
                SirenObsession, // Already obsessed
            ))
            .id();

        // Act
        app.add_systems(Update, handle_obsessed_jobs);
        app.update();

        // Assert
        assert!(
            app.world().get::<Job>(pop).is_none(),
            "Obsessed pop must abandon their current job."
        );
        assert!(
            app.world().get::<BuildingAntenna>(pop).is_some(),
            "Pop should be building an antenna."
        );
    }
}
