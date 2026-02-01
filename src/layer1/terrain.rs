use bevy_ecs::prelude::*;
use rand::Rng;
use ratatui::{prelude::*, widgets::Paragraph};

/// Represents the different types of terrain available in the game.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TerrainType {
    /// Walkable grass terrain.
    Grass,
    /// Walkable dirt terrain.
    Dirt,
    /// Walkable rock terrain.
    Rock,
    /// Water terrain.
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

    /// Returns the color associated with the terrain type.
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

/// A grid representing the game map's terrain.
#[derive(Resource)]
pub struct TerrainGrid {
    /// The width of the grid in tiles.
    pub width: usize,
    /// The height of the grid in tiles.
    pub height: usize,
    /// The flat vector of tiles, stored in row-major order.
    pub tiles: Vec<TerrainType>, // row-major: index = y * width + x
}

impl TerrainGrid {
    /// Creates a new `TerrainGrid` with the specified dimensions, filled with the default terrain.
    #[must_use]
    pub fn new(width: usize, height: usize, default_terrain: TerrainType) -> Self {
        Self {
            width,
            height,
            tiles: vec![default_terrain; width * height],
        }
    }

    /// Retrieves the terrain type at the specified coordinates, if within bounds.
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Option<TerrainType> {
        if x < self.width && y < self.height {
            Some(self.tiles[y * self.width + x])
        } else {
            None
        }
    }

    /// Fills a circle with the specified terrain type.
    ///
    /// The circle is defined by its center `(cx, cy)` and radius `r`.
    /// Tiles within the radius are set to `t`, provided they are within grid bounds.
    pub fn fill_circle(&mut self, cx: usize, cy: usize, r: usize, t: TerrainType) {
        let r_sq = r.saturating_mul(r);
        let min_x = cx.saturating_sub(r);
        let max_x = (cx + r).min(self.width - 1);
        let min_y = cy.saturating_sub(r);
        let max_y = (cy + r).min(self.height - 1);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                // Calculate relative distance squared safely
                let dx = x.abs_diff(cx);
                let dy = y.abs_diff(cy);

                if dx * dx + dy * dy <= r_sq {
                    self.tiles[y * self.width + x] = t;
                }
            }
        }
    }
}

/// Represents the camera viewport into the world.
#[derive(Resource, Default)]
pub struct Viewport {
    /// Top-left corner x-coordinate in grid coords.
    pub x: i32,
    /// Top-left corner y-coordinate in grid coords.
    pub y: i32,
}

/// Generates a new terrain grid with procedural features.
#[must_use]
pub fn generate_terrain(width: usize, height: usize) -> TerrainGrid {
    let mut rng = rand::thread_rng();
    let mut grid = TerrainGrid::new(width, height, TerrainType::Grass);

    scatter_terrain(&mut rng, &mut grid, 50, 2..6, TerrainType::Dirt);
    scatter_terrain(&mut rng, &mut grid, 30, 1..4, TerrainType::Rock);
    scatter_terrain(&mut rng, &mut grid, 10, 3..8, TerrainType::Water);

    grid
}

fn scatter_terrain(
    rng: &mut impl Rng,
    grid: &mut TerrainGrid,
    count: usize,
    radius_range: std::ops::Range<usize>,
    terrain: TerrainType,
) {
    for _ in 0..count {
        let cx = rng.gen_range(0..grid.width);
        let cy = rng.gen_range(0..grid.height);
        let radius = rng.gen_range(radius_range.clone());
        grid.fill_circle(cx, cy, radius, terrain);
    }
}

/// Builds the lines of text to render the terrain within a given area.
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
                if let (Ok(wx), Ok(wy)) = (usize::try_from(world_x), usize::try_from(world_y)) {
                    terrain
                        .get(wx, wy)
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

/// Renders the terrain to the frame.
pub fn render_terrain(frame: &mut Frame, area: Rect, terrain: &TerrainGrid, viewport: &Viewport) {
    let spans = build_terrain_spans(area, terrain, viewport);
    let paragraph = Paragraph::new(spans);
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let grid = TerrainGrid::new(width, height, TerrainType::Grass);

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
        let mut grid = TerrainGrid::new(width, height, TerrainType::Grass);

        // Draw a circle of Dirt at (0,0) with radius 2.
        // Should cover (0,0), (0,1), (0,2), (1,0), (1,1), (2,0) etc.
        // and safely ignore negative coordinates.
        grid.fill_circle(0, 0, 2, TerrainType::Dirt);

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
        let grid = TerrainGrid::new(width, height, TerrainType::Grass);

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
}
