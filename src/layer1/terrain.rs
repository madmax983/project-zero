use bevy_ecs::prelude::*;
use rand::Rng;
use ratatui::{prelude::*, widgets::Paragraph};

/// Represents the type of terrain for a single grid cell.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TerrainType {
    /// Green grass.
    Grass,
    /// Brown dirt.
    Dirt,
    /// Grey rock.
    Rock,
    /// Blue water.
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

/// A resource containing the 2D grid of terrain tiles.
#[derive(Resource)]
pub struct TerrainGrid {
    /// Width of the grid.
    pub width: usize,
    /// Height of the grid.
    pub height: usize,
    /// Flat vector of tiles, stored row-major.
    pub tiles: Vec<TerrainType>, // row-major: index = y * width + x
}

impl TerrainGrid {
    /// Gets the terrain type at the specified coordinates, if within bounds.
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Option<TerrainType> {
        if x < self.width && y < self.height {
            Some(self.tiles[y * self.width + x])
        } else {
            None
        }
    }
}

/// A resource defining the visible area of the map.
#[derive(Resource, Default)]
pub struct Viewport {
    /// The X coordinate of the top-left corner of the viewport in grid space.
    pub x: i32, // Top-left corner in grid coords
    /// The Y coordinate of the top-left corner of the viewport in grid space.
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

fn fill_circle(
    tiles: &mut [TerrainType],
    width: usize,
    height: usize,
    center_x: usize,
    center_y: usize,
    radius: usize,
    terrain: TerrainType,
) {
    // Attempt to convert to i64 for signed arithmetic.
    // If terrain dimensions exceed i64::MAX, we skip drawing to avoid issues.
    let Ok(radius_i64) = i64::try_from(radius) else {
        return;
    };
    let Ok(center_x_i64) = i64::try_from(center_x) else {
        return;
    };
    let Ok(center_y_i64) = i64::try_from(center_y) else {
        return;
    };
    let Ok(width_i64) = i64::try_from(width) else {
        return;
    };
    let Ok(height_i64) = i64::try_from(height) else {
        return;
    };

    let r2 = radius_i64 * radius_i64;

    for dy in -radius_i64..=radius_i64 {
        for dx in -radius_i64..=radius_i64 {
            if dx * dx + dy * dy <= r2 {
                let x = center_x_i64 + dx;
                let y = center_y_i64 + dy;

                if x >= 0 && x < width_i64 && y >= 0 && y < height_i64 {
                    // Safe to cast to usize because we just checked x, y >= 0
                    if let (Ok(xu), Ok(yu)) = (usize::try_from(x), usize::try_from(y)) {
                        // Calculate index and check bounds
                        if let Some(idx) = yu.checked_mul(width).and_then(|row| row.checked_add(xu)) {
                            // Verify index is within bounds (redundant if width/height correct, but safe)
                            if idx < tiles.len() {
                                tiles[idx] = terrain;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Builds a vector of `Line`s representing the terrain visible in the viewport.
#[must_use]
pub fn build_terrain_spans(
    area: Rect,
    terrain: &TerrainGrid,
    viewport: &Viewport,
) -> Vec<Line<'static>> {
    let mut spans: Vec<Line> = Vec::with_capacity(usize::from(area.height));

    for screen_y in 0..area.height {
        let world_y = viewport.y.wrapping_add(i32::from(screen_y));
        let mut line_spans = Vec::with_capacity(usize::from(area.width));

        for screen_x in 0..area.width {
            let world_x = viewport.x.wrapping_add(i32::from(screen_x));

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

/// Renders the terrain grid to the specified frame area.
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
}
