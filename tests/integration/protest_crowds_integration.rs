#[cfg(test)]
mod integration_tests {
    use bevy::prelude::*;
    use scale::layer1::core::integration::faction_strike_mob_bridge_system;
    use scale::layer1::factions::{FactionData, FactionId, FactionState, Factions};
    use scale::layer1::social::protest_crowds::{DisperseMobEvent, FormMobEvent, Mob};

    #[test]
    fn test_faction_strike_forms_mob() {
        let mut app = App::new();

        app.add_event::<FormMobEvent>();
        app.add_event::<DisperseMobEvent>();

        let mut factions = Factions::default();
        factions.map.insert(
            FactionId::MinersGuild,
            FactionData {
                state: FactionState::Loyal,
                ..Default::default()
            },
        );
        app.insert_resource(factions);

        app.add_systems(Update, faction_strike_mob_bridge_system);

        // Run once with Loyal state to initialize Local
        app.update();

        // Change state to Striking
        app.world_mut()
            .resource_mut::<Factions>()
            .map
            .get_mut(&FactionId::MinersGuild)
            .unwrap()
            .state = FactionState::Striking;

        // Run system again
        app.update();

        // Assert FormMobEvent was sent
        let form_events = app.world().resource::<Events<FormMobEvent>>();
        let mut reader = form_events.get_cursor();
        let events: Vec<_> = reader.read(form_events).collect();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].faction, FactionId::MinersGuild);
        assert_eq!(events[0].location, (0, 0));
    }

    #[test]
    fn test_faction_stop_strike_disperses_mob() {
        let mut app = App::new();

        app.add_event::<FormMobEvent>();
        app.add_event::<DisperseMobEvent>();

        let mut factions = Factions::default();
        factions.map.insert(
            FactionId::MinersGuild,
            FactionData {
                state: FactionState::Striking,
                ..Default::default()
            },
        );
        app.insert_resource(factions);

        // Add a mock mob entity
        let mob_entity = app
            .world_mut()
            .spawn(Mob {
                location: (0, 0),
                faction: FactionId::MinersGuild,
            })
            .id();

        app.add_systems(Update, faction_strike_mob_bridge_system);

        // Run once with Striking state to initialize Local
        app.update();

        // Change state to Loyal
        app.world_mut()
            .resource_mut::<Factions>()
            .map
            .get_mut(&FactionId::MinersGuild)
            .unwrap()
            .state = FactionState::Loyal;

        // Run system again
        app.update();

        // Assert DisperseMobEvent was sent
        let disperse_events = app.world().resource::<Events<DisperseMobEvent>>();
        let mut reader = disperse_events.get_cursor();
        let events: Vec<_> = reader.read(disperse_events).collect();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].mob, mob_entity);
    }
}
