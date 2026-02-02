use bevy_ecs::prelude::*;
use rand::Rng;
use ratatui::{prelude::*, widgets::Paragraph};
use std::collections::HashSet;
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
}

impl TerrainType {
    /// Returns a string slice representation of the terrain.
    /// Used for rendering to avoid allocating a new String for every cell every frame.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Grass => ".",
            Self::Dirt => ",",
            Self::Rock => "#",
            Self::Water => "~",
        }
    }

    /// Returns the color associated with this terrain type.
    #[must_use]
    pub const fn color(self) -> Color {
        match self {
            Self::Grass => Color::Green,
            Self::Dirt => Color::Rgb(139, 90, 43),
            Self::Rock => Color::DarkGray,
            Self::Water => Color::Blue,
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
#[derive(Resource, Default)]
pub struct Viewport {
    /// The x-coordinate of the top-left corner of the viewport in grid space.
    pub x: i32,
    /// The y-coordinate of the top-left corner of the viewport in grid space.
    pub y: i32,
}

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

/// Builds a vector of text lines to render the terrain and pops within the given area.
pub fn build_terrain_and_pop_spans<S: BuildHasher>(
    area: Rect,
    terrain: &TerrainGrid,
    viewport: &Viewport,
    pop_positions: &HashSet<(i32, i32), S>,
) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = Vec::new();

    for screen_y in 0..area.height {
        let world_y = viewport.y + i32::from(screen_y);
        let mut line_spans = Vec::new();

        for screen_x in 0..area.width {
            let world_x = viewport.x + i32::from(screen_x);

            // Check for pop first
            if pop_positions.contains(&(world_x, world_y)) {
                let (ch, color) = super::pop::pop_display();
                line_spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
                continue;
            }

            // Otherwise render terrain
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
        lines.push(Line::from(line_spans));
    }
    lines
}

/// Render terrain grid and pops to the given frame area with viewport offset.
pub fn render_terrain_and_pops<S: BuildHasher>(
    frame: &mut Frame,
    area: Rect,
    terrain: &TerrainGrid,
    viewport: &Viewport,
    pop_positions: &HashSet<(i32, i32), S>,
) {
    let lines = build_terrain_and_pop_spans(area, terrain, viewport, pop_positions);
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
                TerrainType::Grass | TerrainType::Dirt | TerrainType::Rock | TerrainType::Water
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
    fn test_render_terrain_and_pops_spans() {
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

        let mut pop_positions = HashSet::new();
        pop_positions.insert((1, 1)); // Pop on top of Dirt
        pop_positions.insert((0, 0)); // Pop on top of Grass

        let spans = build_terrain_and_pop_spans(area, &grid, &viewport, &pop_positions);

        assert_eq!(spans.len(), 3);

        let check_cell = |y: usize, x: usize, expected_char: &str, expected_color: Color| {
            let span = &spans[y].spans[x];
            assert_eq!(span.content, expected_char);
            assert_eq!(span.style.fg, Some(expected_color));
        };

        let (pop_char, pop_color) = super::super::pop::pop_display();
        let pop_str = pop_char.to_string();

        // (0,0) should be Pop
        check_cell(0, 0, &pop_str, pop_color);
        // (1,0) -> Grass
        check_cell(1, 0, ".", Color::Green);
        // (1,1) should be Pop (overriding Dirt)
        check_cell(1, 1, &pop_str, pop_color);
        // (0,1) -> Grass
        check_cell(0, 1, ".", Color::Green);
    }
}
