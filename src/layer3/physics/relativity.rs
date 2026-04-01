use bevy::prelude::*;

#[derive(Component)]
pub struct SystemNode;

#[derive(Component)]
pub struct TimeDilationZone {
    pub dilation_factor: u64,
}

#[derive(Component, Default)]
pub struct LocalTimeTracker {
    pub local_ticks: u64,
    pub accumulated_global_ticks: u64,
}

#[derive(Resource, Default)]
pub struct SimulationTime {
    pub tick: u64,
}

#[derive(Component)]
pub struct Fleet;

#[derive(Component)]
pub struct StationedAt(pub Entity);

pub fn process_time_dilation_system(
    global_time: Res<SimulationTime>,
    mut query: Query<(&TimeDilationZone, &mut LocalTimeTracker)>,
) {
    for (dilation, mut tracker) in query.iter_mut() {
        if dilation.dilation_factor > 0 {
            let delta = global_time
                .tick
                .saturating_sub(tracker.accumulated_global_ticks);
            let ticks_to_add = delta / dilation.dilation_factor;
            if ticks_to_add > 0 {
                tracker.local_ticks += ticks_to_add;
                tracker.accumulated_global_ticks += ticks_to_add * dilation.dilation_factor;
            }
        }
    }
}

pub fn update_fleet_local_time_system(
    system_query: Query<&LocalTimeTracker, (With<SystemNode>, Without<Fleet>)>,
    mut fleet_query: Query<(&StationedAt, &mut LocalTimeTracker), With<Fleet>>,
) {
    for (stationed, mut fleet_tracker) in fleet_query.iter_mut() {
        if let Ok(system_tracker) = system_query.get(stationed.0) {
            fleet_tracker.local_ticks = system_tracker.local_ticks;
            fleet_tracker.accumulated_global_ticks = system_tracker.accumulated_global_ticks;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_dilation_slows_down_local_ticks() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SimulationTime>();
        app.add_systems(Update, process_time_dilation_system);

        let system_entity = app
            .world_mut()
            .spawn((
                SystemNode,
                TimeDilationZone {
                    dilation_factor: 10,
                }, // 1 local tick per 10 global ticks
                LocalTimeTracker::default(),
            ))
            .id();

        for _ in 0..10 {
            app.world_mut().resource_mut::<SimulationTime>().tick += 1;
            app.update();
        }

        let tracker = app.world().get::<LocalTimeTracker>(system_entity).unwrap();
        assert_eq!(tracker.local_ticks, 1);

        for _ in 0..9 {
            app.world_mut().resource_mut::<SimulationTime>().tick += 1;
            app.update();
        }

        let tracker = app.world().get::<LocalTimeTracker>(system_entity).unwrap();
        assert_eq!(tracker.local_ticks, 1); // Not 2 yet

        app.world_mut().resource_mut::<SimulationTime>().tick += 1;
        app.update();

        let tracker = app.world().get::<LocalTimeTracker>(system_entity).unwrap();
        assert_eq!(tracker.local_ticks, 2);
    }

    #[test]
    fn test_fleet_local_time_inherits_from_system() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SimulationTime>();
        app.add_systems(
            Update,
            (process_time_dilation_system, update_fleet_local_time_system).chain(),
        );

        let system_entity = app
            .world_mut()
            .spawn((
                SystemNode,
                TimeDilationZone { dilation_factor: 5 },
                LocalTimeTracker::default(),
            ))
            .id();

        let fleet_entity = app
            .world_mut()
            .spawn((
                Fleet,
                StationedAt(system_entity),
                LocalTimeTracker::default(),
            ))
            .id();

        for _ in 0..5 {
            app.world_mut().resource_mut::<SimulationTime>().tick += 1;
            app.update();
        }

        let fleet_tracker = app.world().get::<LocalTimeTracker>(fleet_entity).unwrap();
        assert_eq!(fleet_tracker.local_ticks, 1);
    }
}
