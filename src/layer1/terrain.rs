use super::building::BuildingType;
use super::designation::DesignationType;
use super::pop::GridPosition;
use bevy_ecs::prelude::*;
use rand::Rng;
use ratatui::{prelude::*, widgets::Paragraph};
use std::collections::HashMap;
use std::hash::BuildHasher;

/// Represents the type of terrain in a cell.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TerrainType {
    /// Green grass, the default ground.
    Grass,
    /// Brown dirt, often found in patches.
    Dirt,
    /// Grey rock, harder material.
    Rock,
    /// Blue water, impassable by normal means.
    Water,
    /// A tree, source of wood.
    Tree,
}

impl TerrainType {
    /// Returns a string slice representation of the terrain.
    /// Used for rendering to avoid allocating a new String for every cell every frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::terrain::TerrainType;
    ///
    /// assert_eq!(TerrainType::Grass.as_str(), ".");
    /// assert_eq!(TerrainType::Rock.as_str(), "#");
    /// ```
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Grass => ".",
            Self::Dirt => ",",
            Self::Rock => "#",
            Self::Water => "~",
            Self::Tree => "↑",
        }
    }

    /// Returns the color associated with this terrain type.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::terrain::TerrainType;
    /// use ratatui::style::Color;
    ///
    /// assert_eq!(TerrainType::Grass.color(), Color::Green);
    /// ```
    #[must_use]
    pub const fn color(self) -> Color {
        match self {
            Self::Grass => Color::Green,
            Self::Dirt => Color::Rgb(139, 90, 43),
            Self::Rock => Color::DarkGray,
            Self::Water => Color::Blue,
            Self::Tree => Color::Rgb(0, 100, 0),
        }
    }

    /// Returns the human-readable name of the terrain type.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::terrain::TerrainType;
    ///
    /// assert_eq!(TerrainType::Grass.name(), "Grass");
    /// ```
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Grass => "Grass",
            Self::Dirt => "Dirt",
            Self::Rock => "Rock",
            Self::Water => "Water",
            Self::Tree => "Tree",
        }
    }
}

/// A 2D grid representing the game map's terrain layer.
#[derive(Resource)]
pub struct TerrainGrid {
    /// The width of the grid in cells.
    pub width: usize,
    /// The height of the grid in cells.
    pub height: usize,
    /// The flat vector of terrain tiles, stored in row-major order.
    pub tiles: Vec<TerrainType>, // row-major: index = y * width + x
}

impl TerrainGrid {
    /// Retrieves the terrain type at the specified coordinates, if within bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::terrain::{TerrainGrid, TerrainType, generate_terrain};
    ///
    /// let grid = generate_terrain(10, 10);
    /// if let Some(tile) = grid.get(0, 0) {
    ///     println!("Tile at (0,0) is {:?}", tile);
    /// }
    /// assert!(grid.get(100, 100).is_none());
    /// ```
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Option<TerrainType> {
        if x < self.width && y < self.height {
            Some(self.tiles[y * self.width + x])
        } else {
            None
        }
    }
}

/// Defines the visible area of the map for the player.
#[derive(Resource, Default, Clone, Copy)]
pub struct Viewport {
    /// The x-coordinate of the top-left corner of the viewport in grid space.
    pub x: i32,
    /// The y-coordinate of the top-left corner of the viewport in grid space.
    pub y: i32,
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

/// Generates a new terrain grid with procedural features.
#[must_use]
pub fn generate_terrain(width: usize, height: usize) -> TerrainGrid {
    let mut rng = rand::thread_rng();
    let mut tiles = vec![TerrainType::Grass; width * height];

    // Scatter some dirt patches
    for _ in 0..50 {
        let cx = rng.gen_range(0..width);
        let cy = rng.gen_range(0..height);
        let radius = rng.gen_range(2..6);
        fill_circle(&mut tiles, width, height, cx, cy, radius, TerrainType::Dirt);
    }

    // Scatter some rock
    for _ in 0..30 {
        let cx = rng.gen_range(0..width);
        let cy = rng.gen_range(0..height);
        let radius = rng.gen_range(1..4);
        fill_circle(&mut tiles, width, height, cx, cy, radius, TerrainType::Rock);
    }

    // Scatter some trees
    for _ in 0..40 {
        let cx = rng.gen_range(0..width);
        let cy = rng.gen_range(0..height);
        let radius = rng.gen_range(2..5);
        fill_circle(&mut tiles, width, height, cx, cy, radius, TerrainType::Tree);
    }

    // A river or lake
    for _ in 0..10 {
        let cx = rng.gen_range(0..width);
        let cy = rng.gen_range(0..height);
        let radius = rng.gen_range(3..8);
        fill_circle(
            &mut tiles,
            width,
            height,
            cx,
            cy,
            radius,
            TerrainType::Water,
        );
    }

    TerrainGrid {
        width,
        height,
        tiles,
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
fn fill_circle(
    tiles: &mut [TerrainType],
    width: usize,
    height: usize,
    cx: usize,
    cy: usize,
    radius: usize,
    terrain_type: TerrainType,
) {
    let r2 = (radius * radius) as i32;
    for dy in -(radius as i32)..=(radius as i32) {
        for dx in -(radius as i32)..=(radius as i32) {
            if dx * dx + dy * dy <= r2 {
                let x = cx as i32 + dx;
                let y = cy as i32 + dy;
                if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                    tiles[y as usize * width + x as usize] = terrain_type;
                }
            }
        }
    }
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
                        .map_or((" ", Color::Black), |tile| (tile.as_str(), tile.color()))
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
                let text = selected.as_str();
                line_spans.push(Span::styled(text, Style::default().fg(Color::White).bg(bg)));
                continue;
            }

            // Designation mode cursor (highest priority, shared with build mode)
            if let Some((_, selected, can_place)) = ctx
                .designation_mode
                .filter(|(cursor, _, _)| cursor.x == world_x && cursor.y == world_y)
            {
                let bg = if can_place { Color::Green } else { Color::Red };
                let text = selected.as_str();
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
                    designation_type.as_str(),
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
                    building_type.as_str(),
                    Style::default().fg(building_type.color()),
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
                    ctx.terrain
                        .get(ux, uy)
                        .map_or((" ", Color::Black), |tile| (tile.as_str(), tile.color()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_type_tree() {
        // Test new variant properties
        assert_eq!(TerrainType::Tree.as_str(), "↑");
        assert_eq!(TerrainType::Tree.color(), Color::Rgb(0, 100, 0));
        assert_eq!(TerrainType::Tree.name(), "Tree");
    }

    #[test]
    fn test_terrain_type_colors() {
        // Test color invariants for all terrain types
        assert_eq!(TerrainType::Grass.color(), Color::Green);
        assert_eq!(TerrainType::Dirt.color(), Color::Rgb(139, 90, 43));
        assert_eq!(TerrainType::Rock.color(), Color::DarkGray);
        assert_eq!(TerrainType::Water.color(), Color::Blue);
    }

    #[test]
    fn test_terrain_type_as_str() {
        // Test string representation invariants
        assert_eq!(TerrainType::Grass.as_str(), ".");
        assert_eq!(TerrainType::Dirt.as_str(), ",");
        assert_eq!(TerrainType::Rock.as_str(), "#");
        assert_eq!(TerrainType::Water.as_str(), "~");
    }

    #[test]
    fn test_terrain_generation() {
        let grid = generate_terrain(80, 50);
        assert_eq!(grid.width, 80);
        assert_eq!(grid.height, 50);
        assert_eq!(grid.tiles.len(), 80 * 50);

        // Check that the grid contains grass and only valid terrain types
        let has_grass = grid.tiles.contains(&TerrainType::Grass);
        assert!(has_grass, "Generated terrain should include grass");

        let all_valid = grid.tiles.iter().all(|t| {
            matches!(
                t,
                TerrainType::Grass
                    | TerrainType::Dirt
                    | TerrainType::Rock
                    | TerrainType::Water
                    | TerrainType::Tree
            )
        });
        assert!(all_valid, "All tiles must be valid terrain types");
    }

    #[test]
    fn test_terrain_grid_get_bounds() {
        let width = 10;
        let height = 5;
        let tiles = vec![TerrainType::Grass; width * height];
        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };

        // Valid access
        assert_eq!(grid.get(0, 0), Some(TerrainType::Grass));
        assert_eq!(grid.get(width - 1, height - 1), Some(TerrainType::Grass));

        // Out of bounds
        assert_eq!(grid.get(width, 0), None);
        assert_eq!(grid.get(0, height), None);
        assert_eq!(grid.get(width, height), None);
        assert_eq!(grid.get(100, 100), None);
    }

    #[test]
    fn test_fill_circle_clipping() {
        let width = 10;
        let height = 10;
        let mut tiles = vec![TerrainType::Grass; width * height];

        // Draw a circle of Dirt at (0,0) with radius 2.
        // Should cover (0,0), (0,1), (0,2), (1,0), (1,1), (2,0) etc.
        // and safely ignore negative coordinates.
        fill_circle(&mut tiles, width, height, 0, 0, 2, TerrainType::Dirt);

        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };

        // (0,0) should be Dirt
        assert_eq!(grid.get(0, 0), Some(TerrainType::Dirt));
        // (2,0) should be Dirt (distance 2 <= 2)
        assert_eq!(grid.get(2, 0), Some(TerrainType::Dirt));
        // (0,2) should be Dirt
        assert_eq!(grid.get(0, 2), Some(TerrainType::Dirt));
        // (3,0) should be Grass (distance 3 > 2)
        assert_eq!(grid.get(3, 0), Some(TerrainType::Grass));

        // Verify no panic or wrapping happened (check last element is still grass if far away)
        assert_eq!(grid.get(9, 9), Some(TerrainType::Grass));
    }

    #[test]
    fn test_viewport_rendering_offsets() {
        let width = 5;
        let height = 5;
        // Fill with Grass
        let tiles = vec![TerrainType::Grass; width * height];
        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };

        // Viewport shifted so (0,0) is at screen (1,1)
        // Viewport x=-1, y=-1.
        // Screen (0,0) -> World (-1, -1) -> Empty
        // Screen (1,1) -> World (0, 0) -> Grass
        let viewport = Viewport { x: -1, y: -1 };
        let area = Rect::new(0, 0, 3, 3);

        let spans = build_terrain_spans(area, &grid, &viewport);

        assert_eq!(spans.len(), 3);

        // Helper to check a span's content
        let check_cell = |y: usize, x: usize, expected_char: &str, expected_color: Color| {
            let span = &spans[y].spans[x];
            assert_eq!(span.content, expected_char);
            assert_eq!(span.style.fg, Some(expected_color));
        };

        // Row 0 (World y = -1): All should be empty
        for x in 0..3 {
            check_cell(0, x, " ", Color::Black);
        }

        // Row 1 (World y = 0):
        // x=0 (World x=-1) -> Empty
        check_cell(1, 0, " ", Color::Black);
        // x=1 (World x=0) -> Grass
        check_cell(1, 1, ".", Color::Green);
        // x=2 (World x=1) -> Grass
        check_cell(1, 2, ".", Color::Green);
    }

    #[test]
    fn test_build_map_layer_spans_designations() {
        let width = 3;
        let height = 3;
        let tiles = vec![TerrainType::Grass; width * height];
        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };
        let viewport = Viewport { x: 0, y: 0 };
        let area = Rect::new(0, 0, 3, 3);

        let pop_data = HashMap::new();
        let buildings_data = HashMap::new();
        let mut designations_data = HashMap::new();
        designations_data.insert(GridPosition { x: 1, y: 1 }, DesignationType::Mine);

        let ctx = MapRenderContext {
            area,
            terrain: &grid,
            viewport: &viewport,
            pops_data: &pop_data,
            buildings_data: &buildings_data,
            designations_data: &designations_data,
            build_mode: None,
            designation_mode: None,
        };

        let spans = build_map_layer_spans(ctx);
        // Middle char should be Mine
        assert_eq!(spans[1].spans[1].content, "⛏");
        assert_eq!(spans[1].spans[1].style.fg, Some(Color::Red));
    }

    #[test]
    fn test_render_map_layer_spans() {
        let width = 5;
        let height = 5;
        let mut tiles = vec![TerrainType::Grass; width * height];
        // Add some dirt at (1, 1). Index = y * width + x = 1 * 5 + 1 = 6.
        tiles[width + 1] = TerrainType::Dirt;
        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };

        let viewport = Viewport { x: 0, y: 0 };
        let area = Rect::new(0, 0, 3, 3);

        let mut pop_data = HashMap::new();
        pop_data.insert(GridPosition { x: 1, y: 1 }, ("P", Color::Yellow));
        pop_data.insert(GridPosition { x: 0, y: 0 }, ("P", Color::Yellow));

        let buildings_data = HashMap::new();
        let designations_data = HashMap::new();

        let ctx = MapRenderContext {
            area,
            terrain: &grid,
            viewport: &viewport,
            pops_data: &pop_data,
            buildings_data: &buildings_data,
            designations_data: &designations_data,
            build_mode: None,
            designation_mode: None,
        };

        let spans = build_map_layer_spans(ctx);

        assert_eq!(spans.len(), 3);

        let check_cell = |y: usize, x: usize, expected_char: &str, expected_color: Color| {
            let span = &spans[y].spans[x];
            assert_eq!(span.content, expected_char);
            assert_eq!(span.style.fg, Some(expected_color));
        };

        // (0,0) should be Pop
        check_cell(0, 0, "P", Color::Yellow);
        // (1,0) -> Grass
        check_cell(1, 0, ".", Color::Green);
        // (1,1) should be Pop (overriding Dirt)
        check_cell(1, 1, "P", Color::Yellow);
        // (0,1) -> Grass
        check_cell(0, 1, ".", Color::Green);
    }

    #[test]
    fn test_build_map_layer_spans_no_pops() {
        let width = 3;
        let height = 3;
        let tiles = vec![TerrainType::Water; width * height];
        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };

        let viewport = Viewport { x: 0, y: 0 };
        let area = Rect::new(0, 0, 3, 3);
        let pop_data = HashMap::new();
        let buildings_data = HashMap::new();
        let designations_data = HashMap::new();

        let ctx = MapRenderContext {
            area,
            terrain: &grid,
            viewport: &viewport,
            pops_data: &pop_data,
            buildings_data: &buildings_data,
            designations_data: &designations_data,
            build_mode: None,
            designation_mode: None,
        };

        let spans = build_map_layer_spans(ctx);

        assert_eq!(spans.len(), 3);
        // All should be water
        for row in &spans {
            for span in &row.spans {
                assert_eq!(span.content, "~");
                assert_eq!(span.style.fg, Some(Color::Blue));
            }
        }
    }

    #[test]
    fn test_build_map_layer_spans_negative_viewport() {
        let width = 5;
        let height = 5;
        let tiles = vec![TerrainType::Rock; width * height];
        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };

        let viewport = Viewport { x: -2, y: -2 };
        let area = Rect::new(0, 0, 4, 4);
        let mut pop_data = HashMap::new();
        pop_data.insert(GridPosition { x: 0, y: 0 }, ("P", Color::Yellow));
        let buildings_data = HashMap::new();
        let designations_data = HashMap::new();

        let ctx = MapRenderContext {
            area,
            terrain: &grid,
            viewport: &viewport,
            pops_data: &pop_data,
            buildings_data: &buildings_data,
            designations_data: &designations_data,
            build_mode: None,
            designation_mode: None,
        };

        let spans = build_map_layer_spans(ctx);

        assert_eq!(spans.len(), 4);

        // First two rows should be black (outside grid)
        for row in spans.iter().take(2) {
            for span in &row.spans {
                assert_eq!(span.content, " ");
                assert_eq!(span.style.fg, Some(Color::Black));
            }
        }

        // Row 2 should have pop at column 2
        assert_eq!(spans[2].spans[2].content, "P");
        assert_eq!(spans[2].spans[2].style.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_build_map_layer_spans_out_of_bounds() {
        let width = 2;
        let height = 2;
        let tiles = vec![TerrainType::Grass; width * height];
        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };

        // Viewport positioned so most of the view is out of bounds
        let viewport = Viewport { x: 10, y: 10 };
        let area = Rect::new(0, 0, 3, 3);
        let pop_data = HashMap::new();
        let buildings_data = HashMap::new();
        let designations_data = HashMap::new();

        let ctx = MapRenderContext {
            area,
            terrain: &grid,
            viewport: &viewport,
            pops_data: &pop_data,
            buildings_data: &buildings_data,
            designations_data: &designations_data,
            build_mode: None,
            designation_mode: None,
        };

        let spans = build_map_layer_spans(ctx);

        assert_eq!(spans.len(), 3);
        // All should be black/empty (out of bounds)
        for row in &spans {
            for span in &row.spans {
                assert_eq!(span.content, " ");
                assert_eq!(span.style.fg, Some(Color::Black));
            }
        }
    }

    #[test]
    fn test_render_map_layer() {
        let width = 5;
        let height = 5;
        let tiles = vec![TerrainType::Dirt; width * height];
        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };

        let viewport = Viewport { x: 0, y: 0 };
        let mut pop_data = HashMap::new();
        pop_data.insert(GridPosition { x: 1, y: 1 }, ("P", Color::Yellow));
        let buildings_data = HashMap::new();
        let designations_data = HashMap::new();

        let backend = ratatui::backend::TestBackend::new(10, 10);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();

        let result = terminal.draw(|frame| {
            let area = Rect::new(0, 0, 5, 5);
            let ctx = MapRenderContext {
                area,
                terrain: &grid,
                viewport: &viewport,
                pops_data: &pop_data,
                buildings_data: &buildings_data,
                designations_data: &designations_data,
                build_mode: None,
                designation_mode: None,
            };
            render_map_layer(frame, ctx);
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_render_terrain() {
        let width = 3;
        let height = 3;
        let tiles = vec![TerrainType::Rock; width * height];
        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };

        let viewport = Viewport { x: 0, y: 0 };

        let backend = ratatui::backend::TestBackend::new(5, 5);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();

        let result = terminal.draw(|frame| {
            let area = Rect::new(0, 0, 3, 3);
            render_terrain(frame, area, &grid, &viewport);
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_build_map_layer_spans_all_terrain_types() {
        let width = 5;
        let height = 4;
        let mut tiles = vec![TerrainType::Grass; width * height];

        // Set different terrain types
        tiles[0] = TerrainType::Grass;
        tiles[1] = TerrainType::Dirt;
        tiles[2] = TerrainType::Rock;
        tiles[3] = TerrainType::Water;
        tiles[4] = TerrainType::Tree;

        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };

        let viewport = Viewport { x: 0, y: 0 };
        let area = Rect::new(0, 0, 5, 1);
        let pop_data = HashMap::new();
        let buildings_data = HashMap::new();
        let designations_data = HashMap::new();

        let ctx = MapRenderContext {
            area,
            terrain: &grid,
            viewport: &viewport,
            pops_data: &pop_data,
            buildings_data: &buildings_data,
            designations_data: &designations_data,
            build_mode: None,
            designation_mode: None,
        };

        let spans = build_map_layer_spans(ctx);

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].spans.len(), 5);

        // Check each terrain type is rendered correctly
        assert_eq!(spans[0].spans[0].content, "."); // Grass
        assert_eq!(spans[0].spans[1].content, ","); // Dirt
        assert_eq!(spans[0].spans[2].content, "#"); // Rock
        assert_eq!(spans[0].spans[3].content, "~"); // Water
        assert_eq!(spans[0].spans[4].content, "↑"); // Tree
    }

    #[test]
    fn test_build_map_layer_spans_pop_priority() {
        let width = 3;
        let height = 3;
        let tiles = vec![TerrainType::Grass; width * height];
        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };

        let viewport = Viewport { x: 0, y: 0 };
        let area = Rect::new(0, 0, 2, 2);

        let mut pop_data = HashMap::new();
        pop_data.insert(GridPosition { x: 0, y: 0 }, ("P", Color::Yellow));
        pop_data.insert(GridPosition { x: 1, y: 1 }, ("P", Color::Yellow));
        let buildings_data = HashMap::new();
        let designations_data = HashMap::new();

        let ctx = MapRenderContext {
            area,
            terrain: &grid,
            viewport: &viewport,
            pops_data: &pop_data,
            buildings_data: &buildings_data,
            designations_data: &designations_data,
            build_mode: None,
            designation_mode: None,
        };

        let spans = build_map_layer_spans(ctx);

        // Pops should override terrain
        assert_eq!(spans[0].spans[0].content, "P");
        assert_eq!(spans[0].spans[0].style.fg, Some(Color::Yellow));
        assert_eq!(spans[1].spans[1].content, "P");
        assert_eq!(spans[1].spans[1].style.fg, Some(Color::Yellow));

        // Non-pop tiles should show terrain
        assert_eq!(spans[0].spans[1].content, ".");
        assert_eq!(spans[1].spans[0].content, ".");
    }

    #[test]
    fn test_build_map_layer_spans_partial_out_of_bounds() {
        let width = 3;
        let height = 3;
        let tiles = vec![TerrainType::Dirt; width * height];
        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };

        // Viewport positioned so half the view is out of bounds
        let viewport = Viewport { x: 1, y: 1 };
        let area = Rect::new(0, 0, 4, 4);
        let pop_data = HashMap::new();
        let buildings_data = HashMap::new();
        let designations_data = HashMap::new();

        let ctx = MapRenderContext {
            area,
            terrain: &grid,
            viewport: &viewport,
            pops_data: &pop_data,
            buildings_data: &buildings_data,
            designations_data: &designations_data,
            build_mode: None,
            designation_mode: None,
        };

        let spans = build_map_layer_spans(ctx);

        assert_eq!(spans.len(), 4);

        // First 2x2 should be Dirt (within bounds)
        for row in spans.iter().take(2) {
            for span in row.spans.iter().take(2) {
                assert_eq!(span.content, ",");
            }
        }

        // Rest should be black (out of bounds)
        for span in spans[0].spans.iter().skip(3) {
            assert_eq!(span.content, " ");
            assert_eq!(span.style.fg, Some(Color::Black));
        }
    }

    #[test]
    fn test_render_map_layer_empty_area() {
        let width = 2;
        let height = 2;
        let tiles = vec![TerrainType::Grass; width * height];
        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };

        let viewport = Viewport { x: 0, y: 0 };
        let pop_data = HashMap::new();
        let buildings_data = HashMap::new();
        let designations_data = HashMap::new();

        let backend = ratatui::backend::TestBackend::new(5, 5);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();

        let result = terminal.draw(|frame| {
            let area = Rect::new(0, 0, 2, 2);
            let ctx = MapRenderContext {
                area,
                terrain: &grid,
                viewport: &viewport,
                pops_data: &pop_data,
                buildings_data: &buildings_data,
                designations_data: &designations_data,
                build_mode: None,
                designation_mode: None,
            };
            render_map_layer(frame, ctx);
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_build_map_layer_spans_single_pop() {
        let width = 2;
        let height = 2;
        let tiles = vec![TerrainType::Grass; width * height];
        let grid = TerrainGrid {
            width,
            height,
            tiles,
        };

        let viewport = Viewport { x: 0, y: 0 };
        let area = Rect::new(0, 0, 2, 2);

        let mut pop_data = HashMap::new();
        pop_data.insert(GridPosition { x: 0, y: 0 }, ("P", Color::Yellow));
        let buildings_data = HashMap::new();
        let designations_data = HashMap::new();

        let ctx = MapRenderContext {
            area,
            terrain: &grid,
            viewport: &viewport,
            pops_data: &pop_data,
            buildings_data: &buildings_data,
            designations_data: &designations_data,
            build_mode: None,
            designation_mode: None,
        };

        let spans = build_map_layer_spans(ctx);

        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].spans[0].content, "P");
    }

    #[test]
    fn test_fill_circle_all_terrain_types() {
        let width = 10;
        let height = 10;

        for terrain_type in [
            TerrainType::Grass,
            TerrainType::Dirt,
            TerrainType::Rock,
            TerrainType::Water,
        ] {
            let mut tiles = vec![TerrainType::Grass; width * height];
            fill_circle(&mut tiles, width, height, 5, 5, 2, terrain_type);

            // Center should be the terrain type
            assert_eq!(tiles[5 * width + 5], terrain_type);
        }
    }
}
