use bevy_ecs::prelude::*;
use ratatui::{
    prelude::*,
    widgets::Paragraph,
};
use rand::Rng;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TerrainType {
    Grass,
    Dirt,
    Rock,
    Water,
}

impl TerrainType {
    pub fn char(&self) -> char {
        match self {
            TerrainType::Grass => '.',
            TerrainType::Dirt => ',',
            TerrainType::Rock => '#',
            TerrainType::Water => '~',
        }
    }

    pub fn color(&self) -> Color {
        match self {
            TerrainType::Grass => Color::Green,
            TerrainType::Dirt => Color::Rgb(139, 90, 43),
            TerrainType::Rock => Color::DarkGray,
            TerrainType::Water => Color::Blue,
        }
    }
}

#[derive(Resource)]
pub struct TerrainGrid {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<TerrainType>,  // row-major: index = y * width + x
}

impl TerrainGrid {
    pub fn get(&self, x: usize, y: usize) -> Option<TerrainType> {
        if x < self.width && y < self.height {
            Some(self.tiles[y * self.width + x])
        } else {
            None
        }
    }
}

#[derive(Resource)]
pub struct Viewport {
    pub x: i32,  // Top-left corner in grid coords
    pub y: i32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

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
        fill_circle(&mut tiles, width, height, cx, cy, radius, TerrainType::Water);
    }

    TerrainGrid { width, height, tiles }
}

fn fill_circle(tiles: &mut [TerrainType], w: usize, h: usize, cx: usize, cy: usize, r: usize, t: TerrainType) {
    let r2 = (r * r) as i32;
    for dy in -(r as i32)..=(r as i32) {
        for dx in -(r as i32)..=(r as i32) {
            if dx*dx + dy*dy <= r2 {
                let x = cx as i32 + dx;
                let y = cy as i32 + dy;
                if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
                    tiles[y as usize * w + x as usize] = t;
                }
            }
        }
    }
}

pub fn render_terrain(frame: &mut Frame, area: Rect, terrain: &TerrainGrid, viewport: &Viewport) {
    let mut spans: Vec<Line> = Vec::new();

    for screen_y in 0..area.height {
        let world_y = viewport.y + screen_y as i32;
        let mut line_spans = Vec::new();

        for screen_x in 0..area.width {
            let world_x = viewport.x + screen_x as i32;

            let (ch, color) = if world_x >= 0 && world_y >= 0 {
                if let Some(tile) = terrain.get(world_x as usize, world_y as usize) {
                    (tile.char(), tile.color())
                } else {
                    (' ', Color::Black)
                }
            } else {
                (' ', Color::Black)
            };

            line_spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
        }
        spans.push(Line::from(line_spans));
    }

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
            matches!(t, TerrainType::Grass | TerrainType::Dirt | TerrainType::Rock | TerrainType::Water)
        });
        assert!(all_valid, "All tiles must be valid terrain types");
    }
}
