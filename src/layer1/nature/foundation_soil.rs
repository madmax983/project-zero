use crate::layer1::map::GridPosition;
use crate::layer1::nature::fertility::FertilityGrid;
use crate::layer1::pop::PopDied;
use bevy::utils::HashSet;
use bevy_ecs::prelude::*;

#[derive(Resource, Default, Debug, Clone)]
pub struct FoundationSoilGrid {
    pub landing_tiles: HashSet<(i32, i32)>,
    pub buffed_tiles: HashSet<(i32, i32)>,
}

pub fn apply_foundation_soil_system(
    mut events: EventReader<PopDied>,
    mut foundation_grid: ResMut<FoundationSoilGrid>,
    mut fertility_grid: Option<ResMut<FertilityGrid>>,
    positions: Query<&GridPosition>,
) {
    for event in events.read() {
        if event.reason == "Old Age" {
            if let Ok(pos) = positions.get(event.entity) {
                let coords = (pos.x, pos.y);
                if foundation_grid.landing_tiles.contains(&coords) {
                    foundation_grid.buffed_tiles.insert(coords);
                    if let Some(ref mut fertility) = fertility_grid {
                        if pos.x >= 0 && pos.y >= 0 {
                            fertility.modify(pos.x as usize, pos.y as usize, 0.2); // Boost fertility
                        }
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
        let mut app = bevy_app::App::new();

        let mut f_grid = FoundationSoilGrid::default();
        f_grid.landing_tiles.insert((0, 0));
        app.world_mut().insert_resource(f_grid);

        let mut fertility = FertilityGrid::new(10, 10);
        fertility.set(0, 0, 0.5);
        app.world_mut().insert_resource(fertility);

        app.add_systems(bevy_app::Update, apply_foundation_soil_system);
        app.add_event::<PopDied>();

        let entity = app.world_mut().spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
        )).id();

        app.world_mut().send_event(PopDied {
            entity,
            name: "Founder".to_string(),
            tick: 1,
            reason: "Old Age".to_string(),
        });

        app.update();

        let f_grid = app.world().resource::<FoundationSoilGrid>();
        assert!(f_grid.buffed_tiles.contains(&(0, 0)));

        let fertility = app.world().resource::<FertilityGrid>();
        assert!(fertility.get(0, 0) > 0.5);
    }

    #[test]
    fn test_no_buff_for_unnatural_death() {
        let mut app = bevy_app::App::new();

        let mut f_grid = FoundationSoilGrid::default();
        f_grid.landing_tiles.insert((0, 0));
        app.world_mut().insert_resource(f_grid);

        app.add_systems(bevy_app::Update, apply_foundation_soil_system);
        app.add_event::<PopDied>();

        let entity = app.world_mut().spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
        )).id();

        app.world_mut().send_event(PopDied {
            entity,
            name: "Unlucky".to_string(),
            tick: 1,
            reason: "Starvation".to_string(),
        });

        app.update();

        let f_grid = app.world().resource::<FoundationSoilGrid>();
        assert!(!f_grid.buffed_tiles.contains(&(0, 0)));
    }
}
