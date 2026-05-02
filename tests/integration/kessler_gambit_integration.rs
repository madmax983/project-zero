#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer2::debris::OrbitalDebris;
    use scale::layer2::fleet::{Fleet, InOrbit};
    use scale::layer2::orbit::kessler_gambit::{TriggerKesslerGambitEvent, trigger_kessler_gambit_system};
    use scale::layer2::station::{Station, StationType};

    #[test]
    fn test_trigger_kessler_gambit_integration() {
        let mut app = App::new();
        app.add_event::<TriggerKesslerGambitEvent>();

        let planet = app.world_mut().spawn(OrbitalDebris(0.0)).id();

        app.world_mut().spawn((
            Station {
                station_type: StationType::Outpost,
            },
            InOrbit { parent: planet },
        ));
        app.world_mut().spawn((Fleet, InOrbit { parent: planet }));

        app.add_systems(Update, trigger_kessler_gambit_system);

        app.world_mut().send_event(TriggerKesslerGambitEvent { planet });
        app.update();

        let debris = app.world().get::<OrbitalDebris>(planet).unwrap();
        assert!(debris.0 > 0.0);
    }
}
