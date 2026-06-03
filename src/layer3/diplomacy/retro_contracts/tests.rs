use super::*;
use crate::layer1::economy::resources::ColonyResources;

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
