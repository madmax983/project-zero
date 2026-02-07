use bevy_ecs::prelude::*;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Paragraph},
};
use std::collections::HashMap;
use std::hash::BuildHasher;

use crate::layer1::{
    BuildMode, Building, BuildingType, Designation, DesignationMode, DesignationType, GridPosition,
    Needs, ResourceItem, ResourceType, TerrainGrid, TerrainType, Viewport,
};

/// Represents a renderable entity on the map.
#[derive(Clone, Copy, Debug)]
pub enum RenderEntity {
    Designation(DesignationType),
    Building(BuildingType),
    Pop(&'static str, Color),
    Item(ResourceType),
}

impl RenderEntity {
    /// Returns the rendering priority (higher is drawn on top).
    const fn priority(&self) -> u8 {
        match self {
            Self::Designation(_) => 4,
            Self::Building(_) => 3,
            Self::Pop(_, _) => 2,
            Self::Item(_) => 1,
        }
    }
}

/// Cache for renderable entities to avoid repeated allocations and iterations.
///
/// Stores only the highest priority entity for each grid position to minimize lookups during rendering.
#[derive(Resource, Default)]
pub struct RenderCache {
    /// Cached entity data.
    pub entities: HashMap<GridPosition, RenderEntity>,
}

/// Updates the `RenderCache` by iterating the world once.
pub fn update_render_cache(world: &mut World) {
    let mut cache = world.remove_resource::<RenderCache>().unwrap_or_default();

    cache.entities.clear();

    for e in world.iter_entities() {
        if let Some(pos) = e.get::<GridPosition>() {
            // Check for Designation
            if let Some(designation) = e.get::<Designation>() {
                insert_if_higher_priority(
                    &mut cache.entities,
                    *pos,
                    RenderEntity::Designation(designation.designation_type),
                );
            }

            // Check for Building
            if let Some(building) = e.get::<Building>() {
                insert_if_higher_priority(
                    &mut cache.entities,
                    *pos,
                    RenderEntity::Building(building.building_type),
                );
            }

            // Check for Pop (via Needs)
            if let Some(needs) = e.get::<Needs>() {
                let (text, color) = get_pop_display(needs);
                insert_if_higher_priority(
                    &mut cache.entities,
                    *pos,
                    RenderEntity::Pop(text, color),
                );
            }

            // Check for ResourceItem
            if let Some(item) = e.get::<ResourceItem>() {
                insert_if_higher_priority(
                    &mut cache.entities,
                    *pos,
                    RenderEntity::Item(item.resource_type),
                );
            }
        }
    }

    world.insert_resource(cache);
}

fn insert_if_higher_priority(
    map: &mut HashMap<GridPosition, RenderEntity>,
    pos: GridPosition,
    entity: RenderEntity,
) {
    map.entry(pos)
        .and_modify(|e| {
            if entity.priority() > e.priority() {
                *e = entity;
            }
        })
        .or_insert(entity);
}

/// Context for rendering the map layer.
pub struct MapRenderContext<'a, S: BuildHasher> {
    /// The area to render into.
    pub area: Rect,
    /// The terrain grid.
    pub terrain: &'a TerrainGrid,
    /// The viewport.
    pub viewport: &'a Viewport,
    /// Map of entity positions to their render data.
    pub entities_data: &'a HashMap<GridPosition, RenderEntity, S>,
    /// Current build mode state (cursor position, selected building, valid placement).
    pub build_mode: Option<(GridPosition, BuildingType, bool)>,
    /// Current designation mode state (cursor position, selected tool, valid placement, drag start).
    pub designation_mode: Option<(GridPosition, DesignationType, bool, Option<GridPosition>)>,
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
#[allow(clippy::too_many_lines)]
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
                    text.to_string(),
                    Style::default().fg(Color::White).bg(bg),
                ));
                continue;
            }

            // Designation mode cursor (highest priority, shared with build mode)
            if let Some((cursor, selected, can_place, drag_start)) = ctx.designation_mode {
                if cursor.x == world_x && cursor.y == world_y {
                    let bg = if can_place { Color::Green } else { Color::Red };
                    let text = get_designation_char(selected);
                    line_spans.push(Span::styled(text, Style::default().fg(Color::White).bg(bg)));
                    continue;
                }

                // Rectangle preview: highlight tiles from drag_start to cursor
                if let Some(start) = drag_start {
                    let min_x = start.x.min(cursor.x);
                    let max_x = start.x.max(cursor.x);
                    let min_y = start.y.min(cursor.y);
                    let max_y = start.y.max(cursor.y);

                    if world_x >= min_x && world_x <= max_x && world_y >= min_y && world_y <= max_y
                    {
                        // Render terrain underneath with a highlight background
                        let (text, fg) = if let (Ok(ux), Ok(uy)) =
                            (usize::try_from(world_x), usize::try_from(world_y))
                        {
                            ctx.terrain.get(ux, uy).map_or((" ", Color::Black), |tile| {
                                (get_terrain_char(tile), get_terrain_color(tile))
                            })
                        } else {
                            (" ", Color::Black)
                        };
                        line_spans.push(Span::styled(
                            text,
                            Style::default().fg(fg).bg(Color::Rgb(50, 50, 80)),
                        ));
                        continue;
                    }
                }
            }

            // Check for entity in cache
            if let Some(entity) = ctx.entities_data.get(&GridPosition {
                x: world_x,
                y: world_y,
            }) {
                match entity {
                    RenderEntity::Designation(tool) => {
                        let color = Color::Red; // Standardize designation color as red
                        line_spans.push(Span::styled(
                            get_designation_char(*tool),
                            Style::default().fg(color),
                        ));
                        continue;
                    }
                    RenderEntity::Building(b) => {
                        line_spans.push(Span::styled(
                            get_building_char(*b).to_string(),
                            Style::default().fg(get_building_color(*b)),
                        ));
                        continue;
                    }
                    RenderEntity::Pop(text, color) => {
                        line_spans.push(Span::styled(*text, Style::default().fg(*color)));
                        continue;
                    }
                    RenderEntity::Item(r) => {
                        line_spans.push(Span::styled(
                            get_resource_char(*r),
                            Style::default().fg(get_resource_color(*r)),
                        ));
                        continue;
                    }
                }
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
        Some((
            designation_mode.cursor,
            designation_mode.tool,
            can,
            designation_mode.drag_start,
        ))
    } else {
        None
    };

    let ctx = MapRenderContext {
        area: inner,
        terrain,
        viewport,
        entities_data: &render_cache.entities,
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
        TerrainType::Grass => Color::Rgb(100, 200, 100),
        TerrainType::Dirt => Color::Rgb(205, 150, 80),
        TerrainType::Rock => Color::Rgb(160, 160, 170),
        TerrainType::Water => Color::Rgb(80, 140, 255),
        TerrainType::Tree => Color::Rgb(50, 180, 50),
    }
}

#[must_use]
pub const fn get_building_char(building: BuildingType) -> char {
    match building {
        BuildingType::Housing => '⌂',
        BuildingType::Farm => '♣',
        BuildingType::Stockpile => '≡',
        BuildingType::LumberMill => 'L',
        BuildingType::StoneMason => 'M',
        BuildingType::Smelter => 'S',
        BuildingType::Smithy => '⚒',
        BuildingType::Tavern => 'T',
        BuildingType::Library => 'K',
    }
}

#[must_use]
pub const fn get_building_color(building: BuildingType) -> Color {
    match building {
        BuildingType::Housing => Color::Rgb(139, 90, 43), // Brown
        BuildingType::Farm => Color::Rgb(218, 165, 32),   // Goldenrod
        BuildingType::Stockpile => Color::Rgb(169, 169, 169), // DarkGray
        BuildingType::LumberMill => Color::Rgb(205, 133, 63), // Peru
        BuildingType::StoneMason => Color::Rgb(119, 136, 153), // LightSlateGray
        BuildingType::Smelter => Color::Rgb(255, 69, 0),  // Red-Orange
        BuildingType::Smithy => Color::Rgb(192, 192, 192), // Silver
        BuildingType::Tavern => Color::Magenta,
        BuildingType::Library => Color::Cyan,
    }
}

#[must_use]
pub const fn get_designation_char(tool: DesignationType) -> &'static str {
    match tool {
        DesignationType::Mine => "%",
        DesignationType::Demolish => "X",
        DesignationType::Chop => "/",
    }
}

#[must_use]
pub const fn get_resource_char(resource: ResourceType) -> &'static str {
    match resource {
        ResourceType::Food => "%",
        ResourceType::Wood => "t",
        ResourceType::Stone => "*",
        ResourceType::Ore => "o",
        ResourceType::Metal => "m",
        ResourceType::Planks => "=",
        ResourceType::Blocks => "■",
    }
}

#[must_use]
pub const fn get_resource_color(resource: ResourceType) -> Color {
    match resource {
        ResourceType::Food => Color::Green,
        ResourceType::Wood => Color::Rgb(139, 69, 19), // SaddleBrown
        ResourceType::Stone => Color::Gray,
        ResourceType::Ore => Color::Rgb(165, 42, 42), // Brown
        ResourceType::Metal => Color::Cyan,
        ResourceType::Planks => Color::Yellow,
        ResourceType::Blocks => Color::White,
    }
}

const HEALTHY_THRESHOLD: f32 = 0.6;
const WARNING_THRESHOLD: f32 = 0.3;

/// Returns the character and color for rendering a pop.
///
/// Display is driven primarily by hunger (the only lethal need).
/// Leisure/rest affect mood but not survival, so they only downgrade
/// from happy to neutral — never to the "dying" indicator.
#[must_use]
pub fn get_pop_display(needs: &Needs) -> (&'static str, Color) {
    if needs.hunger > HEALTHY_THRESHOLD {
        // Well-fed: check other needs for mood
        if needs.worst() > WARNING_THRESHOLD {
            ("☺", Color::Yellow)
        } else {
            ("☻", Color::Rgb(255, 165, 0)) // fed but tired/bored
        }
    } else if needs.hunger > WARNING_THRESHOLD {
        ("☻", Color::Rgb(255, 165, 0)) // getting hungry
    } else {
        ("☹", Color::Red) // starving
    }
}
