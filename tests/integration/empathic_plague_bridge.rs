use bevy::prelude::*;
use scale::layer1::needs::Needs;
use scale::layer1::pop::Pop;
use scale::layer1::social::morale::Morale;
use scale::layer2::empathic_plague::{process_empathic_resonance, EmpathicInfection};

#[test]
fn empathic_plague_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_systems(Update, process_empathic_resonance);

    let infected_morale = Morale {
        value: 1.0,
        ..Default::default()
    };

    let infected_pop = app
        .world_mut()
        .spawn((
            Pop,
            Transform::from_xyz(0.0, 0.0, 0.0),
            EmpathicInfection,
            infected_morale,
        ))
        .id();

    let miserable_morale = Morale {
        value: 0.2,
        ..Default::default()
    };
    let miserable_needs = Needs {
        hunger: 0.1,
        ..Default::default()
    };

    app.world_mut().spawn((
        Pop,
        Transform::from_xyz(1.0, 0.0, 0.0),
        miserable_needs,
        miserable_morale,
    ));

    app.update();

    let morale = app.world().get::<Morale>(infected_pop).unwrap();

    assert!(
        morale.modifiers.iter().any(|m| m.value < 0.0),
        "Infected pop should have received a negative mood modifier from Empathic Resonance"
    );
}
