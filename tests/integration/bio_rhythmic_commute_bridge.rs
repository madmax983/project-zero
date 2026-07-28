#[cfg(test)]
mod integration_tests {
    use bevy::prelude::*;
    use scale::layer1::flora::{BloomPhase, Flora, FloraType, LumifloraCycle};
    use scale::layer1::nature::solar_flare_lottery::SolarFlareEvent;
    use scale::layer2::void_leviathan::LeviathanEclipse;

    // We will write the bridge in layer1/core/integration.rs
    use scale::layer1::core::integration::{
        bio_rhythmic_commute_eclipse_bridge, bio_rhythmic_commute_flare_bridge,
    };

    #[test]
    fn test_eclipse_forces_hibernation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(LeviathanEclipse { active: true });

        app.add_systems(Update, bio_rhythmic_commute_eclipse_bridge);

        let flora = app
            .world_mut()
            .spawn((
                Flora {
                    flora_type: FloraType::Lumiflora,
                    ..Default::default()
                },
                LumifloraCycle {
                    phase: BloomPhase::Blooming,
                    time_in_phase: 10.0,
                },
            ))
            .id();

        app.update();

        let cycle = app.world().get::<LumifloraCycle>(flora).unwrap();
        assert_eq!(
            cycle.phase,
            BloomPhase::Hibernation,
            "Eclipse should force hibernation"
        );
    }

    #[test]
    fn test_flare_forces_hibernation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<SolarFlareEvent>();

        app.add_systems(Update, bio_rhythmic_commute_flare_bridge);

        let flora = app
            .world_mut()
            .spawn((
                Flora {
                    flora_type: FloraType::Lumiflora,
                    ..Default::default()
                },
                LumifloraCycle {
                    phase: BloomPhase::Blooming,
                    time_in_phase: 10.0,
                },
            ))
            .id();

        app.world_mut().send_event(SolarFlareEvent);

        app.update();

        let cycle = app.world().get::<LumifloraCycle>(flora).unwrap();
        assert_eq!(
            cycle.phase,
            BloomPhase::Hibernation,
            "Solar flare should force hibernation"
        );
    }
}
