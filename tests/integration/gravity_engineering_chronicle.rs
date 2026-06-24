use bevy::prelude::*;
use scale::layer1::architecture::building::{
    Building, BuildingType, Height, Material, MaterialType,
};
use scale::layer1::architecture::gravity_engineering::evaluate_structural_integrity_system;
use scale::layer1::architecture::structure::Structure;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::events::BuildingCompletedEvent;
use scale::layer1::core::integration::gravity_engineering_chronicle_bridge;
use scale::layer2::syzygy::PlanetaryGravity;

#[test]
fn test_gravity_engineering_chronicle_bridge_partial_collapse() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<BuildingCompletedEvent>();
    app.add_event::<AddChronicleEvent>();

    app.insert_resource(PlanetaryGravity {
        current: 2.0, // High-G, max height will be 10 / 2 = 5
        base: 2.0,
    });

    // We must chain them so evaluate_structural_integrity_system modifies the Structure BEFORE the bridge reads the event
    app.add_systems(
        Update,
        (
            evaluate_structural_integrity_system,
            gravity_engineering_chronicle_bridge,
        )
            .chain(),
    );

    let tall_building = app
        .world_mut()
        .spawn((
            Building {
                building_type: BuildingType::Housing,
            },
            Height { floors: 6 },         // Too tall
            Material(MaterialType::Wood), // Not reinforced
            Structure {
                current_hp: 100.0,
                max_hp: 100.0,
            },
        ))
        .id();

    app.world_mut().send_event(BuildingCompletedEvent {
        entity: tall_building,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let mut found = false;
    for ev in reader.read(events) {
        if ev.text.contains("suffered a partial structural collapse") {
            found = true;
            assert_eq!(ev.importance, EventImportance::Major);
        }
    }
    assert!(found, "Should emit a Chronicle event for partial collapse");
}

#[test]
fn test_gravity_engineering_chronicle_bridge_instant_collapse() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<BuildingCompletedEvent>();
    app.add_event::<AddChronicleEvent>();

    app.insert_resource(PlanetaryGravity {
        current: 5.0, // Very High-G, max height will be 10 / 5 = 2
        base: 5.0,
    });

    app.add_systems(
        Update,
        (
            evaluate_structural_integrity_system,
            gravity_engineering_chronicle_bridge,
        )
            .chain(),
    );

    let tall_building = app
        .world_mut()
        .spawn((
            Building {
                building_type: BuildingType::Housing,
            },
            Height { floors: 6 }, // Way too tall, excess = 4 floors. 4 * 50 = 200 damage.
            Material(MaterialType::Wood),
            Structure {
                current_hp: 100.0,
                max_hp: 100.0, // Drops to 0
            },
        ))
        .id();

    app.world_mut().send_event(BuildingCompletedEvent {
        entity: tall_building,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let mut found = false;
    for ev in reader.read(events) {
        if ev.text.contains("instantly collapsed") {
            found = true;
            assert_eq!(ev.importance, EventImportance::Major);
        }
    }
    assert!(found, "Should emit a Chronicle event for instant collapse");
}

#[test]
fn test_gravity_engineering_chronicle_bridge_no_damage() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<BuildingCompletedEvent>();
    app.add_event::<AddChronicleEvent>();

    app.insert_resource(PlanetaryGravity {
        current: 1.0, // Normal G, max height 10
        base: 1.0,
    });

    app.add_systems(
        Update,
        (
            evaluate_structural_integrity_system,
            gravity_engineering_chronicle_bridge,
        )
            .chain(),
    );

    let safe_building = app
        .world_mut()
        .spawn((
            Building {
                building_type: BuildingType::Housing,
            },
            Height { floors: 2 }, // Safe
            Material(MaterialType::Wood),
            Structure {
                current_hp: 100.0,
                max_hp: 100.0,
            },
        ))
        .id();

    app.world_mut().send_event(BuildingCompletedEvent {
        entity: safe_building,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    assert_eq!(
        reader.len(events),
        0,
        "Should NOT emit a Chronicle event for a safe building"
    );
}
