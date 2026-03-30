use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::social::Relationships;

#[derive(Component)]
pub struct SpiteFence {
    pub cost_multiplier: f32,
}

pub fn spawn_spite_fence_system(
    mut commands: Commands,
    query: Query<(Entity, &Relationships)>,
    pop_query: Query<&GridPosition, With<crate::layer1::pop::Pop>>,
    existing_fences: Query<&GridPosition, With<SpiteFence>>,
) {
    for (entity, rel) in query.iter() {
        for (&target_entity, &affinity) in &rel.affinities {
            if affinity <= -100.0 {
                if let (Ok(pos1), Ok(pos2)) = (pop_query.get(entity), pop_query.get(target_entity)) {
                    let mid_x = (pos1.x + pos2.x) / 2;
                    let mid_y = (pos1.y + pos2.y) / 2;

                    // Check if fence already exists here
                    let mut exists = false;
                    for fence_pos in existing_fences.iter() {
                        if fence_pos.x == mid_x && fence_pos.y == mid_y {
                            exists = true;
                            break;
                        }
                    }

                    if !exists {
                        commands.spawn((
                            GridPosition { x: mid_x, y: mid_y },
                            SpiteFence { cost_multiplier: 5.0 }
                        ));
                    }
                }
            }
        }
    }
}

#[derive(Resource)]
pub struct SpiteGrid {
    pub width: i32,
    pub height: i32,
    pub tiles: std::collections::HashMap<(i32, i32), f32>,
}

impl Default for SpiteGrid {
    fn default() -> Self {
        Self { width: 100, height: 100, tiles: std::collections::HashMap::new() }
    }
}

impl SpiteGrid {
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height, tiles: std::collections::HashMap::new() }
    }
    pub fn get_cost_multiplier(&self, x: i32, y: i32) -> f32 {
        *self.tiles.get(&(x, y)).unwrap_or(&1.0)
    }
}

pub fn calculate_path_cost_system(
    mut spite_grid_res: ResMut<SpiteGrid>,
    fences: Query<(&GridPosition, &SpiteFence)>,
) {
    spite_grid_res.tiles.clear();
    for (pos, fence) in fences.iter() {
        spite_grid_res.tiles.insert((pos.x, pos.y), fence.cost_multiplier);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_rivals_spawn_spite_fence() {
        let mut app = bevy_app::App::new();
        app.add_systems(bevy_app::Update, spawn_spite_fence_system);

        let pop1 = app.world_mut().spawn((GridPosition { x: 10, y: 10 }, Pop, Relationships::default())).id();
        let pop2 = app.world_mut().spawn((GridPosition { x: 12, y: 10 }, Pop)).id();

        app.world_mut().get_mut::<Relationships>(pop1).unwrap().affinities.insert(pop2, -100.0);

        app.update();

        let mut query = app.world_mut().query::<(&GridPosition, &SpiteFence)>();
        let spite_fences = query.iter(app.world()).count();
        assert_eq!(spite_fences, 1);

        let fence_pos = query.iter(app.world()).next().unwrap().0;
        assert_eq!(fence_pos.x, 11);
        assert_eq!(fence_pos.y, 10);
    }

    #[test]
    fn test_spite_fence_increases_pathfinding_cost() {
        let mut app = bevy_app::App::new();
        app.init_resource::<SpiteGrid>();
        app.add_systems(bevy_app::Update, calculate_path_cost_system);

        let fence_pos = GridPosition { x: 5, y: 5 };
        app.world_mut().spawn((fence_pos, SpiteFence { cost_multiplier: 5.0 }));

        // Run once
        app.update();

        let grid = app.world().resource::<SpiteGrid>();
        assert_eq!(grid.get_cost_multiplier(5, 5), 5.0);
    }
}
