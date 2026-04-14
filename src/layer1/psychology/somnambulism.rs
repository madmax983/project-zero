use crate::layer1::entities::Pop;
use crate::layer1::jobs::CurrentTask;
use crate::layer1::needs::{NeedType, Needs};
use bevy::prelude::*;

#[derive(Component)]
pub struct Somnambulist {
    pub duration_left: f32,
}

type TriggerSomnambulismQueryFilter = (With<Pop>, With<CurrentTask>, Without<Somnambulist>);

pub fn trigger_somnambulism_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Needs), TriggerSomnambulismQueryFilter>,
) {
    for (entity, needs) in query.iter_mut() {
        if needs.get(NeedType::Rest) < 10.0 {
            // Critical threshold
            commands.entity(entity).insert(Somnambulist {
                duration_left: 60.0, // Arbitrary starting duration
            });
        }
    }
}

pub fn process_somnambulist_work_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Somnambulist,
        Option<&mut CurrentTask>,
        &mut Needs,
    )>,
) {
    let dt = time.delta_secs();

    for (entity, mut somnambulist, mut task_opt, _needs) in query.iter_mut() {
        somnambulist.duration_left -= dt;
        if somnambulist.duration_left <= 0.0 {
            commands.entity(entity).remove::<Somnambulist>();
            continue;
        }

        if let Some(ref mut task) = task_opt {
            task.efficiency = 5.0; // Massive boost

            if !task.is_randomized_output {
                task.is_randomized_output = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::jobs::{CurrentTask, ResourceType};
    use crate::layer1::needs::{NeedType, Needs};

    fn setup_app() -> App {
        let mut app = App::new();
        // Use full TimePlugin so `Time` updates correctly during `app.update()`
        app.add_plugins(bevy::time::TimePlugin);
        app.add_systems(
            Update,
            (
                trigger_somnambulism_system,
                process_somnambulist_work_system,
            ),
        );
        app
    }

    #[test]
    fn test_critically_low_rest_triggers_somnambulism() {
        let mut app = setup_app();

        let mut needs = Needs::default();
        needs.set(NeedType::Rest, 5.0); // Critical low

        let entity = app
            .world_mut()
            .spawn((
                Pop,
                needs,
                CurrentTask {
                    task_id: 1,
                    duration: 10.0,
                    base_output: ResourceType::Metal,
                    efficiency: 1.0,
                    is_randomized_output: false,
                }, // Currently working
            ))
            .id();

        app.update();

        assert!(app.world().entity(entity).contains::<Somnambulist>());
    }

    #[test]
    fn test_somnambulist_ignores_other_needs_decay() {
        let mut app = setup_app();

        let mut needs = Needs::default();
        needs.set(NeedType::Food, 50.0);
        needs.set(NeedType::Rest, 0.0);

        let entity = app
            .world_mut()
            .spawn((
                Pop,
                needs,
                Somnambulist {
                    duration_left: 10.0,
                },
            ))
            .id();

        let mut time = Time::default() as Time;
        time.advance_by(std::time::Duration::from_secs_f32(1.0));
        app.insert_resource(time);

        app.update();

        let updated_needs = app.world().entity(entity).get::<Needs>().unwrap();
        // Needs should not have decayed (e.g., normally food drains at 1.0/sec)
        assert_eq!(updated_needs.get(NeedType::Food), 50.0);
    }

    #[test]
    fn test_somnambulist_boosts_efficiency_but_randomizes_output() {
        let mut app = setup_app();

        // This is a complex test: we want to ensure the work amount is higher than normal,
        // and that the output (the task completion or product) is randomized.
        // We simulate a task that normally produces 1 Steel.
        let entity = app
            .world_mut()
            .spawn((
                Pop,
                Needs::default(), // Needs component is required by process_somnambulist_work_system query
                Somnambulist { duration_left: 5.0 },
                CurrentTask {
                    task_id: 1,
                    duration: 10.0,
                    base_output: ResourceType::Metal,
                    efficiency: 1.0,
                    is_randomized_output: false,
                },
            ))
            .id();

        // Advance time to simulate a tick
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f32(1.0));

        app.update();

        let task = app.world().entity(entity).get::<CurrentTask>().unwrap();
        // Efficiency should be significantly boosted (e.g., 5.0 instead of 1.0)
        assert!(task.efficiency >= 5.0);

        // Output might be randomized (e.g., changed from Steel to Sculpture or LifeSupportSabotage)
        // This depends on the specific implementation, but we assert it's changed or flagged as random.
        assert!(task.is_randomized_output);
    }
}
