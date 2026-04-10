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
//!    - **Shell Panes**: Chronicle and tech now live behind shell-managed pane plugins.
//! 3. **Hypertile Bridge**: `shell::plugins` adapts those legacy frame-based renderers into
//!    pane plugins while the runtime shell migration is in progress.

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
/// Seasonal graphics helpers.
pub mod seasonal_gfx;
/// Hypertile shell scaffolding.
pub mod shell;
/// UI State resource.
pub mod state;
/// Status bar rendering.
pub mod status;
/// Tech Tree UI rendering.
pub mod tech;

#[cfg(test)]
mod waste_ui_tests;

use bevy_ecs::prelude::*;
use ratatui::prelude::*;
pub use state::*;

use crate::layer2::render::render_system_view;
use crate::layer2::system::ViewMode;
use crate::shared::menu::MenuState;
use crate::shared::state::GameState;

use self::map::render_map;
use self::menu::render_main_menu;
use self::notifications::render_notifications;
use self::panels::render_info_panel;
use self::shell::UiShell;
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

    // Check Global UI suppression (Cinematic/Possession mode)
    let ui_state = world.get_resource::<UiState>();
    let suppress_ui = ui_state.is_some_and(|s| s.suppress_global_ui);

    if suppress_ui {
        // Full screen map
        render_map(frame, frame.area(), world);
        render_notifications(frame, frame.area(), world);
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
}

/// Render the game UI through the hypertile shell.
pub fn render_with_shell(world: &World, shell: &mut UiShell, frame: &mut Frame) {
    if *world.resource::<GameState>() == GameState::MainMenu {
        let menu_state = world.resource::<MenuState>();
        render_main_menu(frame, frame.area(), menu_state);
        return;
    }

    let ui_state = world.get_resource::<UiState>();
    let suppress_ui = ui_state.is_some_and(|state| state.suppress_global_ui);
    if suppress_ui {
        render_map(frame, frame.area(), world);
        render_notifications(frame, frame.area(), world);
        return;
    }

    match *world.resource::<ViewMode>() {
        ViewMode::System => {
            let _ = shell.switch_to_workspace("System Survey");
        }
        ViewMode::Colony if shell.active_workspace_name() == "System Survey" => {
            let _ = shell.switch_to_workspace("Colony Ops");
        }
        ViewMode::Colony => {}
    }

    shell.render(frame.area(), frame.buffer_mut());
    render_notifications(frame, frame.area(), world);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use crate::shared::menu::MenuState;
    use crate::shared::state::GameState;
    use crate::layer2::system::ViewMode;
    use bevy_ecs::world::World;
    use crate::ui::UiState;

    // Add necessary component for testing full render path
    use crate::layer1::TerrainGrid;
    use crate::layer1::water::WaterGrid;
    use crate::layer1::Viewport;
    use crate::layer1::BuildMode;
    use crate::layer1::DesignationMode;
    use crate::ui::map::RenderCache;
    use crate::shared::time::WallTime;
    use crate::shared::selection::Selection;
    use crate::shared::time::SimulationTime;

    fn setup_world_for_render() -> World {
        let mut world = World::new();
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles: vec![crate::layer1::TerrainType::Grass; 100] });
        world.insert_resource(crate::layer1::nature::terrain::TerrainGrid { width: 10, height: 10, tiles: vec![crate::layer1::TerrainType::Grass; 100] });
        world.insert_resource(WaterGrid::new(10, 10));
        world.insert_resource(Viewport::default());
        world.insert_resource(BuildMode::default());
        world.insert_resource(DesignationMode::default());
        world.insert_resource(RenderCache::default());
        world.insert_resource(WallTime(0.0));
        world.insert_resource(crate::layer1::economy::resources::ColonyResources::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(Selection::default());
        world.insert_resource(crate::layer1::locations::NamedLocations::default());
        world.insert_resource(crate::layer1::chronicle::Chronicle::default());

        world
    }

    #[test]
    fn test_render_main_menu_state() {
        let mut world = World::new();
        world.insert_resource(GameState::MainMenu);
        world.insert_resource(MenuState {
            options: vec!["Start".to_string()],
            selected_index: 0,
        });

        let backend = TestBackend::new(40, 25);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            render(&world, f);
        }).unwrap();
    }

    #[test]
    fn test_render_system_view_mode() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);
        world.insert_resource(ViewMode::System);

        let backend = TestBackend::new(40, 25);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            render(&world, f);
        }).unwrap();
    }

    #[test]
    fn test_render_suppressed_ui() {
        let mut world = setup_world_for_render();
        world.insert_resource(GameState::Running);
        world.insert_resource(ViewMode::Colony);
        world.insert_resource(UiState {
            suppress_global_ui: true,
            ..Default::default()
        });

        let backend = TestBackend::new(40, 25);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            render(&world, f);
        }).unwrap();
    }

    #[test]
    fn test_render_normal_colony() {
        let mut world = setup_world_for_render();
        world.insert_resource(GameState::Running);
        world.insert_resource(ViewMode::Colony);

        let backend = TestBackend::new(40, 25);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            render(&world, f);
        }).unwrap();
    }

    #[test]
    fn test_render_with_shell_main_menu() {
        let mut world = World::new();
        world.insert_resource(GameState::MainMenu);
        world.insert_resource(MenuState {
            options: vec!["Start".to_string()],
            selected_index: 0,
        });

        let backend = TestBackend::new(40, 25);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut shell = crate::ui::shell::build_default_shell(std::rc::Rc::new(std::cell::RefCell::new(World::new())), crate::ui::shell::config::ShellConfig::default());

        terminal.draw(|f| {
            render_with_shell(&world, &mut shell, f);
        }).unwrap();
    }

    #[test]
    fn test_render_with_shell_colony() {
        let mut world = setup_world_for_render();
        world.insert_resource(GameState::Running);
        world.insert_resource(ViewMode::Colony);

        let backend = TestBackend::new(40, 25);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut w2 = setup_world_for_render();
        w2.insert_resource(GameState::Running);
        w2.insert_resource(ViewMode::Colony);
        let mut shell = crate::ui::shell::build_default_shell(std::rc::Rc::new(std::cell::RefCell::new(w2)), crate::ui::shell::config::ShellConfig::default());

        terminal.draw(|f| {
            render_with_shell(&world, &mut shell, f);
        }).unwrap();
    }
}
