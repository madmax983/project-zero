use crate::layer1::events::BuildingRemovedEvent;
use bevy_ecs::prelude::*;
use bevy_utils::HashMap;

/// The starting penalty added to the pathfinding cost when a building is destroyed.
const INITIAL_GHOST_PENALTY: u32 = 20;

/// The GhostGrid tracks locations of destroyed buildings. The network "remembers" its past shape,
/// trying to route resources as if buildings were there, causing inefficiencies.
#[derive(Resource, Default, Debug, Clone)]
pub struct GhostGrid {
    /// Maps grid coordinates (x, y) to the remaining pathfinding penalty.
    pub grid: HashMap<(i32, i32), u32>,
}

impl GhostGrid {
    /// Gets the current pathfinding penalty for the given coordinate.
    pub fn get_penalty(&self, x: i32, y: i32) -> i32 {
        self.grid.get(&(x, y)).copied().unwrap_or(0) as i32
    }
}

/// Listens for `BuildingRemovedEvent` and populates the `GhostGrid` with a new ghost signature.
pub fn ghost_grid_system(
    mut events: EventReader<BuildingRemovedEvent>,
    mut ghost_grid: ResMut<GhostGrid>,
) {
    for event in events.read() {
        let (x, y) = (event.position.x, event.position.y);
        ghost_grid.grid.insert((x, y), INITIAL_GHOST_PENALTY);
    }
}

/// Slowly decays the "memory" of destroyed buildings over time.
pub fn ghost_grid_decay_system(mut ghost_grid: ResMut<GhostGrid>) {
    // Collect keys to remove to avoid mutable borrow issues
    let mut to_remove = Vec::new();

    for (pos, penalty) in ghost_grid.grid.iter_mut() {
        *penalty = penalty.saturating_sub(1);
        if *penalty == 0 {
            to_remove.push(*pos);
        }
    }

    for pos in to_remove {
        ghost_grid.grid.remove(&pos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::BuildingType;
    use crate::layer1::core::map::GridPosition;

    #[test]
    fn test_ghost_grid_initialization() {
        let grid = GhostGrid::default();
        assert_eq!(grid.get_penalty(5, 5), 0);
    }

    #[test]
    fn test_ghost_grid_system_adds_penalty() {
        let mut app = bevy_app::App::new();
        app.insert_resource(GhostGrid::default());
        app.add_event::<BuildingRemovedEvent>();
        app.add_systems(bevy_app::Update, ghost_grid_system);

        app.world_mut().send_event(BuildingRemovedEvent {
            entity: Entity::from_raw(1),
            position: GridPosition { x: 10, y: 15 },
            building_type: BuildingType::Housing,
        });

        app.update();

        let grid = app.world().resource::<GhostGrid>();
        assert_eq!(grid.get_penalty(10, 15), INITIAL_GHOST_PENALTY as i32);
    }

    #[test]
    fn test_ghost_grid_decay_system() {
        let mut app = bevy_app::App::new();
        let mut initial_grid = GhostGrid::default();
        initial_grid.grid.insert((5, 5), 2);
        initial_grid.grid.insert((6, 6), 1);
        app.insert_resource(initial_grid);
        app.add_systems(bevy_app::Update, ghost_grid_decay_system);

        app.update();

        let grid = app.world().resource::<GhostGrid>();
        assert_eq!(grid.get_penalty(5, 5), 1);
        assert_eq!(grid.get_penalty(6, 6), 0);
        assert!(!grid.grid.contains_key(&(6, 6))); // Should be removed

        app.update();

        let grid2 = app.world().resource::<GhostGrid>();
        assert_eq!(grid2.get_penalty(5, 5), 0);
        assert!(!grid2.grid.contains_key(&(5, 5))); // Should be removed
    }
}
