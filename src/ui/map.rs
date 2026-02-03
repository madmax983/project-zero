//! Map rendering logic.
//!
//! This module handles rendering the terrain, buildings, pops, and cursors.

use crate::layer1::{
    BuildingType, DesignationType, GridPosition, Needs, TerrainGrid, Viewport,
};
use crate::ui::colors::{
    get_building_color, get_designation_color, get_pop_color, get_terrain_color,
};
use crate::ui::sprites::{
    get_building_char, get_designation_char, get_pop_char, get_terrain_char,
};
use ratatui::{prelude::*, widgets::Paragraph};
use std::collections::HashMap;
use std::hash::BuildHasher;

/// Context for rendering the map layer.
#[derive(Clone, Copy)]
pub struct MapRenderContext<'a, S: BuildHasher> {
    /// The area to render into.
    pub area: Rect,
    /// The terrain grid.
    pub terrain: &'a TerrainGrid,
    /// The viewport.
    pub viewport: &'a Viewport,
    /// Map of pop positions to their needs (for color/char determination).
    pub pops_data: &'a HashMap<GridPosition, Needs, S>,
    /// Map of building positions.
    pub buildings_data: &'a HashMap<GridPosition, BuildingType, S>,
    /// Map of designation positions.
    pub designations_data: &'a HashMap<GridPosition, DesignationType, S>,
    /// Current build mode state (cursor position, selected building, valid placement).
    pub build_mode: Option<(GridPosition, BuildingType, bool)>,
    /// Current designation mode state (cursor position, selected tool, valid placement).
    pub designation_mode: Option<(GridPosition, DesignationType, bool)>,
}

/// Builds a vector of text lines to render the terrain within the given area.
#[must_use]
pub fn build_terrain_spans(
    area: Rect,
    terrain: &TerrainGrid,
    viewport: &Viewport,
) -> Vec<Line<'static>> {
    let mut spans: Vec<Line> = Vec::with_capacity(area.height as usize);

    for screen_y in 0..area.height {
        let world_y = viewport.y + i32::from(screen_y);
        let mut line_spans = Vec::with_capacity(area.width as usize);

        for screen_x in 0..area.width {
            let world_x = viewport.x + i32::from(screen_x);

            let (text, color) =
                if let (Ok(ux), Ok(uy)) = (usize::try_from(world_x), usize::try_from(world_y)) {
                    terrain
                        .get(ux, uy)
                        .map_or((" ", Color::Black), |tile| {
                            (get_terrain_char(tile), get_terrain_color(tile))
                        })
                } else {
                    (" ", Color::Black)
                };

            line_spans.push(Span::styled(text, Style::default().fg(color)));
        }
        spans.push(Line::from(line_spans));
    }
    spans
}

/// Builds a vector of text lines to render the map layer (terrain, pops, buildings, cursor).
#[must_use]
pub fn build_map_layer_spans<S: BuildHasher>(ctx: MapRenderContext<'_, S>) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = Vec::with_capacity(ctx.area.height as usize);

    for screen_y in 0..ctx.area.height {
        let world_y = ctx.viewport.y + i32::from(screen_y);
        let mut line_spans = Vec::with_capacity(ctx.area.width as usize);

        for screen_x in 0..ctx.area.width {
            let world_x = ctx.viewport.x + i32::from(screen_x);

            // Build mode cursor (highest priority)
            if let Some((_, selected, can_place)) = ctx
                .build_mode
                .filter(|(cursor, _, _)| cursor.x == world_x && cursor.y == world_y)
            {
                let bg = if can_place { Color::Green } else { Color::Red };
                let text = get_building_char(selected);
                line_spans.push(Span::styled(
                    text,
                    Style::default().fg(Color::White).bg(bg),
                ));
                continue;
            }

            // Designation mode cursor (highest priority, shared with build mode)
            if let Some((_, selected, can_place)) = ctx
                .designation_mode
                .filter(|(cursor, _, _)| cursor.x == world_x && cursor.y == world_y)
            {
                let bg = if can_place { Color::Green } else { Color::Red };
                let text = get_designation_char(selected);
                line_spans.push(Span::styled(
                    text,
                    Style::default().fg(Color::White).bg(bg),
                ));
                continue;
            }

            // Designations
            if let Some(designation_type) = ctx.designations_data.get(&GridPosition {
                x: world_x,
                y: world_y,
            }) {
                let color = get_designation_color(*designation_type);
                line_spans.push(Span::styled(
                    get_designation_char(*designation_type),
                    Style::default().fg(color),
                ));
                continue;
            }

            // Buildings
            if let Some(building_type) = ctx.buildings_data.get(&GridPosition {
                x: world_x,
                y: world_y,
            }) {
                line_spans.push(Span::styled(
                    get_building_char(*building_type),
                    Style::default().fg(get_building_color(*building_type)),
                ));
                continue;
            }

            // Check for pop
            if let Some(needs) = ctx.pops_data.get(&GridPosition {
                x: world_x,
                y: world_y,
            }) {
                line_spans.push(Span::styled(
                    get_pop_char(needs),
                    Style::default().fg(get_pop_color(needs)),
                ));
                continue;
            }

            // Otherwise render terrain
            let (text, color) =
                if let (Ok(ux), Ok(uy)) = (usize::try_from(world_x), usize::try_from(world_y)) {
                    ctx.terrain
                        .get(ux, uy)
                        .map_or((" ", Color::Black), |tile| {
                            (get_terrain_char(tile), get_terrain_color(tile))
                        })
                } else {
                    (" ", Color::Black)
                };

            line_spans.push(Span::styled(text, Style::default().fg(color)));
        }
        lines.push(Line::from(line_spans));
    }
    lines
}

/// Render terrain grid, buildings, and pops to the given frame area with viewport offset.
pub fn render_map_layer<S: BuildHasher>(frame: &mut Frame, ctx: MapRenderContext<'_, S>) {
    let area = ctx.area;
    let lines = build_map_layer_spans(ctx);
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

/// Renders the terrain grid to the provided frame.
pub fn render_terrain(frame: &mut Frame, area: Rect, terrain: &TerrainGrid, viewport: &Viewport) {
    let spans = build_terrain_spans(area, terrain, viewport);
    let paragraph = Paragraph::new(spans);
    frame.render_widget(paragraph, area);
}
