use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::collections::VecDeque;

/// A rival colony competing for resources and territory.
#[derive(Component)]
pub struct RivalColony {
    /// The unique identifier of the faction.
    pub faction_id: String,
    /// Points accumulated towards expanding territory.
    pub expansion_points: u32,
}

/// Tracks the resources stolen/gathered by the rival.
#[derive(Component, Default)]
pub struct RivalStockpile {
    /// Stored minerals.
    pub minerals: u32,
}

/// Represents resource types on the grid.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResourceType {
    /// Mineral resources.
    Minerals,
}

/// A 2D grid representing territory ownership and resources.
#[derive(Resource, Default)]
pub struct TerritoryGrid {
    /// Tracks ownership of tiles by Entity ID.
    pub ownership: HashMap<(i32, i32), Entity>,
    /// Tracks resources on tiles.
    pub resources: HashMap<(i32, i32), HashMap<ResourceType, u32>>,
}

impl TerritoryGrid {
    /// Gets the owner of a specific tile.
    pub fn get_owner(&self, x: i32, y: i32) -> Option<Entity> {
        self.ownership.get(&(x, y)).copied()
    }

    /// Sets the owner of a specific tile.
    pub fn set_owner(&mut self, x: i32, y: i32, owner: Entity) {
        self.ownership.insert((x, y), owner);
    }

    /// Gets the amount of a specific resource on a tile.
    pub fn get_resource(&self, x: i32, y: i32, resource_type: ResourceType) -> u32 {
        self.resources
            .get(&(x, y))
            .and_then(|res| res.get(&resource_type))
            .copied()
            .unwrap_or(0)
    }

    /// Sets the amount of a specific resource on a tile.
    pub fn set_resource(&mut self, x: i32, y: i32, resource_type: ResourceType, amount: u32) {
        self.resources
            .entry((x, y))
            .or_default()
            .insert(resource_type, amount);
    }
}

/// System for rival colonies to expand their territory.
pub fn rival_colony_expansion_system(
    mut query: Query<(Entity, &mut RivalColony, &GridPosition)>,
    mut territory: ResMut<TerritoryGrid>,
) {
    for (entity, mut colony, pos) in query.iter_mut() {
        // Claim starting tile if not claimed
        if territory.get_owner(pos.x, pos.y).is_none() {
            territory.set_owner(pos.x, pos.y, entity);
        }

        while colony.expansion_points >= 10 {
            let mut expanded = false;

            // Find an adjacent unclaimed tile by expanding out from the border
            let mut visited = HashMap::new();
            let mut queue = VecDeque::new();
            queue.push_back((pos.x, pos.y));
            visited.insert((pos.x, pos.y), true);

            while let Some((cx, cy)) = queue.pop_front() {
                if territory.get_owner(cx, cy) == Some(entity) {
                    // Check adjacents
                    let adjacents = [(cx + 1, cy), (cx - 1, cy), (cx, cy + 1), (cx, cy - 1)];
                    for (nx, ny) in adjacents {
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            visited.entry((nx, ny))
                        {
                            e.insert(true);
                            if territory.get_owner(nx, ny).is_none() {
                                territory.set_owner(nx, ny, entity);
                                colony.expansion_points -= 10;
                                expanded = true;
                                break;
                            } else if territory.get_owner(nx, ny) == Some(entity) {
                                queue.push_back((nx, ny));
                            }
                        }
                    }
                    if expanded {
                        break;
                    }
                }
            }

            // If we couldn't expand, break the loop to avoid infinite loops if trapped
            if !expanded {
                break;
            }
        }
    }
}

/// System for rival colonies to drain resources from their claimed territory.
pub fn rival_resource_drain_system(
    mut query: Query<(Entity, &mut RivalStockpile)>,
    mut territory: ResMut<TerritoryGrid>,
) {
    // This is simple for now, but inefficient for large territories
    // We iterate over all tiles, check owner, and drain

    // First collect tiles to drain
    let mut to_drain: HashMap<Entity, Vec<(i32, i32)>> = HashMap::new();

    for (&(x, y), &owner) in territory.ownership.iter() {
        to_drain.entry(owner).or_default().push((x, y));
    }

    for (entity, mut stockpile) in query.iter_mut() {
        if let Some(tiles) = to_drain.get(&entity) {
            for &(x, y) in tiles {
                let amount = territory.get_resource(x, y, ResourceType::Minerals);
                if amount > 0 {
                    let drain = amount.min(10); // Drain up to 10 at a time
                    territory.set_resource(x, y, ResourceType::Minerals, amount - drain);
                    stockpile.minerals += drain;
                }
            }
        }
    }
}

/// Register systems and resources
pub struct RivalColonyPlugin;
impl bevy_app::Plugin for RivalColonyPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.init_resource::<TerritoryGrid>().add_systems(
            bevy_app::Update,
            (rival_colony_expansion_system, rival_resource_drain_system),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(RivalColonyPlugin);
        app
    }

    #[test]
    fn test_rival_colony_claims_territory() {
        let mut app = setup_app();

        // Spawn a rival colony center
        let rival_entity = app
            .world_mut()
            .spawn((
                RivalColony {
                    faction_id: "OmniCorp".to_string(),
                    expansion_points: 100,
                },
                GridPosition { x: 50, y: 50 },
            ))
            .id();

        app.update(); // Trigger expansion

        let territory = app.world().resource::<TerritoryGrid>();
        assert_eq!(
            territory.get_owner(50, 50),
            Some(rival_entity),
            "Rival should claim its starting tile"
        );
        assert_eq!(
            territory.get_owner(51, 50),
            Some(rival_entity),
            "Rival should expand its territory using expansion points"
        );
        // Due to BFS, it should claim tiles radiating from the center.
        let mut claimed = 0;
        for _x in 45..=55 {
            for _y in 45..=55 {
                if territory.get_owner(_x, _y) == Some(rival_entity) {
                    claimed += 1;
                }
            }
        }
        // Starting tile (0 cost) + 10 expansions = 11 total tiles
        assert_eq!(
            claimed, 11,
            "Rival should claim exactly 11 tiles (1 starting + 10 expansions)"
        );
    }

    #[test]
    fn test_rival_colony_drains_resources_from_claimed_tiles() {
        let mut app = setup_app();

        // Setup a resource on the grid
        app.world_mut()
            .resource_mut::<TerritoryGrid>()
            .set_resource(51, 50, ResourceType::Minerals, 100);

        let rival_entity = app
            .world_mut()
            .spawn((
                RivalColony {
                    faction_id: "OmniCorp".to_string(),
                    expansion_points: 10,
                },
                RivalStockpile { minerals: 0 },
                GridPosition { x: 50, y: 50 },
            ))
            .id();

        app.update(); // Expands and claims tile (51, 50)
        app.update(); // Drains resources from claimed tile

        let territory = app.world().resource::<TerritoryGrid>();
        assert_eq!(
            territory.get_resource(51, 50, ResourceType::Minerals),
            90,
            "Rival should drain resources from the grid"
        );

        let stockpile = app.world().get::<RivalStockpile>(rival_entity).unwrap();
        assert_eq!(
            stockpile.minerals, 10,
            "Rival should accumulate drained resources in its stockpile"
        );
    }
}
