use crate::layer1::economy::resources::ColonyResources;
use bevy_ecs::prelude::*;
use bevy_time::Time;

#[derive(Event, Clone, Debug)]
pub struct AcceptRetroContractEvent {
    pub credit_advance: f32,
    pub deadline_days: f32,
}

/// Event emitted when a retro-causality contract is failed.
/// Integrators: Listen to this to apply penalties or chronicle events.
#[derive(Event, Clone, Debug)]
pub struct RetroContractFailedEvent {
    pub penalty: f32,
}

#[derive(Component)]
pub struct RetroContract {
    pub days_remaining: f32,
    pub penalty: f32,
    pub is_fulfilled: bool,
}

pub fn handle_retro_contract_acceptance(
    mut commands: Commands,
    mut events: EventReader<AcceptRetroContractEvent>,
    mut resources: ResMut<ColonyResources>,
) {
    for event in events.read() {
        resources.credits += event.credit_advance;

        commands.spawn(RetroContract {
            days_remaining: event.deadline_days,
            penalty: event.credit_advance * 2.0,
            is_fulfilled: false,
        });
    }
}

pub fn evaluate_retro_contracts(
    mut commands: Commands,
    time: Res<Time>,
    mut contracts: Query<(Entity, &mut RetroContract)>,
    mut resources: ResMut<ColonyResources>,
    mut failed_events: EventWriter<RetroContractFailedEvent>,
) {
    let day_delta = time.delta_secs() / 100.0;

    for (entity, mut contract) in contracts.iter_mut() {
        if contract.is_fulfilled {
            commands.entity(entity).despawn();
            continue;
        }

        contract.days_remaining -= day_delta;

        if contract.days_remaining <= 0.0 {
            resources.credits -= contract.penalty;
            failed_events.send(RetroContractFailedEvent {
                penalty: contract.penalty,
            });
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]

#[test]
fn test_accepting_retro_contract_grants_immediate_resources() {
    let mut app = bevy_app::App::new();
    app.insert_resource(ColonyResources {
        credits: 0.0,
        ..ColonyResources::zeroed()
    });
    app.add_event::<AcceptRetroContractEvent>();
    app.add_systems(bevy_app::Update, handle_retro_contract_acceptance);

    app.world_mut().send_event(AcceptRetroContractEvent {
        credit_advance: 1000.0,
        deadline_days: 10.0,
    });
    app.update();

    let resources = app.world().resource::<ColonyResources>();
    assert_eq!(resources.credits, 1000.0);

    let contracts = app
        .world_mut()
        .query::<&RetroContract>()
        .iter(app.world())
        .count();
    assert_eq!(contracts, 1);
}

#[test]
fn test_failing_retro_contract_applies_penalty() {
    let mut app = bevy_app::App::new();
    app.insert_resource(bevy_time::Time::<()>::default());
    app.insert_resource(ColonyResources {
        credits: 500.0,
        ..ColonyResources::zeroed()
    });
    app.add_event::<RetroContractFailedEvent>();
    app.add_systems(bevy_app::Update, evaluate_retro_contracts);

    app.world_mut().spawn(RetroContract {
        days_remaining: 1.0,
        penalty: 2000.0,
        is_fulfilled: false,
    });

    let mut time_resource = app.world_mut().resource_mut::<bevy_time::Time<()>>();
    // Advance time past the deadline. Let's advance it by 100.0s = 1.0 day in MVP implementation
    time_resource.advance_by(std::time::Duration::from_secs_f32(100.1));

    app.update();

    let resources = app.world().resource::<ColonyResources>();
    assert_eq!(resources.credits, -1500.0); // 500 - 2000

    // contract should be despawned
    let contracts = app
        .world_mut()
        .query::<&RetroContract>()
        .iter(app.world())
        .count();
    assert_eq!(contracts, 0);
}

#[test]
fn test_fulfilled_retro_contract_despawns_without_penalty() {
    let mut app = bevy_app::App::new();
    app.insert_resource(bevy_time::Time::<()>::default());
    app.insert_resource(ColonyResources {
        credits: 500.0,
        ..ColonyResources::zeroed()
    });
    app.add_event::<RetroContractFailedEvent>();
    app.add_systems(bevy_app::Update, evaluate_retro_contracts);

    app.world_mut().spawn(RetroContract {
        days_remaining: 1.0,
        penalty: 2000.0,
        is_fulfilled: true,
    });

    let mut time_resource = app.world_mut().resource_mut::<bevy_time::Time<()>>();
    // Advance time past the deadline. Let's advance it by 100.0s = 1.0 day in MVP implementation
    time_resource.advance_by(std::time::Duration::from_secs_f32(100.1));

    app.update();

    // Verify credits haven't decreased
    let resources = app.world().resource::<ColonyResources>();
    assert_eq!(resources.credits, 500.0);

    // Verify contract is despawned
    let contracts = app
        .world_mut()
        .query::<&RetroContract>()
        .iter(app.world())
        .count();
    assert_eq!(contracts, 0);
}
