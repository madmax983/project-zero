# 094: System View Architecture

## Overview

Introduce the **System View** (Layer 2), allowing players to toggle between the colony surface and the planetary system map.

Currently, the game only renders the Colony (Layer 1). This spec adds:
1.  **ViewMode Resource**: Tracks whether the player is viewing the `Colony` or the `System`.
2.  **SystemMap Resource**: Logic for the system layer (orbital bodies).
3.  **OrbitalBody Component**: Entities that exist in the system view (Planets, Moons, Ships).
4.  **Orbit Component**: Defines the path of an orbital body around a parent.
5.  **Input Toggle**: Pressing `Tab` (or `M`) in Normal mode toggles between views.
6.  **Rendering Switch**: The main render loop switches between `render_colony` and `render_system` based on `ViewMode`.

This is the foundational architecture for Layer 2.

## Dependencies

- `001` — Project Scaffold (Main loop)
- `012` — Input Architecture (Context routing)
- `014` — Rendering Architecture (Render loop)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer2/system_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::system::{SystemMap, OrbitalBody, Orbit, ViewMode};
    use crate::shared::input::{InputRouter, InputContextStack, InputContext};
    use crate::platform::input::{GameKeyCode, GameKeyEvent};
    use crate::shared::state::GameState;

    fn key_event(code: GameKeyCode) -> GameKeyEvent {
        GameKeyEvent::new(code)
    }

    #[test]
    fn test_view_mode_resource() {
        let mut world = World::new();
        world.insert_resource(ViewMode::default());

        assert_eq!(*world.resource::<ViewMode>(), ViewMode::Colony);
    }

    #[test]
    fn test_orbital_body_component() {
        let mut world = World::new();
        let entity = world.spawn(OrbitalBody {
            name: "Planet".to_string(),
            radius: 10.0,
            color: ratatui::style::Color::Blue,
            char: 'O',
        }).id();

        let body = world.get::<OrbitalBody>(entity).unwrap();
        assert_eq!(body.name, "Planet");
        assert_eq!(body.char, 'O');
    }

    #[test]
    fn test_orbit_component() {
        let mut world = World::new();
        let sun = world.spawn_empty().id();
        let planet = world.spawn(Orbit {
            parent: sun,
            radius: 100.0,
            speed: 0.1,
            angle: 0.0,
        }).id();

        let orbit = world.get::<Orbit>(planet).unwrap();
        assert_eq!(orbit.parent, sun);
        assert_eq!(orbit.radius, 100.0);
    }

    #[test]
    fn test_toggle_view_mode_input() {
        let mut world = World::new();
        world.insert_resource(GameState::Running);
        world.insert_resource(ViewMode::Colony);

        // Setup Input Stack
        let mut stack = InputContextStack::default();
        stack.push(InputContext::Normal);
        world.insert_resource(stack);

        // Setup Router
        let mut router = InputRouter::new();

        // Press Tab to switch to System View
        router.route(&mut world, key_event(GameKeyCode::Tab));
        assert_eq!(*world.resource::<ViewMode>(), ViewMode::System);

        // Press Tab to switch back to Colony View
        router.route(&mut world, key_event(GameKeyCode::Tab));
        assert_eq!(*world.resource::<ViewMode>(), ViewMode::Colony);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Resources and Components

Create `src/layer2/system.rs`:

```rust
use bevy_ecs::prelude::*;
use ratatui::style::Color;

#[derive(Resource, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ViewMode {
    #[default]
    Colony,
    System,
}

#[derive(Component, Debug, Clone)]
pub struct OrbitalBody {
    pub name: String,
    pub radius: f32,
    pub color: Color,
    pub char: char,
}

#[derive(Component, Debug, Clone)]
pub struct Orbit {
    pub parent: Entity,
    pub radius: f32,
    pub speed: f32,
    pub angle: f32,
}

#[derive(Resource, Default)]
pub struct SystemMap;
```

### 2. Update Render Loop

Modify `src/ui/mod.rs`:

```rust
use crate::layer2::system::ViewMode;
// use crate::layer2::render::render_system_view; // To be implemented

pub fn render(world: &World, frame: &mut Frame) {
    // ... Main Menu check ...

    // Check ViewMode
    let view_mode = world.resource::<ViewMode>();

    match view_mode {
        ViewMode::Colony => {
            // ... existing colony render logic ...
            // render_map(...)
            // render_info_panel(...)
        }
        ViewMode::System => {
            // render_system_view(frame, frame.area(), world);
        }
    }

    // Status bar might be shared or different
    // render_status_bar(...)
}
```

### 3. Implement System View Rendering

Create `src/layer2/render.rs`:

```rust
use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph}};
use bevy_ecs::prelude::*;
use crate::layer2::system::{OrbitalBody, Orbit};

pub fn render_system_view(frame: &mut Frame, area: Rect, world: &World) {
    let block = Block::default().title(" System Map ").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Simple placeholder for bodies
    // Iterate OrbitalBodies and draw them based on Orbit.angle/radius
    // Project polar coordinates to screen (x, y)

    // Center is (inner.width/2, inner.height/2)
    // x = center_x + radius * cos(angle)
    // y = center_y + radius * sin(angle)
}
```

### 4. Update Input Routing

Modify `src/shared/input.rs` in `handle_normal_mode`:

```rust
use crate::layer2::system::ViewMode;

fn handle_normal_mode(world: &mut World, key: GameKeyEvent) {
    match key.code {
        // ...
        GameKeyCode::Tab => {
            let mut view_mode = world.resource_mut::<ViewMode>();
            *view_mode = match *view_mode {
                ViewMode::Colony => ViewMode::System,
                ViewMode::System => ViewMode::Colony,
            };
        }
        // ...
    }
}
```

## REFACTOR Phase: Quality & Design

- **Camera Abstraction**: Render logic currently assumes a `Viewport` for Colony. System View might need its own `SystemViewport` (zoom/pan).
- **Time Scale**: System simulation might tick differently than Colony simulation. Currently they share the main loop.
- **Data Driven**: Load system layout from a file or generator, don't hardcode orbits.

## Acceptance Criteria

- [ ] `ViewMode` resource exists and defaults to `Colony`.
- [ ] `OrbitalBody` and `Orbit` components exist.
- [ ] Pressing `Tab` toggles between Colony and System views.
- [ ] `render` function dispatches to the correct renderer.
- [ ] System View renders *something* (e.g. a placeholder title or a sun).
- [ ] Tests pass.

## Technical Guidance

- Create `src/layer2/mod.rs` to export `system` and `render`.
- Ensure `ViewMode` is inserted into the World in `setup_world` (or via `init_resource` derived Default).
- `render_system_view` needs to handle coordinate projection carefully (terminal cells are non-square, typically 1:2 aspect ratio). Multiply X by 2 or divide Y by 2 to make circles look circular.
