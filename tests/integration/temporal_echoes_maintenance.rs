#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::anomalies::temporal_echoes::BuildingAge;
    use scale::layer1::architecture::structure::Structure;
    use scale::layer1::anomalies::temporal_echoes::temporal_echo_maintenance_bridge_system;

    #[test]
    fn test_temporal_echo_rapid_aging() {
        let mut app = bevy_app::App::new();
        app.add_plugins(MinimalPlugins);

        app.add_systems(Update, temporal_echo_maintenance_bridge_system);

        let building = app
            .world_mut()
            .spawn((
                BuildingAge { ticks: 1000 },
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        app.update();

        let structure = app.world().get::<Structure>(building).unwrap();
        let age = app.world().get::<BuildingAge>(building).unwrap();

        assert_eq!(age.ticks, 0, "Ticks should be consumed");
        assert!(
            structure.current_hp < 100.0,
            "Building should have taken damage from rapid aging"
        );
        assert_eq!(structure.current_hp, 90.0, "1000 ticks * 0.01 = 10 damage");
    }
}
