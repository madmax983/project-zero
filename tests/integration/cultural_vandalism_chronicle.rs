#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::core::chronicle::AddChronicleEvent;
    use scale::layer1::social::cultural_vandalism::{Defaced, Vandalized};
    use scale::layer1::core::integration::cultural_vandalism_chronicle_bridge;

    #[test]
    fn test_cultural_vandalism_chronicle_bridge_defaced() {
        let mut app = App::new();

        app.add_event::<AddChronicleEvent>();
        app.add_systems(Update, cultural_vandalism_chronicle_bridge);

        // Spawn a structure and deface it
        app.world_mut().spawn(Defaced);

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        assert!(
            !chronicle_events.is_empty(),
            "Defacing a structure should trigger a chronicle event"
        );
    }

    #[test]
    fn test_cultural_vandalism_chronicle_bridge_vandalized() {
        let mut app = App::new();

        app.add_event::<AddChronicleEvent>();
        app.add_systems(Update, cultural_vandalism_chronicle_bridge);

        // Spawn a structure and vandalize it
        app.world_mut().spawn(Vandalized);

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        assert!(
            !chronicle_events.is_empty(),
            "Vandalizing a structure should trigger a chronicle event"
        );
    }
}
