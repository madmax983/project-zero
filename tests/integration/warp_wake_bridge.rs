use bevy::prelude::*;
use scale::layer2::fleet::{Fleet, MovementSpeed};
use scale::layer2::navigation::chronological_stutter::HyperlaneTransitEvent;
use scale::layer2::navigation::warp_wake::{CurrentHyperlane, Hyperlane, generate_warp_wake_system};

#[test]
fn test_hyperlane_transit_generates_warp_wake() {
    let mut app = App::new();

    app.add_event::<HyperlaneTransitEvent>();
    app.add_systems(Update, generate_warp_wake_system);

    let hyperlane = app
        .world_mut()
        .spawn(Hyperlane {
            warp_wake_intensity: 0.0,
        })
        .id();

    let fleet = app
        .world_mut()
        .spawn((
            Fleet,
            MovementSpeed {
                base: 10.0,
                current: 10.0,
            },
            CurrentHyperlane { lane: hyperlane },
        ))
        .id();

    app.world_mut().send_event(HyperlaneTransitEvent {
        fleet,
        is_unstable: false,
    });

    app.update();

    let lane = app.world().get::<Hyperlane>(hyperlane).unwrap();
    assert!(
        lane.warp_wake_intensity > 0.0,
        "Warp wake intensity should increase when a fleet completes a jump on the hyperlane"
    );
}
