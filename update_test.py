import re

with open('tests/integration/edible_architecture.rs', 'r') as f:
    content = f.read()

new_test = """
#[test]
fn test_edible_architecture_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<scale::layer1::core::events::BuildingRemovedEvent>();
    app.add_event::<scale::layer1::core::chronicle::AddChronicleEvent>();
    app.add_systems(
        Update,
        (
            scale::layer1::architecture::edible::consume_building_system,
            scale::layer1::core::integration::edible_architecture_chronicle_bridge,
        ),
    );

    let building = app
        .world_mut()
        .spawn((
            scale::layer1::architecture::building::Building {
                building_type: scale::layer1::architecture::building::BuildingType::Housing,
            },
            scale::layer1::map::GridPosition { x: 5, y: 5 },
            scale::layer1::architecture::edible::EdibleMaterial { food_yield: 50.0 },
            scale::layer1::architecture::edible::Consumed,
        ))
        .id();

    app.update();

    let chronicle_events = app
        .world()
        .resource::<Events<scale::layer1::core::chronicle::AddChronicleEvent>>();

    let mut iter = chronicle_events.get_cursor();
    let mut found = false;
    for event in iter.read(chronicle_events) {
        if event.text.contains("Edible Architecture") {
            found = true;
            break;
        }
    }

    assert!(found, "Chronicle event for Edible Architecture should have been emitted");
}
"""

content = re.sub(
    r"\n#\[test\]\nfn test_edible_architecture_chronicle_bridge\(\) \{.*?\}\n",
    new_test,
    content,
    flags=re.DOTALL
)

with open('tests/integration/edible_architecture.rs', 'w') as f:
    f.write(content)
