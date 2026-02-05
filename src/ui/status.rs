use bevy_ecs::prelude::*;
use ratatui::{prelude::*, widgets::Paragraph};

use crate::layer1::{BuildMode, DesignationMode};
use crate::shared::state::GameState;
use crate::shared::time::{SimSpeed, SimulationTime};

pub fn render_status_bar(frame: &mut Frame, area: Rect, world: &World) {
    let sim_time = world.resource::<SimulationTime>();
    let game_state = world.resource::<GameState>();
    let build_mode = world.resource::<BuildMode>();
    let designation_mode = world.resource::<DesignationMode>();

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

    format!(
        " {} │ Tick: {} │ {} │ {} ",
        if paused { "⏸" } else { "▶" },
        tick,
        speed.label(),
        mode_str
    )
}
