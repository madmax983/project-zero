use bevy::prelude::*;
use scale::layer1::building::{Building, BuildingType};
use scale::layer1::events::BuildingCompletedEvent;
use scale::layer1::map::GridPosition;
use scale::layer1::tech::ghost_code::{ghost_infection_system, DataResidue, GhostCode};

#[test]
fn test_ghost_code_infection_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<BuildingCompletedEvent>();
    app.add_systems(Update, ghost_infection_system);

    let pos = GridPosition { x: 5, y: 5 };

    // Spawn residue
    let residue_entity = app
        .world_mut()
        .spawn((
            DataResidue {
                source_type: BuildingType::Hospital,
            }, // Changed from MedicalBay
            pos,
        ))
        .id();

    // Spawn new building
    let new_building = app
        .world_mut()
        .spawn((
            Building {
                building_type: BuildingType::Tower,
            }, // Changed from Turret
            pos,
        ))
        .id();

    // Trigger completion event
    app.world_mut().send_event(BuildingCompletedEvent {
        entity: new_building,
    });

    // Run schedule
    app.update();

    // Verify GhostCode attached and residue despawned
    assert!(
        app.world().get::<GhostCode>(new_building).is_some(),
        "Building should have acquired GhostCode"
    );
    assert!(
        app.world().get_entity(residue_entity).is_err(),
        "Residue should have been consumed"
    ); // Changed to is_err()
}
