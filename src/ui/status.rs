use crate::layer1::{BuildMode, DesignationMode, NamedLocations, Viewport};
use crate::shared::state::GameState;
use crate::shared::time::{SimSpeed, SimulationTime};
use bevy_ecs::prelude::*;
use ratatui::{prelude::*, widgets::Paragraph};

pub fn render_status_bar(frame: &mut Frame, area: Rect, world: &World) {
    let sim_time = world.resource::<SimulationTime>();
    let game_state = world.resource::<GameState>();
    let build_mode = world.resource::<BuildMode>();
    let designation_mode = world.resource::<DesignationMode>();
    let viewport = world.resource::<Viewport>();
    let locations = world.resource::<NamedLocations>();

    // Determine focus point (center of screen)
    // We use the frame width/height to estimate the center of the view relative to the viewport
    let center_x = viewport.x + (i32::from(frame.area().width) / 2);
    let center_y = viewport.y + (i32::from(frame.area().height) / 2);

    let location_name = locations.get(center_x, center_y).map(String::as_str);

    // NOTE: Dual pause state check. GameState::Paused is controlled by spacebar,
    // SimSpeed::Paused exists but is currently not used (no key binds to it).
    // This allows for future distinction between "paused but still simulating at 0x"
    // vs "completely frozen". Current behavior: only GameState::Paused matters (main.rs:86).
    let paused = *game_state == GameState::Paused || sim_time.speed == SimSpeed::Paused;

    let status = get_status_string(
        sim_time.tick,
        sim_time.speed,
        paused,
        build_mode,
        designation_mode,
        location_name,
    );

    let bar = Paragraph::new(status).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(bar, area);
}

#[must_use]
pub fn get_status_string(
    tick: u64,
    speed: SimSpeed,
    paused: bool,
    build_mode: &BuildMode,
    designation_mode: &DesignationMode,
    location_name: Option<&str>,
) -> String {
    let mode_str = if build_mode.active {
        format!(
            "BUILD: {} (Tab:switch Enter:place Esc:exit)",
            build_mode.selected.label()
        )
    } else if designation_mode.active {
        format!(
            "DESIGNATE: {} (Enter:apply Esc:exit)",
            designation_mode.tool.label()
        )
    } else {
        "B:Build  M:Mine  X:Demolish  L:Chronicle  1-3:Speed  q:Quit".to_string()
    };

    let location_str = location_name.map_or_else(String::new, |name| format!("📍 {name} │ "));

    format!(
        " {} │ Tick: {} │ {} │ {}{} ",
        if paused { "⏸" } else { "▶" },
        tick,
        speed.label(),
        location_str,
        mode_str
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_status_string_with_location() {
        let build_mode = BuildMode::default();
        let designation_mode = DesignationMode::default();
        let status = get_status_string(
            100,
            SimSpeed::Normal,
            false,
            &build_mode,
            &designation_mode,
            Some("My City"),
        );

        assert!(status.contains("📍 My City │"));
        assert!(status.contains("Tick: 100"));
    }

    #[test]
    fn test_get_status_string_without_location() {
        let build_mode = BuildMode::default();
        let designation_mode = DesignationMode::default();
        let status = get_status_string(
            100,
            SimSpeed::Normal,
            false,
            &build_mode,
            &designation_mode,
            None,
        );

        assert!(!status.contains("📍"));
        assert!(status.contains("Tick: 100"));
    }
}
