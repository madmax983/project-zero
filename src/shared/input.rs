use bevy_ecs::prelude::*;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::layer1::{BuildMode, GridPosition, Viewport, try_place_building};
use crate::shared::state::GameState;
use crate::shared::time::{SimSpeed, SimulationTime};

/// Defines the current input handling context.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum InputContext {
    /// Normal game mode.
    #[default]
    Normal,
    /// Building placement mode.
    BuildMode,
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
            stack: vec![InputContext::Normal],
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

/// Routes input to context-appropriate handlers.
pub struct InputRouter;

impl InputRouter {
    /// Create a new [`InputRouter`].
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Route input to the appropriate handler based on current context.
    pub fn route(&mut self, world: &mut World, key: KeyEvent) {
        // Only process key press events
        if key.kind != KeyEventKind::Press {
            return;
        }

        let context = world.resource::<InputContextStack>().current();

        match context {
            InputContext::Normal => handle_normal_mode(world, key),
            InputContext::BuildMode => handle_build_mode(world, key),
            InputContext::Overlay => handle_overlay_mode(world, key),
        }
    }
}

impl Default for InputRouter {
    fn default() -> Self {
        Self::new()
    }
}

fn handle_normal_mode(world: &mut World, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            *world.resource_mut::<GameState>() = GameState::Quitting;
        }
        KeyCode::Char(' ') => {
            let mut state = world.resource_mut::<GameState>();
            *state = match *state {
                GameState::Running => GameState::Paused,
                GameState::Paused => GameState::Running,
                GameState::Quitting => GameState::Quitting,
            };
        }
        KeyCode::Char('1') => {
            world.resource_mut::<SimulationTime>().speed = SimSpeed::Normal;
        }
        KeyCode::Char('2') => {
            world.resource_mut::<SimulationTime>().speed = SimSpeed::Fast;
        }
        KeyCode::Char('3') => {
            world.resource_mut::<SimulationTime>().speed = SimSpeed::Faster;
        }
        KeyCode::Char('w') | KeyCode::Up => {
            let mut viewport = world.resource_mut::<Viewport>();
            viewport.y = viewport.y.wrapping_sub(1);
        }
        KeyCode::Char('s') | KeyCode::Down => {
            let mut viewport = world.resource_mut::<Viewport>();
            viewport.y = viewport.y.wrapping_add(1);
        }
        KeyCode::Char('a') | KeyCode::Left => {
            let mut viewport = world.resource_mut::<Viewport>();
            viewport.x = viewport.x.wrapping_sub(1);
        }
        KeyCode::Char('d') | KeyCode::Right => {
            let mut viewport = world.resource_mut::<Viewport>();
            viewport.x = viewport.x.wrapping_add(1);
        }
        KeyCode::Char('b') => {
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
        KeyCode::Char('c') => {
            // Open chronicle (Spec 010 placeholder)
            world
                .resource_mut::<InputContextStack>()
                .push(InputContext::Overlay);
        }
        _ => {}
    }
}

fn handle_build_mode(world: &mut World, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('b') => {
            // Exit build mode
            world.resource_mut::<InputContextStack>().pop();
            world.resource_mut::<BuildMode>().active = false;
        }
        KeyCode::Char('w') | KeyCode::Up => {
            let mut bm = world.resource_mut::<BuildMode>();
            bm.cursor.y = bm.cursor.y.saturating_sub(1);
        }
        KeyCode::Char('s') | KeyCode::Down => {
            let mut bm = world.resource_mut::<BuildMode>();
            bm.cursor.y = bm.cursor.y.saturating_add(1);
        }
        KeyCode::Char('a') | KeyCode::Left => {
            let mut bm = world.resource_mut::<BuildMode>();
            bm.cursor.x = bm.cursor.x.saturating_sub(1);
        }
        KeyCode::Char('d') | KeyCode::Right => {
            let mut bm = world.resource_mut::<BuildMode>();
            bm.cursor.x = bm.cursor.x.saturating_add(1);
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            let build_mode = world.resource::<BuildMode>();
            let cursor = build_mode.cursor;
            let building_type = build_mode.selected;
            try_place_building(world, cursor.x, cursor.y, building_type);
        }
        KeyCode::Tab => {
            let mut build_mode = world.resource_mut::<BuildMode>();
            build_mode.selected = build_mode.selected.next();
        }
        _ => {}
    }
}

fn handle_overlay_mode(world: &mut World, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('c') => {
            world.resource_mut::<InputContextStack>().pop();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::Viewport;
    use crate::shared::state::GameState;
    use crate::shared::time::{SimSpeed, SimulationTime};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key_event(code: KeyCode) -> KeyEvent {
        // KeyEvent::new creates a Press event by default (in crossterm 0.28)
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_input_context_default() {
        let context = InputContext::default();
        assert_eq!(context, InputContext::Normal);
    }

    #[test]
    fn test_input_context_stack_push_pop() {
        let mut stack = InputContextStack::default();
        assert_eq!(stack.current(), InputContext::Normal);

        stack.push(InputContext::BuildMode);
        assert_eq!(stack.current(), InputContext::BuildMode);

        stack.push(InputContext::Overlay);
        assert_eq!(stack.current(), InputContext::Overlay);

        stack.pop();
        assert_eq!(stack.current(), InputContext::BuildMode);

        stack.pop();
        assert_eq!(stack.current(), InputContext::Normal);
    }

    #[test]
    fn test_cannot_pop_normal_context() {
        let mut stack = InputContextStack::default();
        assert_eq!(stack.current(), InputContext::Normal);

        // Popping normal should do nothing (always one context)
        stack.pop();
        assert_eq!(stack.current(), InputContext::Normal);
    }

    #[test]
    fn test_input_router_normal_mode_quit() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);
        world.insert_resource(InputContextStack::default());

        let mut router = InputRouter::new();
        router.route(&mut world, key_event(KeyCode::Char('q')));

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

        let mut router = InputRouter::new();
        router.route(&mut world, key_event(KeyCode::Char('q')));

        // 'q' in build mode should NOT quit (should exit build mode instead)
        assert_eq!(*world.resource::<GameState>(), GameState::Running);
    }

    #[test]
    fn test_input_router_overlay_escape_pops_context() {
        let mut world = World::new();
        let mut stack = InputContextStack::default();
        stack.push(InputContext::Overlay);
        world.insert_resource(stack);

        let mut router = InputRouter::new();
        router.route(&mut world, key_event(KeyCode::Esc));

        // Escape in overlay should pop back to normal
        assert_eq!(
            world.resource::<InputContextStack>().current(),
            InputContext::Normal
        );
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
        world.insert_resource(InputContextStack::default());
        world.insert_resource(BuildMode::default());

        let mut router = InputRouter::new();

        // Test all normal mode bindings work
        router.route(&mut world, key_event(KeyCode::Char(' ')));
        assert_eq!(*world.resource::<GameState>(), GameState::Paused);

        router.route(&mut world, key_event(KeyCode::Char('1')));
        assert_eq!(world.resource::<SimulationTime>().speed, SimSpeed::Normal);

        router.route(&mut world, key_event(KeyCode::Char('w')));
        assert_eq!(world.resource::<Viewport>().y, -1);
    }

    #[test]
    fn test_viewport_overflow_safety() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);
        world.insert_resource(InputContextStack::default());
        world.insert_resource(Viewport {
            x: i32::MAX,
            y: i32::MIN,
        });

        let mut router = InputRouter::new();

        // Move right (x += 1) should wrap
        router.route(&mut world, key_event(KeyCode::Char('d')));
        assert_eq!(world.resource::<Viewport>().x, i32::MIN);

        // Move up (y -= 1) should wrap
        router.route(&mut world, key_event(KeyCode::Char('w')));
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

        let mut router = InputRouter::new();

        // Move right (x += 1) should saturate
        router.route(&mut world, key_event(KeyCode::Char('d')));
        assert_eq!(world.resource::<BuildMode>().cursor.x, i32::MAX);

        // Move up (y -= 1) should saturate
        router.route(&mut world, key_event(KeyCode::Char('w')));
        assert_eq!(world.resource::<BuildMode>().cursor.y, i32::MIN);
    }
}
