use bevy_ecs::archetype::Archetype;
use bevy_ecs::prelude::*;
use ratatui::{prelude::*, widgets::Paragraph};

use crate::layer1::{
    BuildMode, ColonyResources, DesignationMode, MORALE_HIGH_THRESHOLD, MORALE_LOW_THRESHOLD,
    NamedLocations, Needs, Pop, Viewport,
};
use crate::shared::state::GameState;
use crate::shared::time::{SimSpeed, SimulationTime};

pub fn render_status_bar(frame: &mut Frame, area: Rect, world: &World) {
    let sim_time = world.resource::<SimulationTime>();
    let game_state = world.resource::<GameState>();
    let build_mode = world.resource::<BuildMode>();
    let designation_mode = world.resource::<DesignationMode>();
    let viewport = world.resource::<Viewport>();
    let locations = world.resource::<NamedLocations>();
    let resources = world.resource::<ColonyResources>();

    // NOTE: Dual pause state check. GameState::Paused is controlled by spacebar,
    // SimSpeed::Paused exists but is currently not used (no key binds to it).
    // This allows for future distinction between "paused but still simulating at 0x"
    // vs "completely frozen". Current behavior: only GameState::Paused matters (main.rs:86).
    let paused = *game_state == GameState::Paused || sim_time.speed == SimSpeed::Paused;

    let screen_area = frame.area();
    let center_x = viewport.x + i32::from(screen_area.width / 2);
    let center_y = viewport.y + i32::from(screen_area.height / 2);
    let location_name = locations.get(center_x, center_y).map(String::as_str);

    // Count pops safely with immutable world access
    let pop_count = world.component_id::<Pop>().map_or(0, |pop_id| {
        world
            .archetypes()
            .iter()
            .filter(|archetype| archetype.contains(pop_id))
            .map(Archetype::len)
            .sum()
    });

    let avg_morale = calculate_average_morale(world);

    let status = get_status_line(
        sim_time.tick,
        sim_time.speed,
        paused,
        build_mode,
        designation_mode,
        location_name,
        pop_count,
        resources.food,
        resources.tools,
        avg_morale,
    );

    let bar = Paragraph::new(status).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(bar, area);
}

#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn get_status_line<'a>(
    tick: u64,
    speed: SimSpeed,
    paused: bool,
    build_mode: &BuildMode,
    designation_mode: &DesignationMode,
    location_name: Option<&'a str>,
    pop_count: usize,
    food_yield: f32,
    tools: f32,
    avg_morale: f32,
) -> Line<'a> {
    let mut spans = Vec::new();

    // 1. Play/Pause
    if paused {
        spans.push(Span::styled(" ⏸ ", Style::default().fg(Color::Red)));
    } else {
        spans.push(Span::styled(" ▶ ", Style::default().fg(Color::Green)));
    }

    // 2. Day
    spans.push(Span::raw(format!("Day {tick} │ ")));

    // 3. Souls
    spans.push(Span::styled("Souls: ", Style::default().fg(Color::Cyan)));
    spans.push(Span::styled(
        format!("{pop_count} │ "),
        Style::default().fg(Color::White),
    ));

    // 4. Yield (Food)
    let food_color = if food_yield < 10.0 {
        Color::Red
    } else {
        Color::Green
    };
    spans.push(Span::styled("Yield: ", Style::default().fg(food_color)));
    spans.push(Span::styled(
        format!("{food_yield:.0} │ "),
        Style::default().fg(Color::White),
    ));

    // 5. Tools
    spans.push(Span::styled("Tools: ", Style::default().fg(Color::Yellow)));
    spans.push(Span::styled(
        format!("{tools:.0} │ "),
        Style::default().fg(Color::White),
    ));

    // 6. Morale
    let morale_percent = (avg_morale * 100.0) as u8;
    let morale_color = if avg_morale >= MORALE_HIGH_THRESHOLD {
        Color::Green
    } else if avg_morale <= MORALE_LOW_THRESHOLD {
        Color::Red
    } else {
        Color::White
    };
    spans.push(Span::styled("Morale: ", Style::default().fg(Color::Magenta)));
    spans.push(Span::styled(
        format!("{morale_percent}% │ "),
        Style::default().fg(morale_color),
    ));

    // 7. Speed
    spans.push(Span::styled(
        format!("{} ", speed.label()),
        Style::default().fg(Color::DarkGray),
    ));

    // 8. Location
    if let Some(name) = location_name {
        spans.push(Span::raw("│ 📍 "));
        spans.push(Span::styled(
            format!("{name} "),
            Style::default().fg(Color::Magenta),
        ));
    }

    // 9. Mode
    // Add some padding before mode
    spans.push(Span::raw(" "));
    if build_mode.active {
        spans.push(Span::styled(
            format!(
                "BUILD: {} (Tab:switch Enter:place Esc:exit)",
                build_mode.selected.label()
            ),
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ));
    } else if designation_mode.active {
        spans.push(Span::styled(
            format!(
                "DESIGNATE: {} (Enter:apply Esc:exit)",
                designation_mode.tool.label()
            ),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::styled(
            "B:Build  M:Mine  X:Demolish  L:Chronicle  1-3:Speed  q:Quit",
            Style::default().fg(Color::Gray),
        ));
    }

    // Trailing space
    spans.push(Span::raw(" "));

    Line::from(spans)
}

#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn get_status_string(
    tick: u64,
    speed: SimSpeed,
    paused: bool,
    build_mode: &BuildMode,
    designation_mode: &DesignationMode,
    location_name: Option<&str>,
    pop_count: usize,
    food_yield: f32,
    tools: f32,
    avg_morale: f32,
) -> String {
    let line = get_status_line(
        tick,
        speed,
        paused,
        build_mode,
        designation_mode,
        location_name,
        pop_count,
        food_yield,
        tools,
        avg_morale,
    );

    line.spans
        .iter()
        .map(|s| s.content.as_ref())
        .collect::<String>()
}

/// Calculates the average morale of all pops in the world.
pub fn calculate_average_morale(world: &World) -> f32 {
    let (total_morale, morale_count) = world
        .iter_entities()
        .filter_map(|e| e.get::<Needs>())
        .fold((0.0, 0), |(sum, count), n| (sum + n.morale(), count + 1));

    if morale_count > 0 {
        total_morale / morale_count as f32
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::{BuildMode, DesignationMode};
    use crate::shared::time::SimSpeed;

    #[test]
    fn test_calculate_average_morale() {
        let mut world = World::new();

        // No pops
        assert_eq!(calculate_average_morale(&world), 0.0);

        // One pop with Needs
        world.spawn(Needs {
            hunger: 1.0,
            rest: 1.0,
            leisure: 1.0,
        }); // Morale 1.0
        assert!((calculate_average_morale(&world) - 1.0).abs() < f32::EPSILON);

        // Another pop with Needs
        world.spawn(Needs {
            hunger: 0.0,
            rest: 0.0,
            leisure: 0.0,
        }); // Morale 0.0
        assert!((calculate_average_morale(&world) - 0.5).abs() < f32::EPSILON);

        // Pop without Needs (should be ignored)
        world.spawn(Pop);
        assert!((calculate_average_morale(&world) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_get_status_line_colors() {
        let tick = 100;
        let speed = SimSpeed::Normal;
        let paused = false;
        let build_mode = BuildMode::default();
        let designation_mode = DesignationMode::default();
        let location = None;
        let pop = 10;
        let food = 100.0;
        let tools = 10.0;

        // Test High Morale
        let line_high = get_status_line(
            tick,
            speed,
            paused,
            &build_mode,
            &designation_mode,
            location,
            pop,
            food,
            tools,
            0.9,
        );
        let morale_span = line_high
            .spans
            .iter()
            .find(|s| s.content.contains("%"))
            .unwrap();
        assert_eq!(morale_span.style.fg, Some(Color::Green));

        // Test Low Morale
        let line_low = get_status_line(
            tick,
            speed,
            paused,
            &build_mode,
            &designation_mode,
            location,
            pop,
            food,
            tools,
            0.1,
        );
        let morale_span_low = line_low
            .spans
            .iter()
            .find(|s| s.content.contains("%"))
            .unwrap();
        assert_eq!(morale_span_low.style.fg, Some(Color::Red));

        // Test Neutral Morale
        let line_neutral = get_status_line(
            tick,
            speed,
            paused,
            &build_mode,
            &designation_mode,
            location,
            pop,
            food,
            tools,
            0.5,
        );
        let morale_span_neutral = line_neutral
            .spans
            .iter()
            .find(|s| s.content.contains("%"))
            .unwrap();
        assert_eq!(morale_span_neutral.style.fg, Some(Color::White));
    }

    use ratatui::backend::TestBackend;

    #[test]
    fn test_render_status_bar() {
        let mut world = World::new();
        // Insert required resources
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(crate::shared::state::GameState::Running);
        world.insert_resource(crate::layer1::BuildMode::default());
        world.insert_resource(crate::layer1::DesignationMode::default());
        world.insert_resource(crate::layer1::Viewport::default());
        world.insert_resource(crate::layer1::NamedLocations::default());
        world.insert_resource(crate::layer1::ColonyResources::default());

        let backend = TestBackend::new(100, 1); // Wide enough for status bar
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                render_status_bar(f, f.area(), &world);
            })
            .unwrap();

        // Assertions on buffer content?
        let buffer = terminal.backend().buffer();
        let cells: Vec<String> = buffer
            .content
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        let full_text = cells.join("");

        assert!(full_text.contains("Day 0"));
        assert!(full_text.contains("Souls: 0"));
    }

    #[test]
    fn test_get_status_string_formatting() {
        let tick = 100;
        let speed = SimSpeed::Normal;
        let paused = false;
        let build_mode = BuildMode::default();
        let designation_mode = DesignationMode::default();

        // With location
        let status = get_status_string(
            tick,
            speed,
            paused,
            &build_mode,
            &designation_mode,
            Some("Test City"),
            42,    // Pops
            123.0, // Food
            10.0,  // Tools
            0.85,  // Morale
        );

        assert!(status.contains("Day 100"));
        assert!(status.contains("Souls: 42"));
        assert!(status.contains("Yield: 123"));
        assert!(status.contains("Tools: 10"));
        assert!(status.contains("Morale: 85%"));
        assert!(status.contains("1x"));
        assert!(status.contains("📍 Test City"));

        // Without location
        let status_none = get_status_string(
            tick,
            speed,
            paused,
            &build_mode,
            &designation_mode,
            None,
            0,
            0.0,
            0.0,
            0.0,
        );

        assert!(status_none.contains("Day 100"));
        assert!(!status_none.contains("📍"));
    }
}
