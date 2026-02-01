# 012: Input Architecture and Context Routing

## Overview

Establish a centralized input handling system that routes key presses to the appropriate handler based on current UI context (normal mode, build mode, overlay open, etc.). This prevents input conflicts when multiple systems need keyboard control and provides a foundation for future keybinding customization.

## Dependencies

- `001` — Project scaffold (handle_input exists)
- `003` — UI layout (multiple UI states)
- `006` — Building placement (build mode context)
- `010` — Chronicle system (overlay context)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/shared/input.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key_event(code: KeyCode) -> KeyEvent {
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
        assert_eq!(world.resource::<InputContextStack>().current(), InputContext::Normal);
    }

    #[test]
    fn test_input_handler_registration() {
        let router = InputRouter::new();

        // Should have handlers for all contexts
        assert!(router.has_handler_for(InputContext::Normal));
        assert!(router.has_handler_for(InputContext::BuildMode));
        assert!(router.has_handler_for(InputContext::Overlay));
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

        let mut router = InputRouter::new();

        // Test all normal mode bindings work
        router.route(&mut world, key_event(KeyCode::Char(' ')));
        assert_eq!(*world.resource::<GameState>(), GameState::Paused);

        router.route(&mut world, key_event(KeyCode::Char('1')));
        assert_eq!(world.resource::<SimulationTime>().speed, SimSpeed::Normal);

        router.route(&mut world, key_event(KeyCode::Char('w')));
        assert_eq!(world.resource::<Viewport>().y, -1);
    }
}
```

**Test Coverage Requirements:**
- InputContext: all variants, stack operations
- InputContextStack: push, pop, boundary conditions (can't pop Normal)
- InputRouter: routing to correct handler based on context
- Context-specific behavior: keys behave differently per mode
- All tests must pass before spec is considered complete

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Input Context Types

```rust
// src/shared/input.rs

use bevy_ecs::prelude::*;
use crossterm::event::KeyEvent;

/// Defines the current input handling context.
/// Input is routed to different handlers based on the active context.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum InputContext {
    /// Normal game mode - all default keybindings active.
    #[default]
    Normal,

    /// Building placement mode - WASD moves cursor, not viewport.
    BuildMode,

    /// Modal overlay open (Chronicle, help screen, etc.) - most keys disabled.
    Overlay,
}

/// Stack-based input context manager.
/// Contexts are pushed/popped as UI modes activate/deactivate.
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
    /// Get the current active context (top of stack).
    #[must_use]
    pub fn current(&self) -> InputContext {
        *self.stack.last().unwrap() // Stack always has at least Normal
    }

    /// Push a new context onto the stack.
    pub fn push(&mut self, context: InputContext) {
        self.stack.push(context);
    }

    /// Pop the current context, returning to the previous one.
    /// Cannot pop the base Normal context.
    pub fn pop(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    /// Check if a specific context is anywhere in the stack.
    #[must_use]
    pub fn contains(&self, context: InputContext) -> bool {
        self.stack.contains(&context)
    }
}
```

### Input Router

```rust
// src/shared/input.rs

use std::collections::HashMap;

type InputHandler = fn(&mut World, KeyEvent);

/// Routes input to context-appropriate handlers.
pub struct InputRouter {
    handlers: HashMap<InputContext, InputHandler>,
}

impl InputRouter {
    #[must_use]
    pub fn new() -> Self {
        let mut handlers = HashMap::new();
        handlers.insert(InputContext::Normal, handle_normal_mode as InputHandler);
        handlers.insert(InputContext::BuildMode, handle_build_mode as InputHandler);
        handlers.insert(InputContext::Overlay, handle_overlay_mode as InputHandler);

        Self { handlers }
    }

    /// Route input to the appropriate handler based on current context.
    pub fn route(&mut self, world: &mut World, key: KeyEvent) {
        let context = world.resource::<InputContextStack>().current();

        if let Some(handler) = self.handlers.get(&context) {
            handler(world, key);
        }
    }

    #[must_use]
    pub fn has_handler_for(&self, context: InputContext) -> bool {
        self.handlers.contains_key(&context)
    }
}

impl Default for InputRouter {
    fn default() -> Self {
        Self::new()
    }
}
```

### Context-Specific Handlers

```rust
// src/shared/input.rs

use crossterm::event::KeyCode;
use crate::GameState;
use super::time::{SimSpeed, SimulationTime};
use crate::layer1::Viewport;

/// Handle input in normal game mode.
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
            world.resource_mut::<Viewport>().y -= 1;
        }
        KeyCode::Char('s') | KeyCode::Down => {
            world.resource_mut::<Viewport>().y += 1;
        }
        KeyCode::Char('a') | KeyCode::Left => {
            world.resource_mut::<Viewport>().x -= 1;
        }
        KeyCode::Char('d') | KeyCode::Right => {
            world.resource_mut::<Viewport>().x += 1;
        }
        KeyCode::Char('b') => {
            // Toggle build mode (if spec 006 implemented)
            world.resource_mut::<InputContextStack>().push(InputContext::BuildMode);
        }
        KeyCode::Char('c') => {
            // Open chronicle (if spec 010 implemented)
            world.resource_mut::<InputContextStack>().push(InputContext::Overlay);
        }
        _ => {}
    }
}

/// Handle input in building placement mode.
fn handle_build_mode(world: &mut World, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('b') => {
            // Exit build mode
            world.resource_mut::<InputContextStack>().pop();
        }
        KeyCode::Char('w') | KeyCode::Up => {
            // Move cursor up (spec 006 will implement BuildMode resource)
            // For now, just consume the input
        }
        KeyCode::Char('s') | KeyCode::Down => {
            // Move cursor down
        }
        KeyCode::Char('a') | KeyCode::Left => {
            // Move cursor left
        }
        KeyCode::Char('d') | KeyCode::Right => {
            // Move cursor right
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            // Place building
        }
        KeyCode::Tab => {
            // Cycle building type
        }
        _ => {}
    }
}

/// Handle input when modal overlay is open.
fn handle_overlay_mode(world: &mut World, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('c') => {
            // Close overlay
            world.resource_mut::<InputContextStack>().pop();
        }
        KeyCode::Up | KeyCode::Down => {
            // Scroll overlay content (future: overlay state)
        }
        _ => {
            // Most keys ignored in overlay mode
        }
    }
}
```

### Integration with Main

```rust
// src/main.rs - Replace handle_input() completely

use scale::shared::input::{InputRouter, InputContextStack};

fn main() -> anyhow::Result<()> {
    // ... terminal setup ...

    let mut world = World::new();
    world.insert_resource(GameState::Running);
    world.insert_resource(generate_terrain(80, 50));
    world.insert_resource(Viewport::default());
    world.insert_resource(SimulationTime::default());
    world.insert_resource(InputContextStack::default()); // ADD THIS

    let mut schedule = Schedule::default();
    let mut input_router = InputRouter::new(); // ADD THIS

    // Main loop
    loop {
        // Input - use router instead of handle_input
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                input_router.route(&mut world, key); // CHANGED
            }
        }

        // ... rest of main loop ...
    }
}

// REMOVE old handle_input() function - it's replaced by InputRouter
```

### Module Integration

```rust
// src/shared/mod.rs
pub mod time;
pub mod input; // ADD THIS
```

```rust
// src/lib.rs
pub use shared::input::{InputContext, InputContextStack, InputRouter};
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1. **Hardcoded keybindings**: All keys are in match statements
   - Future: Create KeyBinding config system
   - Allow user customization via config file
   - Current hardcoded bindings acceptable for MVP

2. **Handler functions not extensible**: Adding new handlers requires modifying InputRouter::new()
   - Future: Use macro or builder pattern for registration
   - Current HashMap approach simple and works

3. **No input history/recording**: Can't replay inputs or show recent commands
   - Future: Add input buffer for debugging/macros
   - Current immediate dispatch is fine

4. **Context transitions not validated**: Can push same context twice
   - Future: Add validation or use state machine
   - Current stack allows duplicates (may be intentional)

5. **No modifier key support**: Can't do Ctrl+S, Shift+arrows, etc.
   - Future: Check KeyModifiers in handlers
   - Current single-key bindings sufficient for colony sim

### Performance Considerations

- **HashMap lookup per keypress**: O(1), negligible overhead
- **Context stack is Vec**: Small (typically 1-3 items), cheap to clone
- **Function pointers**: Zero runtime cost vs vtable dispatch

### API Design Notes

- `InputContext` is Copy - cheap to pass, store in collections
- `InputContextStack` prevents popping below Normal - always valid state
- Handlers take `&mut World` - can access any resource
- Router is stateful (owns handlers) - could be global/singleton

### Future Extensibility

When adding keybinding customization:
```rust
pub struct KeyBinding {
    key: KeyCode,
    modifiers: KeyModifiers,
    command: Command,
}

pub enum Command {
    Quit,
    Pause,
    SetSpeed(SimSpeed),
    MoveViewport(i32, i32),
    // ...
}
```

When adding input macros/recording:
```rust
pub struct InputRecorder {
    history: Vec<(Instant, KeyEvent)>,
    is_recording: bool,
}
```

When adding context guards:
```rust
impl InputContextStack {
    pub fn push_unique(&mut self, ctx: InputContext) {
        if self.current() != ctx {
            self.push(ctx);
        }
    }
}
```

## Acceptance Criteria (Testable!)

- [x] All tests in RED phase pass
- [x] `cargo test` returns 0 failures
- [x] `cargo clippy -- -D warnings` passes
- [x] Test coverage ≥85% for shared/input.rs
- [x] InputContextStack resource exists and manages contexts
- [x] InputRouter routes keys based on context
- [x] Normal mode: all default keybindings work
- [x] Build mode: 'b' enters, Esc exits, WASD moves cursor not viewport
- [x] Overlay mode: 'c' enters, Esc exits, most keys ignored
- [x] Cannot pop Normal context (always at least one context)
- [x] Input handling decoupled from main.rs

## Technical Guidance

### Context Stack Behavior

```
Initial: [Normal]
Press 'b': [Normal, BuildMode]
Press 'c' (error - c ignored in build mode)
Press Esc: [Normal]
Press 'c': [Normal, Overlay]
Press Esc: [Normal]
```

### Keybinding Conflicts Resolution

When same key has different meanings:
- **'b'**: Normal mode → enter build | Build mode → exit build
- **WASD**: Normal mode → scroll viewport | Build mode → move cursor
- **Escape**: Normal mode → quit | Build/Overlay → pop context

Context stack ensures correct handler is called.

### Integration with Future Specs

**Spec 006 (Building Placement)** should:
- Add BuildMode resource with cursor position
- Implement actual cursor movement in handle_build_mode
- Add building placement on Space/Enter

**Spec 010 (Chronicle)** should:
- Add ChronicleUiState resource
- Implement scrolling in handle_overlay_mode
- Toggle chronicle visibility on 'c' key

**Future keybinding spec** could:
- Load key config from file
- Allow runtime rebinding
- Show keybinding help screen

### Common Pitfalls

1. **Forgetting to push/pop contexts**: UI state and input state must stay synchronized
2. **Popping too many times**: Stack protects against this (min 1 item)
3. **Not checking context in rendering**: Build cursor only shows if BuildMode active
4. **Assuming single context**: Stack allows overlapping (build → help overlay → back to build)

## Questions

*Builder: add questions here if spec is unclear.*
