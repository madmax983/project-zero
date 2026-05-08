use bevy::prelude::*;
use scale::layer1::law::penal::OrganHarvestedEvent;
use scale::layer3::diplomacy_reflection::apply_diplomatic_reactions;
use scale::layer3::diplomacy_reflection::{
    Civilization, DiplomaticRelations, DiplomaticStanding, DiplomaticTraits,
};
use scale::layer3::integration::organ_trade_diplomacy_bridge;

#[test]
fn test_organ_harvesting_causes_embargo() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);

    app.add_event::<OrganHarvestedEvent>();
    app.add_event::<scale::layer3::diplomacy_reflection::TraitChangedEvent>();
    app.add_systems(
        Update,
        (organ_trade_diplomacy_bridge, apply_diplomatic_reactions).chain(),
    );

    // Setup Player Civ
    let player_civ = app
        .world_mut()
        .spawn((
            Civilization {
                id: "player".to_string(),
            },
            DiplomaticTraits::default(),
        ))
        .id();

    // Setup Pacifist Neighbor
    let neighbor = app
        .world_mut()
        .spawn((
            DiplomaticTraits {
                is_pacifist: true,
                ..default()
            },
            DiplomaticRelations {
                relations: vec![DiplomaticStanding {
                    target_id: "player".to_string(),
                    standing: 0.0,
                    sanctioned: false,
                }],
            },
        ))
        .id();

    // Trigger harvest
    app.world_mut().send_event(OrganHarvestedEvent);

    app.update();
    app.update();

    let relations = app.world().get::<DiplomaticRelations>(neighbor).unwrap();
    assert!(
        relations.relations[0].sanctioned,
        "Pacifist neighbor should sanction the player after organ harvesting"
    );
    assert!(
        relations.relations[0].standing < 0.0,
        "Standing should decrease"
    );

    let traits = app.world().get::<DiplomaticTraits>(player_civ).unwrap();
    assert!(traits.is_barbarian, "Player should be marked as barbarian");
}
