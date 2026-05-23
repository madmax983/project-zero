use bevy::prelude::*;
use scale::layer3::intellectual_property_wars::{
    PatentRegistry, TechId, TechUsage, detect_ip_piracy_system,
};
use scale::layer3::diplomacy_reflection::{
    Civilization, DiplomaticRelations, DiplomaticStanding, DiplomaticTraits, TraitChangedEvent,
    apply_diplomatic_reactions,
};
use scale::layer3::integration::ip_piracy_diplomacy_bridge;

#[test]
fn ip_piracy_triggers_sanctions() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Register events and resources
    app.add_event::<TraitChangedEvent>();
    app.init_resource::<PatentRegistry>();

    app.add_systems(Update, (
        detect_ip_piracy_system,
        ip_piracy_diplomacy_bridge,
        apply_diplomatic_reactions,
    ).chain());

    // Create Owner
    let owner = app.world_mut().spawn((
        Civilization { id: "owner".to_string() },
        DiplomaticTraits::default(),
    )).id();

    // Create Pirate
    let pirate = app.world_mut().spawn((
        Civilization { id: "pirate".to_string() },
        DiplomaticTraits::default(),
    )).id();

    // Create Pacifist Neighbor
    let neighbor = app.world_mut().spawn((
        Civilization { id: "neighbor".to_string() },
        DiplomaticTraits { is_pacifist: true, ..default() },
        DiplomaticRelations {
            relations: vec![DiplomaticStanding {
                target_id: "pirate".to_string(),
                standing: 0.0,
                sanctioned: false,
            }],
        }
    )).id();

    // Setup Patent
    let tech_id = TechId::new("hyper_drive");
    app.world_mut()
        .resource_mut::<PatentRegistry>()
        .register(tech_id.clone(), owner, 100);

    // Pirate uses tech illegally
    app.world_mut().spawn(TechUsage {
        civilization: pirate,
        tech: tech_id,
        is_legal: false,
    });

    app.update();

    // Check relations
    let relations = app.world().get::<DiplomaticRelations>(neighbor).unwrap();
    assert!(
        relations.relations[0].sanctioned,
        "Pirate should be sanctioned by pacifist neighbor"
    );
    assert!(
        relations.relations[0].standing < 0.0,
        "Pirate's standing should decrease"
    );
}
