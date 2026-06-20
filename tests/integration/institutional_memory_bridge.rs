use bevy::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::institutional_memory_chronicle_bridge;
use scale::layer1::institutional_memory::Manual;
use scale::layer1::skills::SkillType;

#[test]
fn test_institutional_memory_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, institutional_memory_chronicle_bridge);

    // Initial update to clear any startup things
    app.update();

    // Spawn a new Manual
    app.world_mut().spawn(Manual {
        skill_type: SkillType::Engineering,
        xp_multiplier: 1.2,
        durability: 100.0,
        max_durability: 100.0,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<&AddChronicleEvent> = cursor.read(events).collect();

    assert_eq!(
        emitted.len(),
        1,
        "Should emit exactly one AddChronicleEvent when a Manual is produced"
    );
    assert!(
        emitted[0]
            .text
            .contains("A high-skill colonist has authored an instructional manual"),
        "Chronicle event text should mention the authored manual"
    );
}
