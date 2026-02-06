//! User Interface.
//!
//! This module handles the TUI rendering using `ratatui`.
#![allow(missing_docs)]

pub mod chronicle;
pub mod inspector;
pub mod map;
pub mod menu;
pub mod panels;
pub mod status;

use bevy_ecs::prelude::*;
use ratatui::prelude::*;

use crate::shared::menu::MenuState;
use crate::shared::state::GameState;

use self::chronicle::render_chronicle;
use self::map::render_map;
use self::menu::render_main_menu;
use self::panels::render_info_panel;
use self::status::render_status_bar;

/// Render the full game UI for one frame.
///
/// Works with any ratatui backend (native crossterm or WASM ratzilla).
pub fn render(world: &World, frame: &mut Frame) {
    if *world.resource::<GameState>() == GameState::MainMenu {
        let menu_state = world.resource::<MenuState>();
        render_main_menu(frame, frame.area(), menu_state);
        return;
    }

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
    render_map(frame, map_area, world);

    // Render info panel
    render_info_panel(frame, info_area, world);

    // Render status bar
    render_status_bar(frame, status_area, world);

    // Render chronicle
    render_chronicle(frame, frame.area(), world);
}
