#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::architecture::{Building, BuildingType};
    use scale::layer1::core::chronicle::AddChronicleEvent;
    use scale::layer1::core::integration::rogue_cult_chronicle_bridge;
    use scale::layer1::tech::machine_awakening::Bot;
    use scale::layer1::tech::rogue_automation_cults::{
        rogue_cult_formation_system, MachineCultFormedEvent, MaintenanceDebt,
    };

    #[test]
    fn test_rogue_cult_formation_chronicle_bridge() {
        let mut app = App::new();
        app.add_event::<AddChronicleEvent>();
        app.add_event::<MachineCultFormedEvent>();

        app.add_systems(
            Update,
            (rogue_cult_formation_system, rogue_cult_chronicle_bridge).chain(),
        );

        let _shrine = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Mainframe,
                },
                MaintenanceDebt {
                    current: 150.0,
                    threshold: 100.0,
                },
            ))
            .id();

        let _bot = app
            .world_mut()
            .spawn((
                Bot,
                MaintenanceDebt {
                    current: 120.0,
                    threshold: 100.0,
                },
            ))
            .id();

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = chronicle_events.get_cursor();
        let events: Vec<_> = reader.read(chronicle_events).collect();
        assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
        assert!(events[0].text.contains("Machine Cult"));
    }
}
