//! User Interface.
//!
//! This module handles the TUI rendering using `ratatui`.
//!
//! # Architecture
//!
//! The UI is built using the `ratatui` crate, which provides a backend-agnostic way to
//! render terminal interfaces. This allows the game to run:
//! - **Natively** using `crossterm`.
//! - **In Browser** using `ratzilla` (WASM).
//!
//! # Layout
//!
//! The `render` function acts as the main entry point for the UI system. It handles
//! the high-level layout switching:
//!
//! 1. **Main Menu**: If `GameState` is `MainMenu`, it delegates to `render_main_menu`.
//! 2. **Game Interface**: Otherwise, it renders the simulation view, split into:
//!    - **Map**: The main gameplay area (`map::render_map`).
//!    - **Info Panel**: Selected entity details (`panels::render_info_panel`).
//!    - **Status Bar**: Global colony stats (`status::render_status_bar`).
//!    - **Chronicle**: Historical events overlay (`chronicle::render_chronicle`).

/// Chronicle overlay rendering.
pub mod chronicle;
/// Entity inspector panel.
pub mod inspector;
/// Map rendering logic.
pub mod map;
/// Main menu rendering.
pub mod menu;
/// Notifications overlay rendering.
pub mod notifications;
/// Info panels (inspector, etc).
pub mod panels;
/// Status bar rendering.
pub mod status;

#[cfg(test)]
mod waste_ui_tests;

use bevy_ecs::prelude::*;
use ratatui::prelude::*;

use crate::layer2::render::render_system_view;
use crate::layer2::system::ViewMode;
use crate::shared::menu::MenuState;
use crate::shared::state::GameState;

use self::chronicle::render_chronicle;
use self::map::render_map;
use self::menu::render_main_menu;
use self::notifications::render_notifications;
use self::panels::render_info_panel;
use self::status::render_status_bar;

/// Render the full game UI for one frame.
///
/// This function is called every frame by the platform layer (native main loop or WASM animation frame).
/// It queries the ECS world for necessary state (`GameState`, `MenuState`, etc.) and draws to the
/// provided `ratatui` frame.
///
/// # Arguments
///
/// * `world` - The ECS world containing all game state and resources.
/// * `frame` - The `ratatui` frame to render into.
pub fn render(world: &World, frame: &mut Frame) {
    if *world.resource::<GameState>() == GameState::MainMenu {
        let menu_state = world.resource::<MenuState>();
        render_main_menu(frame, frame.area(), menu_state);
        return;
    }

    // Check ViewMode
    let view_mode = world.resource::<ViewMode>();

    if *view_mode == ViewMode::System {
        render_system_view(frame, frame.area(), world);
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

    // Render notifications overlay on top of map
    render_notifications(frame, map_area, world);

    // Render info panel
    render_info_panel(frame, info_area, world);

    // Render status bar
    render_status_bar(frame, status_area, world);

    // Render chronicle
    render_chronicle(frame, frame.area(), world);
}
