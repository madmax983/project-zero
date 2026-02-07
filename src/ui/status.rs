use bevy_ecs::archetype::Archetype;
use bevy_ecs::prelude::*;
use ratatui::{prelude::*, widgets::Paragraph};

use crate::layer1::{BuildMode, ColonyResources, DesignationMode, NamedLocations, Pop, Viewport};
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

    // Calculate average morale
    let (total_morale, morale_count) = world
        .iter_entities()
        .filter_map(|e| e.get::<crate::layer1::Needs>())
        .fold((0.0, 0), |(sum, needs), n| (sum + n.morale(), needs + 1));

    #[allow(clippy::cast_precision_loss)]
    let avg_morale = if morale_count > 0 {
        total_morale / morale_count as f32
    } else {
        0.0
    };

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
#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
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
    morale: f32,
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

    // 4. Morale
    let morale_percent = (morale * 100.0) as u8;
    let morale_color = if morale < 0.3 {
        Color::Red
    } else if morale < 0.7 {
        Color::Yellow
    } else {
        Color::Green
    };
    spans.push(Span::styled("Morale: ", Style::default().fg(morale_color)));
    spans.push(Span::styled(
        format!("{morale_percent}% │ "),
        Style::default().fg(Color::White),
    ));

    // 5. Yield (Food)
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

    // 6. Tools
    spans.push(Span::styled("Tools: ", Style::default().fg(Color::Yellow)));
    spans.push(Span::styled(
        format!("{tools:.0} │ "),
        Style::default().fg(Color::White),
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
        let cost = build_mode.selected.cost();
        let mut cost_parts: Vec<String> = Vec::new();
        if cost.wood > 0.0 {
            cost_parts.push(format!("{:.0}W", cost.wood));
        }
        if cost.stone > 0.0 {
            cost_parts.push(format!("{:.0}S", cost.stone));
        }
        if cost.ore > 0.0 {
            cost_parts.push(format!("{:.0}O", cost.ore));
        }
        if cost.metal > 0.0 {
            cost_parts.push(format!("{:.0}M", cost.metal));
        }
        let cost_str = if cost_parts.is_empty() {
            "Free".to_string()
        } else {
            cost_parts.join(" ")
        };
        spans.push(Span::styled(
            format!(
                "BUILD: {} [{}] (Tab:switch Enter:place Esc:exit)",
                build_mode.selected.label(),
                cost_str,
            ),
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ));
    } else if designation_mode.active {
        let hint = if designation_mode.drag_start.is_some() {
            "Enter:confirm area Esc:exit"
        } else {
            "Enter:start area Esc:exit"
        };
        spans.push(Span::styled(
            format!("DESIGNATE: {} ({hint})", designation_mode.tool.label()),
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
    morale: f32,
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
        morale,
    );

    line.spans
        .iter()
        .map(|s| s.content.as_ref())
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::{BuildMode, DesignationMode};
    use crate::shared::time::SimSpeed;

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
        assert!(status.contains("Morale: 85%"));
        assert!(status.contains("Yield: 123"));
        assert!(status.contains("Tools: 10"));
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

    #[test]
    fn test_render_status_bar_calculates_morale() {
        use crate::layer1::{ColonyResources, NamedLocations, Needs, Pop, Viewport};
        use crate::shared::state::GameState;
        use crate::shared::time::SimulationTime;
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let mut world = World::new();
        // Setup resources needed by render_status_bar
        world.insert_resource(SimulationTime::default());
        world.insert_resource(GameState::Running);
        world.insert_resource(BuildMode::default());
        world.insert_resource(DesignationMode::default());
        world.insert_resource(Viewport::default());
        world.insert_resource(NamedLocations::default());
        world.insert_resource(ColonyResources::default());

        // Spawn pops with needs
        world.spawn((
            Pop,
            Needs {
                hunger: 1.0,
                rest: 1.0,
                leisure: 1.0,
            },
        )); // Morale 1.0
        world.spawn((
            Pop,
            Needs {
                hunger: 0.0,
                rest: 0.0,
                leisure: 0.0,
            },
        )); // Morale 0.0
            // Avg = 0.5

        let backend = TestBackend::new(100, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                render_status_bar(f, f.area(), &world);
            })
            .unwrap();

        // Convert buffer to string to check content
        let buffer = terminal.backend().buffer();
        let cells: Vec<String> = buffer
            .content
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();
        let full_text = cells.join("");

        assert!(full_text.contains("Morale: 50%"));
    }
}
