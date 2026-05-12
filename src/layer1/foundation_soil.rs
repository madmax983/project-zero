use crate::layer1::map::GridPosition;
use crate::layer1::nature::fertility::FertilityGrid;
use crate::layer1::pop::PopDied;
use bevy_ecs::prelude::*;
use std::collections::HashSet;

/// Event triggered when foundation soil is applied.
#[derive(Event, Debug, Clone)]
pub struct FoundationSoilAppliedEvent;

/// Tracks original landing tiles where Foundation Soil effects can apply.
#[derive(Resource, Default)]
pub struct LandingTiles {
    pub coords: HashSet<(i32, i32)>,
}

/// A grid tracking structural integrity from Foundation Soil buffs.
#[derive(Resource, Default)]
pub struct FoundationSoilGrid {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
}

impl FoundationSoilGrid {
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        let size = width
            .checked_mul(height)
            .expect("Grid size overflow or too large");
        assert!(size <= 10_000_000, "Grid size overflow or too large");
        Self {
            width,
            height,
            values: vec![0.0; size],
        }
    }

    pub fn add(&mut self, x: i32, y: i32, amount: f32) {
        if x < 0 || y < 0 {
            return;
        }
        let x = x as usize;
        let y = y as usize;
        if x < self.width && y < self.height {
            let idx = y.checked_mul(self.width).unwrap().checked_add(x).unwrap();
            if idx < self.values.len() {
                self.values[idx] += amount;
            }
        }
    }
}

pub fn apply_foundation_soil_system(
    mut events: EventReader<PopDied>,
    landing_tiles: Option<Res<LandingTiles>>,
    mut fertility_grid: Option<ResMut<FertilityGrid>>,
    mut foundation_soil_grid: Option<ResMut<FoundationSoilGrid>>,
    query: Query<&GridPosition>,
    mut applied_events: EventWriter<FoundationSoilAppliedEvent>,
) {
    let landing = match landing_tiles {
        Some(l) => l,
        None => return,
    };

    for event in events.read() {
        if event.reason == "Old Age" {
            if let Ok(pos) = query.get(event.entity) {
                if landing.coords.contains(&(pos.x, pos.y)) {
                    if let Some(fg) = fertility_grid.as_mut() {
                        let x = pos.x as usize;
                        let y = pos.y as usize;
                        if x < fg.width && y < fg.height {
                            let idx = y.checked_mul(fg.width).unwrap().checked_add(x).unwrap();
                            if idx < fg.values.len() {
                                fg.values[idx] = (fg.values[idx] + 0.5).min(1.0);
                            }
                        }
                    }
                    if let Some(fsg) = foundation_soil_grid.as_mut() {
                        fsg.add(pos.x, pos.y, 100.0);
                        applied_events.send(FoundationSoilAppliedEvent);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_foundation_soil_buff_on_natural_death() {
        // Arrange: Setup world with a TerrainGrid and a Pop on a landing tile
        let mut app = bevy_app::App::new();
        // Setup Grid
        let fg = FertilityGrid::new(10, 10);
        let fsg = FoundationSoilGrid::new(10, 10);

        let mut landing_tiles = LandingTiles::default();
        landing_tiles.coords.insert((0, 0));

        app.world_mut().insert_resource(fg);
        app.world_mut().insert_resource(fsg);
        app.world_mut().insert_resource(landing_tiles);
        app.world_mut().init_resource::<Events<PopDied>>();
        app.world_mut()
            .init_resource::<Events<FoundationSoilAppliedEvent>>();

        let entity = app
            .world_mut()
            .spawn((Pop, GridPosition { x: 0, y: 0 }))
            .id();

        // Act: Trigger natural death (simulate event)
        let mut events = app.world_mut().resource_mut::<Events<PopDied>>();
        events.send(PopDied {
            entity,
            name: "Test".to_string(),
            tick: 0,
            reason: "Old Age".to_string(),
        });

        app.add_systems(bevy_app::Update, apply_foundation_soil_system);
        app.update();

        // Assert: The tile at (0,0) should have increased fertility/integrity
        let fg = app.world().resource::<FertilityGrid>();
        let fsg = app.world().resource::<FoundationSoilGrid>();

        let idx = 0;
        assert!(fg.values[idx] > 1.0 - f32::EPSILON); // Maxed out, or increased
        assert!(fsg.values[idx] > 0.0);
    }

    #[test]
    fn test_no_buff_for_unnatural_death() {
        // Arrange: Pop dies of starvation/violence
        let mut app = bevy_app::App::new();

        let fg = FertilityGrid::new(10, 10);
        let fsg = FoundationSoilGrid::new(10, 10);

        let mut landing_tiles = LandingTiles::default();
        landing_tiles.coords.insert((0, 0));

        app.world_mut().insert_resource(fg);
        app.world_mut().insert_resource(fsg);
        app.world_mut().insert_resource(landing_tiles);
        app.world_mut().init_resource::<Events<PopDied>>();
        app.world_mut()
            .init_resource::<Events<FoundationSoilAppliedEvent>>();

        let entity = app
            .world_mut()
            .spawn((Pop, GridPosition { x: 0, y: 0 }))
            .id();

        // Act: Trigger unnatural death
        let mut events = app.world_mut().resource_mut::<Events<PopDied>>();
        events.send(PopDied {
            entity,
            name: "Test".to_string(),
            tick: 0,
            reason: "Violence".to_string(),
        });

        app.add_systems(bevy_app::Update, apply_foundation_soil_system);
        app.update();

        // Assert: Tile at death location does not receive buff
        let fsg = app.world().resource::<FoundationSoilGrid>();
        assert_eq!(fsg.values[0], 0.0);
    }
}
