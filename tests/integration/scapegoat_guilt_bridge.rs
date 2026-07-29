use bevy::prelude::*;
use scale::layer1::pop::Pop;
use scale::layer1::traits::{Trait, Traits};
use scale::layer1::unrest::{
    handle_denounce_event_system, DenounceEvent, MentalBreakType, MentalState, ScapegoatAction,
    ScapegoatTarget, Unrest,
};

#[test]
fn test_scapegoat_guilt_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<DenounceEvent>();
    app.insert_resource(Unrest {
        level: 0.9,
        ..Default::default()
    });

    app.add_systems(Update, handle_denounce_event_system);

    let target = app
        .world_mut()
        .spawn((Pop, Traits::default(), ScapegoatTarget, MentalState::Normal))
        .id();

    let bystander = app.world_mut().spawn((Pop, Traits::default())).id();

    // Send Denounce event (PublicShame)
    app.world_mut().send_event(DenounceEvent {
        target,
        action: ScapegoatAction::PublicShame,
    });

    app.update();

    let state = app.world().get::<MentalState>(target).unwrap();
    assert!(
        matches!(state, MentalState::Broken(MentalBreakType::Daze)),
        "Target should be broken"
    );

    assert!(
        app.world()
            .get::<Traits>(bystander)
            .unwrap()
            .has(Trait::Guilt),
        "Bystander pop should receive the Guilt trait"
    );
}
