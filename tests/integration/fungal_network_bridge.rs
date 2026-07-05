use bevy::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::fungal_network::fungal_network_chronicle_bridge;
use scale::layer1::fungal_network::SporeTap;

#[test]
fn test_fungal_network_wakes_up_chronicle_event() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, fungal_network_chronicle_bridge);

    // Initial tick - no taps, no events
    app.update();

    {
        let events = app.world().resource::<Events<AddChronicleEvent>>();
        assert_eq!(events.len(), 0, "No events should be emitted initially");
    }

    // Spawn first tap
    app.world_mut().spawn(SporeTap);
    app.update();

    {
        let events = app.world().resource::<Events<AddChronicleEvent>>();
        assert_eq!(
            events.len(),
            1,
            "Should emit one AddChronicleEvent when first tap is built"
        );

        // Read the event to check content
        let mut cursor = events.get_cursor();
        let emitted: Vec<&AddChronicleEvent> = cursor.read(events).collect();
        assert!(
            emitted[0].text.contains("mycelial network")
                || emitted[0].text.contains("Fungal Network")
                || emitted[0].text.contains("wakes up"),
            "Event text should mention the network waking up"
        );
    }

    // Clear events
    app.world_mut()
        .resource_mut::<Events<AddChronicleEvent>>()
        .clear();

    // Spawn second tap
    app.world_mut().spawn(SporeTap);
    app.update();

    {
        let events = app.world().resource::<Events<AddChronicleEvent>>();
        assert_eq!(
            events.len(),
            0,
            "Should NOT emit another AddChronicleEvent for subsequent taps"
        );
    }
}
