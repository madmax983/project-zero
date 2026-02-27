#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;
    use scale::layer1::water::WaterGrid;
    use scale::layer1::{TerrainGrid, TerrainType, Viewport};
    use scale::ui::map::{build_map_layer_spans, MapRenderContext};
    use std::collections::HashMap;

    #[test]
    fn test_render_overflow() {
        // Setup
        let width = 10;
        let height = 10;
        let tiles = vec![TerrainType::Grass; width * height];
        let terrain = TerrainGrid {
            width,
            height,
            tiles,
        };

        let water = WaterGrid::new(10, 10);
        let viewport = Viewport {
            x: i32::MAX - 5,
            y: 0,
        }; // Near max
        let entities = HashMap::new();

        let ctx = MapRenderContext {
            area: Rect {
                x: 0,
                y: 0,
                width: 20,
                height: 10,
            }, // Width 20.
            // viewport.x (i32::MAX - 5) + screen_x (6) = i32::MAX + 1 -> Overflow!
            terrain: &terrain,
            water: &water,
            viewport: &viewport,
            entities_data: &entities,
            build_mode: None,
            designation_mode: None,
            season: None,
            wall_time: 0.0,
        };

        // This should panic in debug mode due to overflow if not handled
        let _ = build_map_layer_spans(ctx);
    }
}
