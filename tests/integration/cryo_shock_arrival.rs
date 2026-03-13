use bevy::MinimalPlugins;
use bevy_app::App;
use bevy_ecs::prelude::*;
use scale::layer1::cryo_shock::CryoShock;
use scale::layer1::integration::cryo_pop_spawn_bridge_system;
use scale::layer1::pop::{spawn_initial_pops, Pop, PopBorn};
use scale::layer1::terrain::{TerrainGrid, TerrainType};

#[test]
fn initial_colonists_receive_cryo_shock() {
    let mut world = World::new();

    // Create minimal terrain for popping logic to work
    let width = 10;
    let height = 10;
    let tiles = vec![TerrainType::Grass; width * height];
    let terrain = TerrainGrid {
        width,
        height,
        tiles,
    };
    world.insert_resource(terrain);

    // Call spawn_initial_pops
    spawn_initial_pops(&mut world);

    // Verify all pops got CryoShock
    let mut query = world.query::<(&Pop, Option<&CryoShock>)>();

    let mut pop_count = 0;
    for (_, shock_opt) in query.iter(&world) {
        pop_count += 1;
        assert!(
            shock_opt.is_some(),
            "Initial colonists must arrive with CryoShock component"
        );
        let shock = shock_opt.unwrap();
        assert_eq!(shock.duration_ticks, 1000);
        assert_eq!(shock.severity, 0.5);
    }
    assert_eq!(pop_count, 5, "Expected 5 initial colonists to spawn");
}

#[test]
fn new_arrivals_from_cryo_receive_cryo_shock() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<PopBorn>();

    app.add_systems(bevy_app::Update, cryo_pop_spawn_bridge_system);

    // Spawn a dummy pop
    let pop_entity = app.world_mut().spawn(Pop).id();

    // Emit event with 'Cryo' in source
    app.world_mut()
        .resource_mut::<Events<PopBorn>>()
        .send(PopBorn {
            entity: pop_entity,
            name: "Test Pop".to_string(),
            tick: 0,
            source: "Cryo-Stasis Pod".to_string(),
        });

    app.update();

    let shock_opt = app.world().get::<CryoShock>(pop_entity);
    assert!(
        shock_opt.is_some(),
        "PopBorn event with 'Cryo' source must apply CryoShock component"
    );
    let shock = shock_opt.unwrap();
    assert_eq!(shock.duration_ticks, 1000);
    assert_eq!(shock.severity, 0.5);
}

#[test]
fn new_arrivals_not_from_cryo_do_not_receive_cryo_shock() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<PopBorn>();

    app.add_systems(bevy_app::Update, cryo_pop_spawn_bridge_system);

    // Spawn a dummy pop
    let pop_entity = app.world_mut().spawn(Pop).id();

    // Emit event with non-cryo source
    app.world_mut()
        .resource_mut::<Events<PopBorn>>()
        .send(PopBorn {
            entity: pop_entity,
            name: "Test Clone".to_string(),
            tick: 0,
            source: "Clone Vat".to_string(),
        });

    app.update();

    let shock_opt = app.world().get::<CryoShock>(pop_entity);
    assert!(
        shock_opt.is_none(),
        "PopBorn event from 'Clone Vat' should not apply CryoShock"
    );
}
