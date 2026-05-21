use bevy::prelude::*;
use scale::layer1::actions::escape::{DistressSignal, LifeboatLaunchedEvent};
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::fleet::{Fleet, InOrbit};
use scale::layer2::integration::escape_pods_integration_system;

#[test]
fn test_escape_pods_integration_system() {
    // Arrange
    let mut app = App::new();
    app.add_event::<LifeboatLaunchedEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, escape_pods_integration_system);

    let pop1 = app.world_mut().spawn_empty().id();
    let pop2 = app.world_mut().spawn_empty().id();

    // Act
    app.world_mut().send_event(LifeboatLaunchedEvent {
        occupants: vec![pop1, pop2],
    });
    app.update();

    // Assert DistressSignal appears in Layer 2
    let mut q = app.world_mut().query::<(&Fleet, &DistressSignal, &InOrbit)>();
    let signals: Vec<_> = q.iter(app.world()).collect();
    assert_eq!(signals.len(), 1, "Expected one DistressSignal fleet in orbit");
    assert_eq!(signals[0].1.occupants.len(), 2);
    assert!(signals[0].1.occupants.contains(&pop1));
    assert!(signals[0].1.occupants.contains(&pop2));

    // Assert AddChronicleEvent is sent
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut reader = chronicle_events.get_reader();
    let events: Vec<_> = reader.read(chronicle_events).collect();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0].text.contains("lifeboat has been launched"));
}
