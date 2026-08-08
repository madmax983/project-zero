#[cfg(test)]
mod tests {
    use bevy_app::{App, Update};
    use bevy_ecs::prelude::*;
    use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
    use scale::cross_layer::tourism::{FactionMarker, CasusBelli};
    // Include the actual function we wrote
    use scale::layer3::tourism_integration::weaponized_tourism_chronicle_bridge;

    #[test]
    fn test_weaponized_tourism_chronicle_bridge() {
        let mut app = App::new();

        app.add_event::<AddChronicleEvent>();

        app.add_systems(Update, weaponized_tourism_chronicle_bridge);

        let _origin_faction = app.world_mut().spawn((
            FactionMarker,
            CasusBelli {
                target: Entity::PLACEHOLDER,
                reason: "Tourist Harmed".to_string(),
            },
        )).id();

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut cursor = chronicle_events.get_cursor();
        let events: Vec<_> = cursor.read(chronicle_events).collect();

        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].text,
            "A diplomatic crisis has erupted: Casus Belli declared due to a harmed tourist."
        );
        assert_eq!(events[0].importance, EventImportance::Major);
    }
}
