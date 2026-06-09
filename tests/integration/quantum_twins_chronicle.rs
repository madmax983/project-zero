use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent};
use scale::layer1::tech::cognitive_overclocking::ActiveState;

#[cfg(test)]
mod tests {
    use super::*;
    use scale::layer1::core::integration::quantum_twins_severance_chronicle_bridge;

    #[test]
    fn test_quantum_twins_severance_chronicle_bridge() {
        let mut app = App::new();

        app.add_event::<AddChronicleEvent>();
        app.add_systems(Update, quantum_twins_severance_chronicle_bridge);

        // A pop not in catatonic state
        let pop_normal = app.world_mut().spawn(ActiveState::Working).id();

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        assert!(
            chronicle_events.is_empty(),
            "No event should be emitted for non-catatonic pop"
        );

        app.world_mut().get_mut::<ActiveState>(pop_normal).map(|mut state| *state = ActiveState::Catatonic);

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        assert!(
            !chronicle_events.is_empty(),
            "Catatonic state should trigger a chronicle event"
        );
    }
}
