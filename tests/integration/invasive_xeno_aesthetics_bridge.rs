#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::core::chronicle::AddChronicleEvent;
    use scale::layer1::core::integration::invasive_xeno_aesthetics_chronicle_bridge;
    use scale::layer1::culture::invasive_xeno_aesthetics::AestheticDeprivation;

    #[test]
    fn test_invasive_xeno_aesthetics_chronicle_bridge() {
        let mut app = App::new();

        app.add_event::<AddChronicleEvent>();
        app.add_systems(Update, invasive_xeno_aesthetics_chronicle_bridge);

        // Spawn a pop with AestheticDeprivation
        app.world_mut().spawn(AestheticDeprivation);

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        assert!(
            !chronicle_events.is_empty(),
            "Aesthetic deprivation should trigger a chronicle event"
        );
    }
}
