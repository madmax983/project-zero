use bevy::prelude::*;
use scale::layer2::communications::radio_broadcasts::SystemThreat;
use scale::layer2::integration::radio_broadcasts_threat_bridge_system;
use scale::layer3::pirates::PirateThreatLevel;

#[test]
fn test_radio_broadcasts_increases_pirate_threat() {
    let mut app = App::new();
    app.add_systems(Update, radio_broadcasts_threat_bridge_system);

    app.world_mut().spawn(SystemThreat { level: 5.0 });
    app.init_resource::<PirateThreatLevel>();
    app.world_mut().resource_mut::<PirateThreatLevel>().level = 0.0;

    app.update();

    let threat = app.world().resource::<PirateThreatLevel>().level;
    assert!(
        threat > 0.0,
        "SystemThreat should leak into global PirateThreatLevel"
    );
}
