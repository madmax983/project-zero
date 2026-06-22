use bevy_app::App;
use bevy_ecs::prelude::*;
use scale::layer1::social::sartorial_rebellion::{
    adopt_visual_signifier_system, enforce_dress_code_system, Appearance, DressCodePolicy,
    SignifierType, FactionSignifiers
};
use scale::layer1::pop::Pop;
use scale::layer1::social::factions::{FactionId, FactionMember};
use scale::layer1::unrest::Unrest;

#[test]
fn test_sartorial_rebellion_integration() {
    let mut app = App::new();

    app.insert_resource(DressCodePolicy {
        banned_signifiers: vec![SignifierType::RedBandana],
    });

    let mut faction_signifiers = FactionSignifiers::default();
    faction_signifiers
        .map
        .insert(FactionId::MinersGuild, SignifierType::RedBandana);
    app.insert_resource(faction_signifiers);

    app.insert_resource(Unrest {
        level: 0.10,
        modifiers: vec![],
    });

    // Run enforce dress code *before* or *after* doesn't matter too much if adopt respects policy,
    // but let's just make sure both run. Actually, we should test them somewhat sequentially to be sure.
    app.add_systems(
        bevy_app::Update,
        (
            enforce_dress_code_system,     // Remove banned ones
            adopt_visual_signifier_system, // Try to adopt (will fail if banned)
        )
            .chain(),
    );

    // Spawn a pop with the banned signifier ALREADY adopted (from before the ban)
    let pop = app
        .world_mut()
        .spawn((
            Pop,
            FactionMember {
                faction_id: Some(FactionId::MinersGuild),
            },
            Appearance {
                signifiers: vec![SignifierType::RedBandana],
            },
        ))
        .id();

    app.update();

    // The signifier should be removed, but unrest should increase
    let appearance = app.world().get::<Appearance>(pop).unwrap();
    assert!(!appearance.signifiers.contains(&SignifierType::RedBandana));

    let unrest = app.world().resource::<Unrest>();
    assert!(
        unrest.level > 0.10,
        "Unrest should increase when signifier is banned"
    );
}
