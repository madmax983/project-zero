use bevy::prelude::*;
use bevy::utils::HashSet;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::social::factions::FactionId;
use scale::layer3::diplomacy::galactic_games::{
    resolve_galactic_games_system, FactionInfluences, GalacticGamesEvent,
};

#[test]
fn test_galactic_games_chronicle_bridge() {
    let mut app = App::new();

    // Register systems - we will chain a bridge system after resolve_galactic_games_system
    app.add_systems(
        Update,
        (
            resolve_galactic_games_system,
            scale::layer3::integration::galactic_games_chronicle_bridge,
        )
            .chain(),
    );

    app.add_event::<AddChronicleEvent>();

    // Mock dependencies for GalacticGamesEvent
    let mut influences = FactionInfluences::default();
    influences.map.insert(FactionId::FarmersGuild, 0);
    app.insert_resource(influences);

    app.insert_resource(GalacticGamesEvent {
        active: true,
        required_physical: 0.0,
        winner: Some(FactionId::FarmersGuild), // We simulate it being resolved in previous frame, or force a winner
        participating_factions: HashSet::new(),
    });

    // Act
    app.update();

    // Assert
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = chronicle_events.get_cursor();
    let events: Vec<_> = cursor.read(chronicle_events).collect();

    assert_eq!(
        events.len(),
        1,
        "Should emit one Chronicle event for the winner"
    );
    assert!(events[0].text.contains("Galactic Games"));
}
