use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::architecture::building::Building;
use scale::layer1::architecture::ruins::Ruin;
use scale::layer1::core::integration::architecture_of_regret_bridge_system;
use scale::layer1::map::GridPosition;
use scale::layer1::social::unrest::Unrest;
use scale::layer3::guilt::{
    apply_guilt_unrest_system, process_guilt_generation_system, GuiltResource, PsychicResonance,
};

#[test]
fn test_architecture_of_regret_seam() {
    let mut app = App::new();
    app.init_resource::<GuiltResource>();

    app.add_systems(
        Update,
        (
            architecture_of_regret_bridge_system,
            process_guilt_generation_system,
            apply_guilt_unrest_system,
        )
            .chain(),
    );

    // Arrange: A building and a ruin share the same position.
    let _ruin = app
        .world_mut()
        .spawn((
            Ruin {
                original_type: scale::layer1::architecture::building::BuildingType::Housing,
                material: scale::layer1::architecture::building::MaterialType::Stone,
            },
            GridPosition { x: 5, y: 5 },
            PsychicResonance { intensity: 10.0 }, // Emits resonance
        ))
        .id();

    let building = app
        .world_mut()
        .spawn((
            Building {
                building_type: scale::layer1::architecture::building::BuildingType::Smelter,
            },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    app.world_mut().insert_resource(Unrest {
        level: 0.0,
        modifiers: vec![],
    });

    // Act: update
    app.update();

    // Assert: building absorbs resonance
    assert!(
        app.world().get::<PsychicResonance>(building).is_some(),
        "Building should have absorbed PsychicResonance"
    );

    // Assert: guilt generation
    let guilt = app.world().resource::<GuiltResource>().amount;
    assert!(
        guilt >= 10.0,
        "Guilt should have been generated from resonance"
    );

    // Assert: unrest applied
    let unrest = app.world().resource::<Unrest>();
    assert!(
        unrest
            .modifiers
            .iter()
            .any(|m| m.label == "Psychic Resonance"),
        "Unrest should have Psychic Resonance modifier"
    );
}
