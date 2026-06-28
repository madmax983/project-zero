use bevy::prelude::*;
use scale::layer1::social::slippery_slope::{
    apply_morale_buffs, apply_stress_penalties, process_atrocities, AtrocityEvent, Desensitization,
    MoraleBuffEvent, StressPenaltyEvent,
};
use scale::layer1::social::morale::Morale;
use scale::layer1::needs::Needs;
use scale::layer1::pop::Pop;

#[test]
fn test_atrocity_increases_desensitization() {
    let mut app = App::new();
    app.add_systems(Update, process_atrocities);

    app.world_mut().insert_resource(Desensitization { level: 0.0 });
    app.world_mut().init_resource::<Events<AtrocityEvent>>();

    app.world_mut().send_event(AtrocityEvent { severity: 10.0 });

    app.update();

    let d_level = app.world().resource::<Desensitization>().level;
    assert!(d_level > 0.0);
}

#[test]
fn test_high_desensitization_dampens_morale_buffs() {
    let mut app = App::new();
    app.add_systems(Update, apply_morale_buffs);

    app.world_mut().insert_resource(Desensitization { level: 1.0 }); // 100% numb
    app.world_mut().init_resource::<Events<MoraleBuffEvent>>();

    let pop = app
        .world_mut()
        .spawn((
            Pop,
            Morale {
                value: 50.0,
                ..Default::default()
            },
        ))
        .id();

    app.world_mut().send_event(MoraleBuffEvent {
        target: pop,
        amount: 50.0,
    });

    app.update();

    let morale = app.world().get::<Morale>(pop).unwrap();
    // The buff should be heavily dampened
    assert!(morale.value < 100.0);
    assert_eq!(morale.value, 50.0); // 100% dampening means no effect
}

#[test]
fn test_high_desensitization_dampens_stress_penalties() {
    let mut app = App::new();
    app.add_systems(Update, apply_stress_penalties);

    // Colony is numb
    app.world_mut()
        .insert_resource(Desensitization { level: 1.0 }); // 100% numb
    app.world_mut()
        .init_resource::<Events<StressPenaltyEvent>>();

    let pop = app
        .world_mut()
        .spawn((
            Pop,
            Needs {
                leisure: 100.0,
                ..Default::default()
            },
        ))
        .id();

    // Send a massive stress penalty
    app.world_mut().send_event(StressPenaltyEvent {
        target: pop,
        amount: 50.0,
    });

    app.update();

    let needs = app.world().get::<Needs>(pop).unwrap();
    // The penalty should be heavily dampened
    assert_eq!(needs.leisure, 100.0); // 100% dampening means no stress effect
}
