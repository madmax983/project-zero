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

    // 6. Speed
    spans.push(Span::styled(
        format!("{} ", speed.label()),
        Style::default().fg(Color::DarkGray),
    ));

    // 7. Location
    if let Some(name) = location_name {
        spans.push(Span::raw("│ 📍 "));
        spans.push(Span::styled(
            format!("{name} "),
            Style::default().fg(Color::Magenta),
        ));
    }

    // 8. Mode
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
        );

        assert!(status.contains("Day 100"));
        assert!(status.contains("Souls: 42"));
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
        );

        assert!(status_none.contains("Day 100"));
        assert!(!status_none.contains("📍"));
    }
}
