use crate::layer1::resources::ColonyResources;
use crate::shared::selection::Selection;
use crate::ui::inspector::render_inspector;
use bevy_ecs::prelude::*;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn test_inspector_shows_waste_stats() {
    let mut world = World::new();
    world.insert_resource(Selection::default());

    // Setup resources with waste
    let resources = ColonyResources {
        waste: 5.0,
        max_waste: 10.0,
        ..Default::default()
    };
    world.insert_resource(resources);

    // Render Inspector (Selection::None shows stats)
    let backend = TestBackend::new(40, 25);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            render_inspector(f, f.area(), &world);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let cells: Vec<String> = buffer
        .content
        .iter()
        .map(|c| c.symbol().to_string())
        .collect();
    let full_text = cells.join("");

    println!("{}", full_text);
    assert!(
        full_text.contains("Waste"),
        "Inspector should display Waste in global stats"
    );
    println!("{}", full_text);
    assert!(
        full_text.contains("5/10"),
        "Inspector should display Waste amounts"
    );
}
