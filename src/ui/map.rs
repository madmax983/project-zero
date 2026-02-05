use bevy_ecs::prelude::*;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Paragraph},
};
use std::collections::HashMap;
use std::hash::BuildHasher;

use crate::layer1::{
    BuildMode, Building, BuildingType, Designation, DesignationMode, DesignationType, GridPosition,
    Needs, TerrainGrid, TerrainType, Viewport,
};

/// Cache for renderable entities to avoid repeated allocations and iterations.
#[derive(Resource, Default)]
pub struct RenderCache {
    /// Cached pop display data.
    pub pops: HashMap<GridPosition, (&'static str, Color)>,
    /// Cached building data.
    pub buildings: HashMap<GridPosition, BuildingType>,
    /// Cached designation data.
    pub designations: HashMap<GridPosition, DesignationType>,
}

/// Updates the `RenderCache` by iterating the world once.
pub fn update_render_cache(world: &mut World) {
    let mut cache = world.remove_resource::<RenderCache>().unwrap_or_default();

    cache.pops.clear();
    cache.buildings.clear();
    cache.designations.clear();

    for e in world.iter_entities() {
        if let Some(pos) = e.get::<GridPosition>() {
            // Check for Pop (via Needs)
            if let Some(needs) = e.get::<Needs>() {
                cache.pops.insert(*pos, get_pop_display(needs));
            }

            // Check for Building
            if let Some(building) = e.get::<Building>() {
                cache.buildings.insert(*pos, building.building_type);
            }

            // Check for Designation
            if let Some(designation) = e.get::<Designation>() {
                cache
                    .designations
                    .insert(*pos, designation.designation_type);
            }
        }
    }

    world.insert_resource(cache);
}

/// Context for rendering the map layer.
pub struct MapRenderContext<'a, S: BuildHasher> {
    /// The area to render into.
    pub area: Rect,
    /// The terrain grid.
    pub terrain: &'a TerrainGrid,
    /// The viewport.
    pub viewport: &'a Viewport,
    /// Map of pop positions to their display char and color.
    pub pops_data: &'a HashMap<GridPosition, (&'static str, Color), S>,
    /// Map of building positions.
    pub buildings_data: &'a HashMap<GridPosition, BuildingType, S>,
    /// Map of designation positions.
    pub designations_data: &'a HashMap<GridPosition, DesignationType, S>,
    /// Current build mode state (cursor position, selected building, valid placement).
    pub build_mode: Option<(GridPosition, BuildingType, bool)>,
    /// Current designation mode state (cursor position, selected tool, valid placement).
    pub designation_mode: Option<(GridPosition, DesignationType, bool)>,
}

impl<S: BuildHasher> Clone for MapRenderContext<'_, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: BuildHasher> Copy for MapRenderContext<'_, S> {}

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
                    terrain.get(ux, uy).map_or((" ", Color::Black), |tile| {
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
                // Convert char to string slice is tricky without allocation if we want static str.
                // But span accepts Cow/String.
                // We can just use a 1-char string or format.
                // Or better, change get_building_char to return &'static str for consistency.
                line_spans.push(Span::styled(
                    text.to_string(),
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
                line_spans.push(Span::styled(text, Style::default().fg(Color::White).bg(bg)));
                continue;
            }

            // Designations
            if let Some(designation_type) = ctx.designations_data.get(&GridPosition {
                x: world_x,
                y: world_y,
            }) {
                let color = Color::Red; // Standardize designation color as red
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
                    get_building_char(*building_type).to_string(),
                    Style::default().fg(get_building_color(*building_type)),
                ));
                continue;
            }

            // Check for pop
            if let Some((text, color)) = ctx.pops_data.get(&GridPosition {
                x: world_x,
                y: world_y,
            }) {
                line_spans.push(Span::styled(*text, Style::default().fg(*color)));
                continue;
            }

            // Otherwise render terrain
            let (text, color) =
                if let (Ok(ux), Ok(uy)) = (usize::try_from(world_x), usize::try_from(world_y)) {
                    ctx.terrain.get(ux, uy).map_or((" ", Color::Black), |tile| {
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

/// Renders the full map with borders and simulation state.
pub fn render_map(frame: &mut Frame, area: Rect, world: &World) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Colony ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Render terrain inside
    let terrain = world.resource::<TerrainGrid>();
    let viewport = world.resource::<Viewport>();
    let build_mode = world.resource::<BuildMode>();
    let designation_mode = world.resource::<DesignationMode>();
    let render_cache = world.resource::<RenderCache>();

    // Build mode cursor info
    let build_mode_cursor = if build_mode.active {
        let can_place =
            crate::layer1::can_place_building(world, build_mode.cursor.x, build_mode.cursor.y);
        Some((build_mode.cursor, build_mode.selected, can_place))
    } else {
        None
    };

    // Designation mode cursor info
    let designation_mode_cursor = if designation_mode.active {
        let can = crate::layer1::can_designate(
            world,
            designation_mode.cursor.x,
            designation_mode.cursor.y,
            designation_mode.tool,
        );
        Some((designation_mode.cursor, designation_mode.tool, can))
    } else {
        None
    };

    let ctx = MapRenderContext {
        area: inner,
        terrain,
        viewport,
        pops_data: &render_cache.pops,
        buildings_data: &render_cache.buildings,
        designations_data: &render_cache.designations,
        build_mode: build_mode_cursor,
        designation_mode: designation_mode_cursor,
    };

    render_map_layer(frame, ctx);
}

/// Renders the terrain grid to the provided frame.
pub fn render_terrain(frame: &mut Frame, area: Rect, terrain: &TerrainGrid, viewport: &Viewport) {
    let spans = build_terrain_spans(area, terrain, viewport);
    let paragraph = Paragraph::new(spans);
    frame.render_widget(paragraph, area);
}

// --- Visual Helpers ---

#[must_use]
pub const fn get_terrain_char(terrain: TerrainType) -> &'static str {
    match terrain {
        TerrainType::Grass => ".",
        TerrainType::Dirt => ",",
        TerrainType::Rock => "#",
        TerrainType::Water => "~",
        TerrainType::Tree => "↑",
    }
}

#[must_use]
pub const fn get_terrain_color(terrain: TerrainType) -> Color {
    match terrain {
        TerrainType::Grass => Color::Green,
        TerrainType::Dirt => Color::Rgb(139, 90, 43),
        TerrainType::Rock => Color::DarkGray,
        TerrainType::Water => Color::Blue,
        TerrainType::Tree => Color::Rgb(0, 100, 0),
    }
}

#[must_use]
pub const fn get_building_char(building: BuildingType) -> char {
    match building {
        BuildingType::Housing => '⌂',
        BuildingType::Farm => '♣',
        BuildingType::Stockpile => '≡',
    }
}

#[must_use]
pub const fn get_building_color(building: BuildingType) -> Color {
    match building {
        BuildingType::Housing => Color::Rgb(139, 90, 43), // Brown
        BuildingType::Farm => Color::Rgb(218, 165, 32),   // Goldenrod
        BuildingType::Stockpile => Color::Rgb(169, 169, 169), // DarkGray
    }
}

#[must_use]
pub const fn get_designation_char(tool: DesignationType) -> &'static str {
    match tool {
        DesignationType::Mine => "⛏",
        DesignationType::Demolish => "X",
        DesignationType::Chop => "🪓",
    }
}

const HEALTHY_THRESHOLD: f32 = 0.6;
const WARNING_THRESHOLD: f32 = 0.3;

/// Returns the character and color for rendering a pop.
#[must_use]
pub fn get_pop_display(needs: &Needs) -> (&'static str, Color) {
    let health = needs.worst();
    if health > HEALTHY_THRESHOLD {
        ("☺", Color::Yellow)
    } else if health > WARNING_THRESHOLD {
        ("☻", Color::Rgb(255, 165, 0))
    } else {
        ("☹", Color::Red)
    }
}
