use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent};
use scale::layer3::diplomacy::galactic_games::GalacticGamesEvent;

// We need to declare the bridge function here to test it before adding to simulation.rs
// but we'll add it to src/layer3/integration.rs later.
// We'll import it from the library once we build it in the green phase.
// For now, let's write the test so it expects the function to exist.

#[cfg(test)]
mod tests {
    use super::*;
    // Assuming the function will be placed in scale::layer3::integration
    use scale::layer3::integration::galactic_games_chronicle_bridge;
    use scale::layer1::social::factions::FactionId;
    use std::collections::HashSet;

    #[test]
    fn test_galactic_games_chronicle_bridge() {
        let mut app = App::new();

        app.add_event::<AddChronicleEvent>();
        app.add_systems(Update, galactic_games_chronicle_bridge);

        // Initially no winner
        app.insert_resource(GalacticGamesEvent {
            active: true,
            required_physical: 90.0,
            winner: None,
            participating_factions: HashSet::new(),
        });

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        assert!(
            chronicle_events.is_empty(),
            "No event should be emitted without a winner"
        );

        // Now declare a winner
        app.world_mut().resource_mut::<GalacticGamesEvent>().winner = Some(FactionId::FarmersGuild);
        app.world_mut().resource_mut::<GalacticGamesEvent>().active = false;

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        assert!(
            !chronicle_events.is_empty(),
            "Declaring a winner should trigger a chronicle event"
        );
    }
}
