use bevy_ecs::prelude::*;
use scale::layer1::research::Discovery;
use scale::layer3::diplomacy::succession::Faction;
use scale::layer1::rearguard::CombatStats;
use scale::layer3::diplomacy::open_source_science::{PublishDiscoveryEvent, process_publication_system, process_enemy_exploits_system, GlobalPrestige};
use bevy_app::App;
use bevy_app::Update;

#[test]
fn test_publishing_discovery_increases_prestige() {
    let mut app = App::new();
    app.insert_resource(GlobalPrestige { value: 100 });
    app.add_event::<PublishDiscoveryEvent>();
    app.add_systems(Update, process_publication_system);

    let discovery = app.world_mut().spawn(Discovery {
        data_type: "ShieldFrequency".to_string(),
        value: 50
    }).id();

    app.world_mut().resource_mut::<Events<PublishDiscoveryEvent>>().send(PublishDiscoveryEvent {
        discovery,
    });

    app.update();

    let prestige = app.world().resource::<GlobalPrestige>();
    assert_eq!(prestige.value, 150, "Publishing a discovery should increase the colony's global prestige.");
}

#[test]
fn test_published_discovery_gives_enemies_combat_bonus() {
    let mut app = App::new();
    app.add_event::<PublishDiscoveryEvent>();
    app.add_systems(Update, process_enemy_exploits_system);

    let discovery = app.world_mut().spawn(Discovery {
        data_type: "ShieldFrequency".to_string(),
        value: 50
    }).id();

    // Hostile Pirate Faction
    let pirate = app.world_mut().spawn((
        Faction { name: "Pirates".to_string() },
        CombatStats { attack: 10.0, defense: 10.0, attack_bonus: 0.0 },
    )).id();

    app.world_mut().resource_mut::<Events<PublishDiscoveryEvent>>().send(PublishDiscoveryEvent {
        discovery,
    });

    app.update();

    let stats = app.world().get::<CombatStats>(pirate).unwrap();
    assert!(stats.attack_bonus > 0.0, "Hostile factions should receive an attack bonus when tactical data is published.");
}
