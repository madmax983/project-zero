use crate::layer1::needs::Needs;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct SleepDebtConfig {
    pub critical_threshold: f32,
    pub coma_duration_ticks: f32,
}

impl Default for SleepDebtConfig {
    fn default() -> Self {
        Self {
            critical_threshold: 500.0,
            coma_duration_ticks: 2160.0,
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

pub fn process_sleep_debt_system(
    _time: Res<SimulationTime>,
    mut query: Query<(&mut Needs, &mut SleepDebt)>,
) {
    let dt = 1.0;
    for (mut needs, mut debt) in query.iter_mut() {
        if debt.active_contract {
            needs.rest = 100.0;
            debt.hours += dt;
        }
    }
}

pub fn check_critical_sleep_debt_system(
    query: Query<(Entity, &SleepDebt)>,
    config: Option<Res<SleepDebtConfig>>,
    mut arrival_events: EventWriter<RepoManArrivalEvent>,
) {
    let threshold = config.map_or(500.0, |c| c.critical_threshold);
    for (entity, debt) in query.iter() {
        if debt.hours >= threshold {
            arrival_events.send(RepoManArrivalEvent { target_pop: entity });
        }
    }
}

pub fn process_repo_men_action_system(
    mut commands: Commands,
    repo_query: Query<(Entity, &RepoMan)>,
    pop_query: Query<Entity, With<SleepDebt>>,
    config: Option<Res<SleepDebtConfig>>,
) {
    let duration = config.map_or(2160.0, |c| c.coma_duration_ticks);
    for (repo_entity, repo_man) in repo_query.iter() {
        if let Ok(target_pop) = pop_query.get(repo_man.target) {
            commands.entity(target_pop).insert(ForcedComa {
                duration_remaining: duration,
            });
            commands.entity(repo_entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_wakefulness_stim_prevents_rest_decay_and_accrues_debt() {
        // Arrange
        let mut app = bevy_app::App::new();
        app.insert_resource(SimulationTime {
            tick: 0,
            speed: crate::shared::time::SimSpeed::Normal,
        });
        app.add_systems(bevy_app::Update, process_sleep_debt_system);

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Needs {
                    rest: 100.0,
                    ..Default::default()
                },
                SleepDebt {
                    hours: 0.0,
                    active_contract: true,
                },
            ))
            .id();

        // Act
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
        let mut app = bevy_app::App::new();
        app.add_event::<RepoManArrivalEvent>();
        app.init_resource::<SleepDebtConfig>();
        app.add_systems(bevy_app::Update, check_critical_sleep_debt_system);

        app.world_mut().spawn((
            Pop,
            SleepDebt {
                hours: 500.0,
                active_contract: true,
            }, // High debt
        ));

        app.update();

        let repo_events = app.world().resource::<Events<RepoManArrivalEvent>>();
        let mut reader = repo_events.get_cursor();
        assert!(
            reader.read(repo_events).len() > 0,
            "Critical debt should spawn repo men"
        );
    }

    #[test]
    fn test_repo_men_apply_coma_state() {
        let mut app = bevy_app::App::new();
        app.init_resource::<SleepDebtConfig>();
        app.add_systems(bevy_app::Update, process_repo_men_action_system);

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                SleepDebt {
                    hours: 500.0,
                    active_contract: true,
                },
            ))
            .id();

        let _repo_man = app.world_mut().spawn(RepoMan { target: pop }).id();

        app.update();

        // Target should have ForcedComa component
        assert!(app.world().get::<ForcedComa>(pop).is_some());
    }
}
