use bevy::prelude::*;
use scale::layer1::architecture::resonant_architecture::{apply_resonant_architecture_system, Room, RoomBoundary, BuildingMaterial, PopResonanceTraits};
use scale::layer1::core::map::GridPosition;
use scale::layer1::entities::pop::Pop;

#[test]
fn test_resonant_architecture_integration() {
    let mut app = App::new();
    app.add_systems(Update, apply_resonant_architecture_system);

    app.world_mut().spawn((
        Room { material: BuildingMaterial::MindStone },
        RoomBoundary { radius: 5 },
        GridPosition { x: 0, y: 0 },
    ));

    let pop = app
        .world_mut()
        .spawn((
            Pop,
            GridPosition { x: 1, y: 0 }, // Inside room
            PopResonanceTraits::default(),
        ))
        .id();

    app.update();

    let traits = app.world().get::<PopResonanceTraits>(pop).unwrap();
    assert_eq!(traits.research_speed_mult, 2.0);
    assert_eq!(traits.stress_gain_mult, 2.0);
}
