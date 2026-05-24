use bevy_app::App;
use scale::layer2::phantom_signal::{
    apply_sensor_probes_system, process_phantom_signal_evasion_system,
    PhantomSignal, SensorProbe,
};
use scale::layer2::fleet::{Fleet, InOrbit};
use scale::layer2::system::SystemBody;

#[test]
fn test_phantom_signal_evades() {
    let mut app = App::new();
    app.add_systems(bevy_app::Update, process_phantom_signal_evasion_system);

    let node1 = app.world_mut().spawn(SystemBody).id();
    let node2 = app.world_mut().spawn(SystemBody).id();
    let _fleet = app.world_mut().spawn((Fleet, InOrbit { parent: node1 })).id();
    let signal = app
        .world_mut()
        .spawn((PhantomSignal { pinned: false }, InOrbit { parent: node1 }))
        .id();

    app.update();

    let signal_orbit = app.world().get::<InOrbit>(signal).unwrap();
    assert_ne!(signal_orbit.parent, node1, "Signal should evade fleet");
    assert_eq!(signal_orbit.parent, node2, "Signal should move to available node");
}

#[test]
fn test_phantom_signal_pinned() {
    let mut app = App::new();
    app.add_systems(bevy_app::Update, apply_sensor_probes_system);

    let node1 = app.world_mut().spawn(SystemBody).id();
    let signal = app
        .world_mut()
        .spawn((PhantomSignal { pinned: false }, InOrbit { parent: node1 }))
        .id();
    app.world_mut().spawn((SensorProbe, InOrbit { parent: node1 }));

    app.update();

    let signal_comp = app.world().get::<PhantomSignal>(signal).unwrap();
    assert!(signal_comp.pinned, "Signal should be pinned by probe");
}
