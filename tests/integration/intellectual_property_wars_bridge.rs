use bevy::prelude::*;
use scale::layer3::diplomacy_reflection::{
    apply_diplomatic_reactions, Civilization, DiplomaticRelations, DiplomaticStanding,
    DiplomaticTraits, TraitChangedEvent,
};
use scale::layer3::integration::ip_piracy_diplomacy_bridge;
use scale::layer3::intellectual_property_wars::{
    CassusBelli, CassusBelliReason, IntellectualPropertyWarsPlugin, PatentRegistry, TechId,
    TechUsage,
};

#[test]
fn test_ip_piracy_triggers_sanctions() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(IntellectualPropertyWarsPlugin);

    // Add necessary events and systems from diplomacy reflection
    app.add_event::<TraitChangedEvent>();
    app.add_systems(
        Update,
        (ip_piracy_diplomacy_bridge, apply_diplomatic_reactions).chain(),
    );

    // Setup:
    // 1. Owner of a tech
    let owner = app
        .world_mut()
        .spawn(Civilization {
            id: "owner".to_string(),
        })
        .id();

    // 2. Pirate civ (will steal the tech)
    let pirate = app
        .world_mut()
        .spawn((
            Civilization {
                id: "pirate".to_string(),
            },
            DiplomaticTraits {
                is_barbarian: false, // Starts normal
                ..Default::default()
            },
        ))
        .id();

    // 3. A pacifist neighbor who hates barbarians
    let neighbor = app
        .world_mut()
        .spawn((
            Civilization {
                id: "neighbor".to_string(),
            },
            DiplomaticTraits {
                is_pacifist: true,
                ..Default::default()
            },
            DiplomaticRelations {
                relations: vec![DiplomaticStanding {
                    target_id: "pirate".to_string(),
                    standing: 0.0,
                    sanctioned: false,
                }],
            },
        ))
        .id();

    let tech_id = TechId::new("hyper_shields");

    // Owner registers the tech
    app.world_mut()
        .resource_mut::<PatentRegistry>()
        .register(tech_id.clone(), owner, 100);

    // Pirate uses the tech illegally
    app.world_mut().spawn(TechUsage {
        civilization: pirate,
        tech: tech_id,
        is_legal: false,
    });

    // Act: run systems
    // Frame 1: detect_ip_piracy_system (from IntellectualPropertyWarsPlugin) spawns CassusBelli
    //          ip_piracy_diplomacy_bridge sees Added<CassusBelli> and sets pirate is_barbarian = true, emits TraitChangedEvent
    //          apply_diplomatic_reactions processes TraitChangedEvent and makes neighbor sanction the pirate
    app.update();
    app.update();

    // Assert: Pirate is now a barbarian
    let pirate_traits = app.world().get::<DiplomaticTraits>(pirate).unwrap();
    assert!(
        pirate_traits.is_barbarian,
        "Pirate should be marked as barbarian for IP infringement"
    );

    // Assert: CassusBelli was spawned
    let cb_query = app
        .world_mut()
        .query::<&CassusBelli>()
        .iter(app.world())
        .collect::<Vec<_>>();
    assert_eq!(cb_query.len(), 1);
    assert_eq!(cb_query[0].aggressor, owner);
    assert_eq!(cb_query[0].target, pirate);
    assert_eq!(cb_query[0].reason, CassusBelliReason);

    // Assert: Neighbor sanctioned the pirate
    let relations = app.world().get::<DiplomaticRelations>(neighbor).unwrap();
    assert!(
        relations.relations[0].sanctioned,
        "Pacifist neighbor should sanction the IP pirate"
    );
    assert!(
        relations.relations[0].standing < 0.0,
        "Standing should decrease"
    );
}
