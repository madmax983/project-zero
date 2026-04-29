#![cfg(test)]

use bevy_app::App;
use bevy_ecs::prelude::*;
use scale::layer1::culture::astrology::{
    astrological_buff_system, AstrologicalBelief, Productivity,
};
use scale::layer2::integration::astrological_beliefs_bridge_system;
use scale::layer2::syzygy::{update_syzygy_cycle_system, SyzygyCycle, SyzygyPhase};
use scale::shared::time::SimulationTime;

#[test]
fn test_astrological_beliefs_syzygy_integration() {
    let mut app = App::new();

    app.insert_resource(SimulationTime {
        tick: 1000, // Trigger phase active
        ..Default::default()
    });

    app.insert_resource(SyzygyCycle {
        current_phase: SyzygyPhase::Inactive,
        next_syzygy_tick: 1000,
    });

    let entity = app
        .world_mut()
        .spawn((
            AstrologicalBelief {
                lucky_alignment: false,
                unlucky_alignment: true, // start "retrograde"
            },
            Productivity { multiplier: 1.0 },
        ))
        .id();

    app.add_systems(
        bevy_app::Update,
        (
            update_syzygy_cycle_system,
            astrological_beliefs_bridge_system,
            astrological_buff_system,
        )
            .chain(),
    );

    app.update();

    let cycle = app.world().resource::<SyzygyCycle>();
    assert_eq!(
        cycle.current_phase,
        SyzygyPhase::Active,
        "Syzygy should be Active"
    );

    let belief = app.world().get::<AstrologicalBelief>(entity).unwrap();
    assert!(belief.lucky_alignment, "Belief should be lucky alignment");
    assert!(
        !belief.unlucky_alignment,
        "Belief should not be unlucky alignment"
    );

    let productivity = app.world().get::<Productivity>(entity).unwrap();
    assert_eq!(
        productivity.multiplier, 1.5,
        "Productivity should be boosted"
    );
}
