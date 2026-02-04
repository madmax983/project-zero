//! User Interface.
//!
//! This module handles the TUI rendering using `ratatui`.

use bevy_ecs::prelude::*;
use ratatui::prelude::*;

/// Chronicle overlay and history view.
pub mod chronicle;
/// Map rendering (terrain, buildings, entities).
pub mod map;
/// Info and Log panels.
pub mod panels;
/// Status bar.
pub mod status;

/// Main UI drawing function.
pub fn draw_ui(frame: &mut Frame, world: &World) {
    // Main vertical split: content + status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Content area
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    // Horizontal split: map + info panel
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20),    // Map area
            Constraint::Length(20), // Info panel
        ])
        .split(main_chunks[0]);

    let map_area = content_chunks[0];
    let info_area = content_chunks[1];
    let status_area = main_chunks[1];

    // Render map
    map::render_map(frame, map_area, world);

    // Render info panel
    panels::render_info_panel(frame, info_area, world);

    // Render status bar
    status::render_status_bar(frame, status_area, world);

    // Render chronicle
    chronicle::render_chronicle(frame, frame.area(), world);
}
