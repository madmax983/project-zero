use crate::layer1::resources::ColonyResources;
use crate::ui::inspector::render_inspector;
use crate::ui::selection::Selection;
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
    let mut full_text = String::with_capacity(buffer.area.area() as usize);
    for c in &buffer.content {
        full_text.push_str(c.symbol());
    }

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
