use bevy::prelude::*;
use scale::layer1::architecture::hostage_protocol::*;
use scale::layer1::social::morale::Morale;
use scale::layer1::pop::Pop;
use scale::layer1::map::GridPosition;
use scale::layer1::unrest::{calculate_unrest_system, Unrest};
use scale::layer1::building::{BuildingType, Building};

#[test]
fn hostage_protocol_suppresses_global_unrest() {
    let mut app = App::new();

    app.insert_resource::<Time>(Time::default());
    app.insert_resource::<Unrest>(Unrest::default());

    app.add_systems(Update, (
        hostage_protocol_suppression_system,
        calculate_unrest_system,
    ).chain());

    app.world_mut().spawn((
        Building {
            building_type: BuildingType::Housing,
        },
        GridPosition { x: 0, y: 0 },
        HostageProtocol {
            is_active: true,
            suppression_power: 0.5,
            malfunction_chance: 0.01,
        },
    ));

    app.world_mut()
        .spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Morale::default(),
        ));

    app.update();

    let unrest = app.world().resource::<Unrest>();
    assert!(unrest.level < 1.0, "Global unrest should be reduced due to hostage protocol increasing morale");
}

#[test]
fn hostage_protocol_malfunction_chain() {
    let mut app = App::new();
    app.insert_resource::<Time>(Time::default());

    app.add_systems(Update, (
        hostage_protocol_malfunction_system,
        defuse_countdown_system,
        hostage_protocol_detonation_system,
    ).chain());

    let building = app.world_mut().spawn((
        Building {
            building_type: BuildingType::Housing,
        },
        GridPosition { x: 0, y: 0 },
        HostageProtocol {
            is_active: true,
            suppression_power: 0.5,
            malfunction_chance: 1.0, // High chance
        },
    )).id();

    app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(1));
    app.update();

    assert!(app.world().get::<SelfDestructCountdown>(building).is_some(), "Building should start countdown");
}
