//! Rendering logic for Layer 2: System View.

use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph}};
use bevy_ecs::prelude::*;
use crate::layer2::system::{OrbitalBody, Orbit};

/// Renders the System View (Layer 2).
///
/// This function is called when `ViewMode` is `System`.
/// It renders the system map, including the sun and orbital bodies.
///
/// # Arguments
///
/// * `frame` - The ratatui frame to render into.
/// * `area` - The screen area to render within.
/// * `world` - The ECS world containing system state.
pub fn render_system_view(frame: &mut Frame, area: Rect, world: &World) {
    let block = Block::default().title(" System Map ").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Simple placeholder for bodies
    // Iterate OrbitalBodies and draw them based on Orbit.angle/radius
    // Project polar coordinates to screen (x, y)

    // Center is (inner.width/2, inner.height/2)
    let center_x = inner.x + inner.width / 2;
    let center_y = inner.y + inner.height / 2;

    // Draw the sun (center)
    let sun_pos = (center_x, center_y);
    if inner.contains(ratatui::layout::Position { x: sun_pos.0, y: sun_pos.1 }) {
        frame.buffer_mut().set_string(
            sun_pos.0,
            sun_pos.1,
            "☼", // Sun symbol
            Style::default().fg(Color::Yellow),
        );
    }

    // Draw orbital bodies
    for entity in world.iter_entities() {
        if let Some(body) = entity.get::<OrbitalBody>() {
            if let Some(orbit) = entity.get::<Orbit>() {
                // Simple polar to cartesian projection
                // Terminal cells are roughly 1:2 aspect ratio, so multiply X by 2 for circular appearance
                let radius_x = orbit.radius * 2.0;
                let radius_y = orbit.radius;

                let x = center_x as f32 + radius_x * orbit.angle.cos();
                let y = center_y as f32 + radius_y * orbit.angle.sin();

                // Cast to i32 for safe comparison before casting to u16
                let x_i32 = x.round() as i32;
                let y_i32 = y.round() as i32;

                if x_i32 >= 0 && y_i32 >= 0 {
                    let pos_x = x_i32 as u16;
                    let pos_y = y_i32 as u16;

                    if inner.contains(ratatui::layout::Position { x: pos_x, y: pos_y }) {
                        frame.buffer_mut().set_string(
                            pos_x,
                            pos_y,
                            body.char.to_string(),
                            Style::default().fg(body.color),
                        );
                    }
                }
            }
        }
    }

    // Instructions
    let instructions = "Press [Tab] to return to Colony View";
    frame.render_widget(
        Paragraph::new(instructions).alignment(Alignment::Center),
        Rect::new(area.x, area.bottom() - 1, area.width, 1),
    );
}
