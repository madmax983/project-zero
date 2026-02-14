use bevy_ecs::prelude::*;

use crate::layer1::{
    BuildMode, ChronicleUiState, DesignationMode, DesignationType, GridPosition, Viewport,
    try_cancel_designation, try_designate_area, try_place_building,
};
use crate::platform::input::{GameKeyCode, GameKeyEvent, GameMouseEvent};
use crate::shared::menu::MenuState;
use crate::shared::selection::{Selection, handle_selection_click, screen_to_world};
use crate::shared::state::GameState;
use crate::shared::time::{SimSpeed, SimulationTime};

/// Defines the current input handling context.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum InputContext {
    /// Main Menu mode.
    #[default]
    MainMenu,
    /// Normal game mode.
    Normal,
    /// Building placement mode.
    BuildMode,
    /// Designation mode.
    DesignationMode,
    /// Modal overlay.
    Overlay,
}

/// Stack-based input context manager.
#[derive(Resource, Debug)]
pub struct InputContextStack {
    stack: Vec<InputContext>,
}

impl Default for InputContextStack {
    fn default() -> Self {
        Self {
            stack: vec![InputContext::MainMenu],
        }
    }
}

impl InputContextStack {
    /// Get the current active context.
    ///
    /// # Panics
    ///
    /// Panics if the input stack is empty (which should never happen due to default init).
    #[must_use]
    pub fn current(&self) -> InputContext {
        *self
            .stack
            .last()
            .expect("Input stack should never be empty")
    }

    /// Push a new context.
    pub fn push(&mut self, context: InputContext) {
        self.stack.push(context);
    }

    /// Pop the current context, if not the base.
    pub fn pop(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }
}

/// Route input to the appropriate handler based on current context.
pub fn route_input(world: &mut World, key: GameKeyEvent) {
    let context = world.resource::<InputContextStack>().current();

    match context {
        InputContext::MainMenu => handle_main_menu_mode(world, key),
        InputContext::Normal => handle_normal_mode(world, key),
        InputContext::BuildMode => handle_build_mode(world, key),
        InputContext::DesignationMode => handle_designation_mode(world, key),
        InputContext::Overlay => handle_overlay_mode(world, key),
    }
}

/// Route mouse input to the appropriate handler based on current context.
pub fn route_mouse_input(world: &mut World, mouse: GameMouseEvent) {
    let context = world.resource::<InputContextStack>().current();

    match context {
        InputContext::Normal => {
            let viewport = *world.resource::<Viewport>();
            handle_selection_click(world, mouse, &viewport);
        }
        InputContext::DesignationMode => {
            handle_designation_mouse(world, mouse);
        }
        _ => {}
    }
}

fn handle_normal_mode(world: &mut World, key: GameKeyEvent) {
    match key.code {
        GameKeyCode::Char('q') => {
            *world.resource_mut::<GameState>() = GameState::Quitting;
        }
        GameKeyCode::Esc => {
            let mut selection = world.resource_mut::<Selection>();
            if selection.is_selected() {
                selection.clear();
            } else {
                *world.resource_mut::<GameState>() = GameState::Quitting;
            }
        }
        GameKeyCode::Char(' ') => {
            let mut state = world.resource_mut::<GameState>();
            *state = match *state {
                GameState::Running => GameState::Paused,
                GameState::Paused => GameState::Running,
                GameState::Quitting => GameState::Quitting,
                GameState::MainMenu => GameState::MainMenu,
            };
        }
        GameKeyCode::Char('1') => {
            world.resource_mut::<SimulationTime>().speed = SimSpeed::Normal;
        }
        GameKeyCode::Char('2') => {
            world.resource_mut::<SimulationTime>().speed = SimSpeed::Fast;
        }
        GameKeyCode::Char('3') => {
            world.resource_mut::<SimulationTime>().speed = SimSpeed::Faster;
        }
        GameKeyCode::Char('w') | GameKeyCode::Up => {
            let mut viewport = world.resource_mut::<Viewport>();
            viewport.y = viewport.y.wrapping_sub(1);
        }
        GameKeyCode::Char('s') | GameKeyCode::Down => {
            let mut viewport = world.resource_mut::<Viewport>();
            viewport.y = viewport.y.wrapping_add(1);
        }
        GameKeyCode::Char('a') | GameKeyCode::Left => {
            let mut viewport = world.resource_mut::<Viewport>();
            viewport.x = viewport.x.wrapping_sub(1);
        }
        GameKeyCode::Char('d') | GameKeyCode::Right => {
            let mut viewport = world.resource_mut::<Viewport>();
            viewport.x = viewport.x.wrapping_add(1);
        }
        GameKeyCode::Char('b') => {
            // Enter build mode
            world
                .resource_mut::<InputContextStack>()
                .push(InputContext::BuildMode);

            // Sync side effects
            let (vx, vy) = {
                let viewport = world.resource::<Viewport>();
                (viewport.x, viewport.y)
            };
            let mut build_mode = world.resource_mut::<BuildMode>();
            build_mode.active = true;
            build_mode.cursor = GridPosition {
                x: vx + 10,
                y: vy + 10,
            };
        }
        GameKeyCode::Char('m') => {
            // Enter Designation mode (Mine)
            enter_designation_mode(world, DesignationType::Mine);
        }
        GameKeyCode::Char('x') => {
            // Enter Designation mode (Demolish)
            enter_designation_mode(world, DesignationType::Demolish);
        }
        GameKeyCode::Char('c') => {
            // Enter Designation mode (Chop)
            enter_designation_mode(world, DesignationType::Chop);
        }
        GameKeyCode::Char('l' | 'h') => {
            // Open chronicle
            world
                .resource_mut::<InputContextStack>()
                .push(InputContext::Overlay);
            world.resource_mut::<ChronicleUiState>().is_open = true;
            *world.resource_mut::<GameState>() = GameState::Paused;
        }
        _ => {}
    }
}

fn handle_main_menu_mode(world: &mut World, key: GameKeyEvent) {
    match key.code {
        GameKeyCode::Up | GameKeyCode::Char('w') => {
            world.resource_mut::<MenuState>().prev();
        }
        GameKeyCode::Down | GameKeyCode::Char('s') => {
            world.resource_mut::<MenuState>().next();
        }
        GameKeyCode::Enter | GameKeyCode::Char(' ') => {
            let selected = world.resource::<MenuState>().selected_index;
            match selected {
                0 => {
                    // Start Game
                    *world.resource_mut::<GameState>() = GameState::Running;
                    let mut stack = world.resource_mut::<InputContextStack>();
                    stack.stack = vec![InputContext::Normal];
                }
                1 => {
                    // Quit
                    *world.resource_mut::<GameState>() = GameState::Quitting;
                }
                _ => {}
            }
        }
        GameKeyCode::Esc | GameKeyCode::Char('q') => {
            *world.resource_mut::<GameState>() = GameState::Quitting;
        }
        _ => {}
    }
}

fn enter_designation_mode(world: &mut World, tool: DesignationType) {
    world
        .resource_mut::<InputContextStack>()
        .push(InputContext::DesignationMode);

    let (vx, vy) = {
        let viewport = world.resource::<Viewport>();
        (viewport.x, viewport.y)
    };
    let mut mode = world.resource_mut::<DesignationMode>();
    mode.active = true;
    mode.tool = tool;
    mode.cursor = GridPosition {
        x: vx + 10,
        y: vy + 10,
    };
    mode.drag_start = None;
}

fn handle_build_mode(world: &mut World, key: GameKeyEvent) {
    match key.code {
        GameKeyCode::Esc | GameKeyCode::Char('b') => {
            // Exit build mode
            world.resource_mut::<InputContextStack>().pop();
            world.resource_mut::<BuildMode>().active = false;
        }
        GameKeyCode::Char('w') | GameKeyCode::Up => {
            let mut bm = world.resource_mut::<BuildMode>();
            bm.cursor.y = bm.cursor.y.saturating_sub(1);
        }
        GameKeyCode::Char('s') | GameKeyCode::Down => {
            let mut bm = world.resource_mut::<BuildMode>();
            bm.cursor.y = bm.cursor.y.saturating_add(1);
        }
        GameKeyCode::Char('a') | GameKeyCode::Left => {
            let mut bm = world.resource_mut::<BuildMode>();
            bm.cursor.x = bm.cursor.x.saturating_sub(1);
        }
        GameKeyCode::Char('d') | GameKeyCode::Right => {
            let mut bm = world.resource_mut::<BuildMode>();
            bm.cursor.x = bm.cursor.x.saturating_add(1);
        }
        GameKeyCode::Char(' ') | GameKeyCode::Enter => {
            let build_mode = world.resource::<BuildMode>();
            let cursor = build_mode.cursor;
            let building_type = build_mode.selected;
            try_place_building(world, cursor.x, cursor.y, building_type);
        }
        GameKeyCode::Tab => {
            let mut build_mode = world.resource_mut::<BuildMode>();
            build_mode.selected = build_mode.selected.next();
        }
        GameKeyCode::Char('m') | GameKeyCode::BackTab => {
            let mut build_mode = world.resource_mut::<BuildMode>();
            build_mode.selected_material = build_mode.selected_material.next();
        }
        _ => {}
    }
}

fn handle_designation_mode(world: &mut World, key: GameKeyEvent) {
    match key.code {
        GameKeyCode::Esc => {
            // Exit designation mode
            world.resource_mut::<InputContextStack>().pop();
            let mut mode = world.resource_mut::<DesignationMode>();
            mode.active = false;
            mode.drag_start = None;
        }
        GameKeyCode::Char('m') => {
            // Switch to Mine tool
            world.resource_mut::<DesignationMode>().tool = DesignationType::Mine;
        }
        GameKeyCode::Char('x') => {
            // Switch to Demolish tool
            world.resource_mut::<DesignationMode>().tool = DesignationType::Demolish;
        }
        GameKeyCode::Char('c') => {
            // Switch to Chop tool
            world.resource_mut::<DesignationMode>().tool = DesignationType::Chop;
        }
        GameKeyCode::Char('w') | GameKeyCode::Up => {
            let mut mode = world.resource_mut::<DesignationMode>();
            mode.cursor.y = mode.cursor.y.saturating_sub(1);
        }
        GameKeyCode::Char('s') | GameKeyCode::Down => {
            let mut mode = world.resource_mut::<DesignationMode>();
            mode.cursor.y = mode.cursor.y.saturating_add(1);
        }
        GameKeyCode::Char('a') | GameKeyCode::Left => {
            let mut mode = world.resource_mut::<DesignationMode>();
            mode.cursor.x = mode.cursor.x.saturating_sub(1);
        }
        GameKeyCode::Char('d') | GameKeyCode::Right => {
            let mut mode = world.resource_mut::<DesignationMode>();
            mode.cursor.x = mode.cursor.x.saturating_add(1);
        }
        GameKeyCode::Char(' ') | GameKeyCode::Enter => {
            let mode = world.resource::<DesignationMode>();
            let cursor = mode.cursor;
            let tool = mode.tool;
            let drag_start = mode.drag_start;

            if let Some(start) = drag_start {
                // Second press: designate the rectangle and clear drag_start
                try_designate_area(world, start.x, start.y, cursor.x, cursor.y, tool);
                world.resource_mut::<DesignationMode>().drag_start = None;
            } else {
                // First press: set drag_start
                world.resource_mut::<DesignationMode>().drag_start = Some(cursor);
            }
        }
        GameKeyCode::Backspace | GameKeyCode::Delete => {
            let mode = world.resource::<DesignationMode>();
            try_cancel_designation(world, mode.cursor.x, mode.cursor.y);
        }
        _ => {}
    }
}

fn handle_designation_mouse(world: &mut World, mouse: GameMouseEvent) {
    let viewport = *world.resource::<Viewport>();
    let (world_x, world_y) = screen_to_world(mouse.x, mouse.y, &viewport);

    let mode = world.resource::<DesignationMode>();
    let tool = mode.tool;
    let drag_start = mode.drag_start;

    if let Some(start) = drag_start {
        // Second click: designate the rectangle and clear drag_start
        try_designate_area(world, start.x, start.y, world_x, world_y, tool);
        let mut mode = world.resource_mut::<DesignationMode>();
        mode.drag_start = None;
        mode.cursor = GridPosition {
            x: world_x,
            y: world_y,
        };
    } else {
        // First click: set drag_start and move cursor
        let mut mode = world.resource_mut::<DesignationMode>();
        let pos = GridPosition {
            x: world_x,
            y: world_y,
        };
        mode.drag_start = Some(pos);
        mode.cursor = pos;
    }
}

fn handle_overlay_mode(world: &mut World, key: GameKeyEvent) {
    match key.code {
        GameKeyCode::Esc | GameKeyCode::Char('l' | 'h') => {
            world.resource_mut::<InputContextStack>().pop();
            world.resource_mut::<ChronicleUiState>().is_open = false;
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::Viewport;
    use crate::platform::input::{GameKeyCode, GameKeyEvent, GameMouseEvent};
    use crate::shared::selection::{Selection, SelectionTarget};
    use crate::shared::state::GameState;
    use crate::shared::time::{SimSpeed, SimulationTime};

    fn key_event(code: GameKeyCode) -> GameKeyEvent {
        GameKeyEvent::new(code)
    }

    fn mouse_event(column: u16, row: u16) -> GameMouseEvent {
        GameMouseEvent::new(column, row)
    }

    #[test]
    fn test_input_context_default() {
        let context = InputContext::default();
        assert_eq!(context, InputContext::MainMenu);
    }

    #[test]
    fn test_input_context_stack_push_pop() {
        let mut stack = InputContextStack::default();
        assert_eq!(stack.current(), InputContext::MainMenu);

        stack.push(InputContext::BuildMode);
        assert_eq!(stack.current(), InputContext::BuildMode);

        stack.push(InputContext::Overlay);
        assert_eq!(stack.current(), InputContext::Overlay);

        stack.pop();
        assert_eq!(stack.current(), InputContext::BuildMode);

        stack.pop();
        assert_eq!(stack.current(), InputContext::MainMenu);
    }

    #[test]
    fn test_cannot_pop_main_menu_context() {
        let mut stack = InputContextStack::default();
        assert_eq!(stack.current(), InputContext::MainMenu);

        // Popping base should do nothing (always one context)
        stack.pop();
        assert_eq!(stack.current(), InputContext::MainMenu);
    }

    #[test]
    fn test_input_router_normal_mode_quit() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);
        let mut stack = InputContextStack::default();
        stack.push(InputContext::Normal);
        world.insert_resource(stack);

        route_input(&mut world, key_event(GameKeyCode::Char('q')));

        assert_eq!(*world.resource::<GameState>(), GameState::Quitting);
    }

    #[test]
    fn test_input_router_normal_mode_esc_clears_selection() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);
        let mut stack = InputContextStack::default();
        stack.push(InputContext::Normal);
        world.insert_resource(stack);
        let mut selection = Selection::default();
        selection.select_tile(10, 10);
        world.insert_resource(selection);

        route_input(&mut world, key_event(GameKeyCode::Esc));

        assert!(!world.resource::<Selection>().is_selected());
        assert_eq!(*world.resource::<GameState>(), GameState::Running);
    }

    #[test]
    fn test_input_router_normal_mode_esc_quits_if_no_selection() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);
        let mut stack = InputContextStack::default();
        stack.push(InputContext::Normal);
        world.insert_resource(stack);
        world.insert_resource(Selection::default()); // No selection

        route_input(&mut world, key_event(GameKeyCode::Esc));

        assert_eq!(*world.resource::<GameState>(), GameState::Quitting);
    }

    #[test]
    fn test_input_router_build_mode_blocks_quit() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);
        let mut stack = InputContextStack::default();
        stack.push(InputContext::BuildMode);
        world.insert_resource(stack);
        world.insert_resource(BuildMode {
            active: true,
            ..Default::default()
        });

        route_input(&mut world, key_event(GameKeyCode::Char('q')));

        // 'q' in build mode should NOT quit (should exit build mode instead)
        assert_eq!(*world.resource::<GameState>(), GameState::Running);
    }

    #[test]
    fn test_input_router_overlay_escape_pops_context() {
        let mut world = World::new();
        let mut stack = InputContextStack::default();
        stack.push(InputContext::Overlay); // [MainMenu, Overlay]
        world.insert_resource(stack);
        world.insert_resource(ChronicleUiState { is_open: true });

        route_input(&mut world, key_event(GameKeyCode::Esc));

        // Escape in overlay should pop back to base
        assert_eq!(
            world.resource::<InputContextStack>().current(),
            InputContext::MainMenu
        );
        assert!(!world.resource::<ChronicleUiState>().is_open);
    }

    #[test]
    fn test_chronicle_toggle() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);
        let mut stack = InputContextStack::default();
        stack.push(InputContext::Normal);
        world.insert_resource(stack);
        world.insert_resource(ChronicleUiState::default());

        // Open with 'l'
        route_input(&mut world, key_event(GameKeyCode::Char('l')));
        assert_eq!(
            world.resource::<InputContextStack>().current(),
            InputContext::Overlay
        );
        assert!(world.resource::<ChronicleUiState>().is_open);
        assert_eq!(*world.resource::<GameState>(), GameState::Paused);

        // Close with 'l'
        route_input(&mut world, key_event(GameKeyCode::Char('l')));
        assert_eq!(
            world.resource::<InputContextStack>().current(),
            InputContext::Normal
        );
        assert!(!world.resource::<ChronicleUiState>().is_open);
    }

    #[test]
    fn test_input_context_is_copy() {
        let ctx1 = InputContext::Normal;
        let ctx2 = ctx1; // Should copy
        assert_eq!(ctx1, ctx2);
    }

    #[test]
    fn test_keybinding_normal_mode_all_keys() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);
        world.insert_resource(SimulationTime::default());
        world.insert_resource(Viewport::default());
        let mut stack = InputContextStack::default();
        stack.push(InputContext::Normal);
        world.insert_resource(stack);
        world.insert_resource(BuildMode::default());
        world.insert_resource(DesignationMode::default());
        world.insert_resource(MenuState::default()); // Added for GameState toggle check if needed

        // Test all normal mode bindings work
        route_input(&mut world, key_event(GameKeyCode::Char(' ')));
        assert_eq!(*world.resource::<GameState>(), GameState::Paused);

        route_input(&mut world, key_event(GameKeyCode::Char('1')));
        assert_eq!(world.resource::<SimulationTime>().speed, SimSpeed::Normal);

        route_input(&mut world, key_event(GameKeyCode::Char('w')));
        assert_eq!(world.resource::<Viewport>().y, -1);
    }

    #[test]
    fn test_viewport_overflow_safety() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);
        let mut stack = InputContextStack::default();
        stack.push(InputContext::Normal);
        world.insert_resource(stack);
        world.insert_resource(Viewport {
            x: i32::MAX,
            y: i32::MIN,
        });

        // Move right (x += 1) should wrap
        route_input(&mut world, key_event(GameKeyCode::Char('d')));
        assert_eq!(world.resource::<Viewport>().x, i32::MIN);

        // Move up (y -= 1) should wrap
        route_input(&mut world, key_event(GameKeyCode::Char('w')));
        assert_eq!(world.resource::<Viewport>().y, i32::MAX);
    }

    #[test]
    fn test_cursor_overflow_safety() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);
        let mut stack = InputContextStack::default();
        stack.push(InputContext::BuildMode);
        world.insert_resource(stack);

        let build_mode = BuildMode {
            active: true,
            cursor: GridPosition {
                x: i32::MAX,
                y: i32::MIN,
            },
            ..Default::default()
        };
        world.insert_resource(build_mode);

        // Move right (x += 1) should saturate
        route_input(&mut world, key_event(GameKeyCode::Char('d')));
        assert_eq!(world.resource::<BuildMode>().cursor.x, i32::MAX);

        // Move up (y -= 1) should saturate
        route_input(&mut world, key_event(GameKeyCode::Char('w')));
        assert_eq!(world.resource::<BuildMode>().cursor.y, i32::MIN);
    }

    #[test]
    fn test_designation_mode_entry_and_exit() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);
        let mut stack = InputContextStack::default();
        stack.push(InputContext::Normal);
        world.insert_resource(stack);
        world.insert_resource(Viewport::default());
        world.insert_resource(DesignationMode::default());

        // Enter mine mode
        route_input(&mut world, key_event(GameKeyCode::Char('m')));
        assert_eq!(
            world.resource::<InputContextStack>().current(),
            InputContext::DesignationMode
        );
        assert!(world.resource::<DesignationMode>().active);
        assert_eq!(
            world.resource::<DesignationMode>().tool,
            DesignationType::Mine
        );

        // Exit
        route_input(&mut world, key_event(GameKeyCode::Esc));
        assert_eq!(
            world.resource::<InputContextStack>().current(),
            InputContext::Normal
        );
        assert!(!world.resource::<DesignationMode>().active);

        // Enter demolish mode
        route_input(&mut world, key_event(GameKeyCode::Char('x')));
        assert_eq!(
            world.resource::<InputContextStack>().current(),
            InputContext::DesignationMode
        );
        assert_eq!(
            world.resource::<DesignationMode>().tool,
            DesignationType::Demolish
        );
    }

    #[test]
    fn test_route_mouse_normal_mode_selects() {
        let mut world = World::new();
        let mut stack = InputContextStack::default();
        stack.push(InputContext::Normal);
        world.insert_resource(stack);
        world.insert_resource(Viewport::default());
        world.insert_resource(Selection::default());
        // Need GridPosition/Entity to select? Or just select tile.
        // Selecting tile is enough to verify "something happened".

        route_mouse_input(&mut world, mouse_event(10, 10));

        let selection = world.resource::<Selection>();
        assert_eq!(selection.target(), SelectionTarget::Tile(10, 10));
    }

    #[test]
    fn test_route_mouse_build_mode_ignores_click() {
        let mut world = World::new();
        let mut stack = InputContextStack::default();
        stack.push(InputContext::BuildMode);
        world.insert_resource(stack);
        world.insert_resource(Viewport::default());
        world.insert_resource(Selection::default());

        route_mouse_input(&mut world, mouse_event(10, 10));

        let selection = world.resource::<Selection>();
        assert!(!selection.is_selected());
    }

    #[test]
    fn test_route_mouse_overlay_ignores_click() {
        let mut world = World::new();
        let mut stack = InputContextStack::default();
        stack.push(InputContext::Overlay);
        world.insert_resource(stack);
        world.insert_resource(Viewport::default());
        world.insert_resource(Selection::default());

        route_mouse_input(&mut world, mouse_event(10, 10));

        let selection = world.resource::<Selection>();
        assert!(!selection.is_selected());
    }

    #[test]
    fn test_build_mode_backtab_cycles_material() {
        use crate::layer1::{BuildingType, MaterialType};
        let mut world = World::new();
        world.insert_resource(GameState::Running);
        let mut stack = InputContextStack::default();
        stack.push(InputContext::BuildMode);
        world.insert_resource(stack);
        world.insert_resource(BuildMode {
            active: true,
            selected: BuildingType::Wall, // Walls support materials
            selected_material: MaterialType::Wood,
            ..Default::default()
        });

        // First BackTab -> Stone
        route_input(&mut world, key_event(GameKeyCode::BackTab));
        assert_eq!(
            world.resource::<BuildMode>().selected_material,
            MaterialType::Stone
        );
    }
}
