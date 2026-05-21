use crate::layer1::entities::pop::Job;
use crate::layer1::entities::pop::Pop;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Intelligence {
    pub value: u32,
}

#[derive(Component)]
pub struct SirenObsession;

#[derive(Event)]
pub struct SirenSignalEvent;

#[derive(Component)]
pub struct Idle;

#[derive(Component)]
pub struct BuildingAntenna;

#[derive(Resource)]
pub struct SirenConfig {
    pub threshold: u32,
}

impl Default for SirenConfig {
    fn default() -> Self {
        Self { threshold: 80 }
    }
}

pub fn apply_siren_obsession(
    mut commands: Commands,
    mut events: EventReader<SirenSignalEvent>,
    config: Res<SirenConfig>,
    query: Query<(Entity, &Intelligence), With<Pop>>,
) {
    if !events.is_empty() {
        events.clear();
        for (entity, int) in query.iter() {
            if int.value >= config.threshold {
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
        commands
            .entity(entity)
            .remove::<Job>()
            .insert(BuildingAntenna);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::mind::utility_types::AssignmentType;

    #[test]
    fn test_siren_signal_inflicts_obsession_on_high_int_pops() {
        let mut app = bevy_app::App::new();
        app.init_resource::<SirenConfig>();
        app.add_event::<SirenSignalEvent>();
        app.add_systems(bevy_app::Update, apply_siren_obsession);

        let normal_pop = app
            .world_mut()
            .spawn((Pop, Intelligence { value: 50 }))
            .id();
        let smart_pop = app
            .world_mut()
            .spawn((Pop, Intelligence { value: 95 }))
            .id();

        app.world_mut().send_event(SirenSignalEvent);
        app.update();

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
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, handle_obsessed_jobs);

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Intelligence { value: 90 },
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: AssignmentType::ObservatoryWorker,
                },
                SirenObsession,
            ))
            .id();

        app.update();

        assert!(
            app.world().get::<Job>(pop).is_none(),
            "Obsessed pop must abandon their current job."
        );
        assert!(
            app.world().get::<Idle>(pop).is_some()
                || app.world().get::<BuildingAntenna>(pop).is_some(),
            "Pop should be building an antenna."
        );
    }
}
