# 424: Subterranean Megafauna Migration

## 1. Overview
The "Subterranean Megafauna Migration" feature introduces massive, blind creatures that tunnel through the deep Z-levels of the map. They ignore player buildings but cause localized "Tremors" (building damage) and leave behind temporary "Tunnels" (empty space). These tunnels can be exploited by the player as instant, free subway systems for haulers until they naturally collapse.

This feature adds a dynamic element to deep-level expansion, forcing players to balance the risk of destruction with the reward of free, fast transit routes.

## 2. Dependencies
- `src/layer1/terrain.rs`: Terrain grid and Z-levels.
- `src/layer1/health.rs`: Building damage system.
- `src/layer1/pathfinding.rs`: Pathfinding logic for haulers using tunnels.
- `src/layer1/chronicle.rs`: For generating events about migration tremors.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::health::Health;
    use crate::layer1::building::Building;
    use crate::shared::math::GridPosition;

    fn setup_world() -> World {
        let mut world = World::new();
        let mut grid = TerrainGrid::new(10, 10, 5); // x, y, z
        // Set solid rock at z=2
        for x in 0..10 {
            for y in 0..10 {
                grid.set(x, y, 2, TerrainType::SolidRock);
            }
        }
        world.insert_resource(grid);
        world
    }

    #[test]
    fn test_megafauna_migration_creates_tunnels() {
        let mut world = setup_world();

        let migration_path = vec![
            GridPosition { x: 2, y: 2, z: 2 },
            GridPosition { x: 3, y: 2, z: 2 },
            GridPosition { x: 4, y: 2, z: 2 },
        ];

        // Trigger migration
        trigger_megafauna_migration(&mut world, migration_path);

        let grid = world.resource::<TerrainGrid>();
        // Verify path is now Empty Space / Tunnel
        assert_eq!(grid.get(2, 2, 2), Some(&TerrainType::Tunnel));
        assert_eq!(grid.get(3, 2, 2), Some(&TerrainType::Tunnel));
        assert_eq!(grid.get(4, 2, 2), Some(&TerrainType::Tunnel));
        // Verify adjacent tiles are still SolidRock
        assert_eq!(grid.get(1, 2, 2), Some(&TerrainType::SolidRock));
    }

    #[test]
    fn test_megafauna_migration_causes_tremors() {
        let mut world = setup_world();

        // Place a building above the migration path (z=1)
        let building = world.spawn((
            GridPosition { x: 3, y: 2, z: 1 },
            Building { name: "Mine Shaft".to_string(), ..Default::default() },
            Health { current: 100.0, max: 100.0 },
        )).id();

        let migration_path = vec![
            GridPosition { x: 2, y: 2, z: 2 },
            GridPosition { x: 3, y: 2, z: 2 },
            GridPosition { x: 4, y: 2, z: 2 },
        ];

        trigger_megafauna_migration(&mut world, migration_path);

        // Tremors should damage the building directly above
        let health = world.get::<Health>(building).unwrap();
        assert!(health.current < 100.0, "Building should take damage from tremors");
    }

    #[test]
    fn test_tunnel_collapse_over_time() {
        let mut world = setup_world();

        // Create a tunnel
        world.resource_mut::<TerrainGrid>().set(5, 5, 2, TerrainType::Tunnel);

        // Add a resource to track tunnel lifespan
        world.insert_resource(TunnelManager::new());
        world.resource_mut::<TunnelManager>().add_tunnel(GridPosition { x: 5, y: 5, z: 2 }, 10.0);

        // Simulate time passing
        update_tunnel_collapse_system(&mut world, 11.0); // Pass 11.0 seconds

        let grid = world.resource::<TerrainGrid>();
        assert_eq!(grid.get(5, 5, 2), Some(&TerrainType::SolidRock), "Tunnel should collapse back into rock");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::health::Health;
use crate::shared::math::GridPosition;

#[derive(Resource)]
pub struct TunnelManager {
    // position, time_remaining
    pub active_tunnels: Vec<(GridPosition, f32)>,
}

impl TunnelManager {
    pub fn new() -> Self {
        Self { active_tunnels: Vec::new() }
    }

    pub fn add_tunnel(&mut self, pos: GridPosition, lifespan: f32) {
        self.active_tunnels.push((pos, lifespan));
    }
}

pub fn trigger_megafauna_migration(world: &mut World, path: Vec<GridPosition>) {
    let mut grid = world.resource_mut::<TerrainGrid>();
    let mut to_damage = Vec::new();

    for pos in path {
        grid.set(pos.x, pos.y, pos.z, TerrainType::Tunnel);
        // Identify tiles directly above for tremor damage
        to_damage.push(GridPosition { x: pos.x, y: pos.y, z: pos.z - 1 });

        if let Some(mut tunnel_manager) = world.get_resource_mut::<TunnelManager>() {
            tunnel_manager.add_tunnel(pos, 60.0 * 5.0); // 5 minutes default
        }
    }

    // Apply tremor damage
    let mut query = world.query::<(Entity, &GridPosition, &mut Health)>();
    for (_entity, pos, mut health) in query.iter_mut(world) {
        if to_damage.contains(pos) {
            health.current -= 25.0; // Flat damage for tremors
        }
    }
}

pub fn update_tunnel_collapse_system(world: &mut World, delta_time: f32) {
    let mut collapsed = Vec::new();
    if let Some(mut tunnel_manager) = world.get_resource_mut::<TunnelManager>() {
        tunnel_manager.active_tunnels.retain_mut(|(pos, lifespan)| {
            *lifespan -= delta_time;
            if *lifespan <= 0.0 {
                collapsed.push(*pos);
                false
            } else {
                true
            }
        });
    }

    let mut grid = world.resource_mut::<TerrainGrid>();
    for pos in collapsed {
        grid.set(pos.x, pos.y, pos.z, TerrainType::SolidRock);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** Move `trigger_megafauna_migration` to be driven by a Bevy Event (`MegafaunaMigrationEvent`) rather than a direct function call to integrate cleanly with the ECS schedule.
- **Systemization:** `update_tunnel_collapse_system` should be a standard Bevy system that reads `Res<Time>` instead of accepting `delta_time` directly.
- **Pathfinding:** Update `TerrainType::Tunnel` to have a very low traversal cost in `src/layer1/pathfinding.rs` so haulers naturally prefer using them.
- **Safety:** Ensure tunnel collapse logic checks if entities are inside the tunnel when it collapses (crushing them or pushing them out).

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] Tunnels are created along the migration path.
- [ ] Tremors deal damage to buildings situated directly above the migration path.
- [ ] Tunnels collapse into `SolidRock` after their lifespan expires.
- [ ] Code has >=85% test coverage.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.

## 7. Technical Guidance
- `TerrainGrid` needs to support Z-levels effectively, ensure the map initialization handles deep levels.
- For tunnel collapse, emit a `TunnelCollapseEvent` so that the rendering layer can update visually and pathfinding caches can invalidate.
- Consider adding a `TremorEvent` to allow the chronicle system to log the event without hard-coupling it to the damage function.

## 8. Questions
*Builder: add questions here if spec is unclear.*
