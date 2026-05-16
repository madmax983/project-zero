use bevy_ecs::prelude::*;
use crate::layer1::psychology::needs::Needs;

#[derive(Resource)]
pub struct SleepDebtConfig {
    pub critical_threshold: f32,
    pub coma_duration: f32,
}

impl Default for SleepDebtConfig {
    fn default() -> Self {
        Self {
            critical_threshold: 500.0,
            coma_duration: 2160.0,
        }
    }
}

#[derive(Component)]
pub struct SleepDebt {
    pub hours: f32,
    pub active_contract: bool,
}

#[derive(Event)]
pub struct RepoManArrivalEvent {
    pub target_pop: Entity,
}

#[derive(Component)]
pub struct RepoMan {
    pub target: Entity,
}

#[derive(Component)]
pub struct ForcedComa {
    pub duration_remaining: f32,
}

#[derive(Component, Default)]
pub struct RepoManSpawned;

#[allow(clippy::needless_pass_by_value)]
pub fn process_sleep_debt_system(
    time: Res<bevy_time::Time<bevy_time::Virtual>>,
    mut query: Query<(&mut Needs, &mut SleepDebt)>,
) {
    let dt = time.delta_secs() / 3600.0; // Simulated hours
    for (mut needs, mut debt) in query.iter_mut() {
        if debt.active_contract {
            // Prevent rest decay
            needs.rest = 100.0;
            // Accrue debt
            debt.hours += dt;
        }
    }
}

pub fn check_critical_sleep_debt_system(
    mut commands: Commands,
    config: Res<SleepDebtConfig>,
    query: Query<(Entity, &SleepDebt), Without<RepoManSpawned>>,
    mut arrival_events: EventWriter<RepoManArrivalEvent>,
) {
    for (entity, debt) in query.iter() {
        if debt.hours >= config.critical_threshold {
            arrival_events.send(RepoManArrivalEvent { target_pop: entity });
            commands.entity(entity).insert(RepoManSpawned);
        }
    }
}

pub fn spawn_repo_man_system(
    mut commands: Commands,
    mut events: EventReader<RepoManArrivalEvent>,
) {
    for event in events.read() {
        commands.spawn(RepoMan { target: event.target_pop });
    }
}

pub fn process_repo_men_action_system(
    mut commands: Commands,
    config: Res<SleepDebtConfig>,
    repo_query: Query<(Entity, &RepoMan)>,
    pop_query: Query<Entity, With<SleepDebt>>,
) {
    for (repo_entity, repo_man) in repo_query.iter() {
        if let Ok(target_pop) = pop_query.get(repo_man.target) {
            commands.entity(target_pop).insert(ForcedComa { duration_remaining: config.coma_duration });
            commands.entity(repo_entity).despawn();
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn tick_forced_coma_system(
    mut commands: Commands,
    time: Res<bevy_time::Time<bevy_time::Virtual>>,
    mut query: Query<(Entity, &mut ForcedComa, &mut SleepDebt)>,
) {
    let dt = time.delta_secs() / 3600.0; // Assume time flows in simulated hours like in process_sleep_debt_system
    for (entity, mut coma, mut debt) in query.iter_mut() {
        coma.duration_remaining -= dt;
        if coma.duration_remaining <= 0.0 {
            commands.entity(entity).remove::<ForcedComa>();
            commands.entity(entity).remove::<RepoManSpawned>();
            debt.hours = 0.0; // Debt is reclaimed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use bevy_app::App;
    use bevy_app::Update;
    use bevy_ecs::event::Events;
    use bevy_time::Time;

    #[test]
    fn test_wakefulness_stim_prevents_rest_decay_and_accrues_debt() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_sleep_debt_system);
        app.insert_resource(Time::<bevy_time::Virtual>::default());

        let pop = app.world_mut().spawn((
            Pop,
            Needs { rest: 100.0, ..Default::default() },
            SleepDebt { hours: 0.0, active_contract: true },
        )).id();

        // Act
        // Simulate time passing (1 hour)
        app.world_mut().resource_mut::<Time::<bevy_time::Virtual>>().advance_by(std::time::Duration::from_secs_f32(3600.0));
        app.update();

        // Assert
        let debt = app.world().get::<SleepDebt>(pop).unwrap();
        let needs = app.world().get::<Needs>(pop).unwrap();

        // Rest should not decay, but debt should increase
        assert_eq!(needs.rest, 100.0);
        assert!(debt.hours > 0.0);
    }

    #[test]
    fn test_critical_sleep_debt_triggers_repo_event() {
        let mut app = App::new();
        app.add_event::<RepoManArrivalEvent>();
        app.init_resource::<SleepDebtConfig>();
        app.add_systems(Update, check_critical_sleep_debt_system);

        let pop = app.world_mut().spawn((
            Pop,
            SleepDebt { hours: 500.0, active_contract: true }, // High debt
        )).id();

        app.update();

        let repo_events = app.world().resource::<Events<RepoManArrivalEvent>>();
        let mut reader = repo_events.get_cursor();
        assert!(reader.read(repo_events).len() > 0, "Critical debt should spawn repo men");
        assert!(app.world().get::<RepoManSpawned>(pop).is_some());
    }

    #[test]
    fn test_repo_men_apply_coma_state() {
        let mut app = App::new();
        app.init_resource::<SleepDebtConfig>();
        app.add_systems(Update, process_repo_men_action_system);

        let pop = app.world_mut().spawn((
            Pop,
            SleepDebt { hours: 500.0, active_contract: true },
        )).id();

        let _repo_man = app.world_mut().spawn(RepoMan { target: pop }).id();

        app.update();

        // Target should have ForcedComa component
        assert!(app.world().get::<ForcedComa>(pop).is_some());
    }
}
