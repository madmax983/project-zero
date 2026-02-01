# 003: Camera Controls (Pan, Zoom)

## Overview

Enable the player to navigate the map with keyboard panning and scroll wheel zoom. Essential for exploring the colony.

## Dependencies

- `001` — Project scaffold
- `002` — Terrain grid (need something to look at)

## Requirements

### Must Have
- WASD or Arrow keys to pan camera
- Mouse scroll wheel to zoom in/out
- Zoom limits: min 0.5x, max 4.0x
- Pan speed scales with zoom level (zoomed out = faster pan)
- Camera starts centered on grid

### Must NOT Have
- Camera bounds/clamping (can fly off into void—that's fine for now)
- Minimap
- Edge-of-screen scrolling

## Technical Guidance

### Camera Setup

Bevy 0.15 uses `Camera2d` with `OrthographicProjection`.

```rust
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        },
    ));
}
```

### Input Handling

```rust
fn camera_pan(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<(&mut Transform, &OrthographicProjection), With<Camera2d>>,
    time: Res<Time>,
) {
    let (mut transform, projection) = camera_query.single_mut();
    let speed = 300.0 * projection.scale; // Scale with zoom
    
    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    // ... etc
    
    direction = direction.normalize_or_zero();
    transform.translation.x += direction.x * speed * time.delta_secs();
    transform.translation.y += direction.y * speed * time.delta_secs();
}

fn camera_zoom(
    mut scroll_events: EventReader<MouseWheel>,
    mut camera_query: Query<&mut OrthographicProjection, With<Camera2d>>,
) {
    let mut projection = camera_query.single_mut();
    for event in scroll_events.read() {
        projection.scale -= event.y * 0.1;
        projection.scale = projection.scale.clamp(0.5, 4.0);
    }
}
```

### System Registration

Add to `Update` schedule, only run in `Playing` state.

## Acceptance Criteria

- [ ] WASD moves camera across terrain
- [ ] Arrow keys also work
- [ ] Scroll wheel zooms in and out
- [ ] Zoom has limits (doesn't go infinite)
- [ ] Panning feels faster when zoomed out
- [ ] `cargo check` passes

## Questions

*Builder: add questions here if spec is unclear.*
