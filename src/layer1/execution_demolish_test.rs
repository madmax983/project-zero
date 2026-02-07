#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::execution::{AtTarget, MovementTarget, work_execution_system};
    use crate::layer1::housing::Housing;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::utility_ai::{ActionType, PopAction};
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        crate::setup::init_task_pools();
        let mut world = World::new();
        // Setup Grid
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources::default());
        world
    }

    #[test]
    fn test_demolish_execution_removes_building_and_designation() {
        let mut world = setup_world();

        // 1. Spawn a Building (Housing) at (5, 5)
        let building_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 5, y: 5 },
                Housing::default(),
            ))
            .id();

        // Mark as occupied
        world.resource_mut::<OccupiedTiles>().0.insert((5, 5));

        // 2. Spawn a Designation (Demolish) at (5, 5)
        let designation_entity = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Demolish,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // 3. Spawn a Pop assigned to work there
        // Pop is already at target (5, 5) or adjacent (works from adjacent usually, but let's assume at target for simplicity of test setup as work_execution checks AtTarget)
        // Wait, work_execution_system checks AtTarget.
        let pop_entity = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                MovementTarget {
                    target_entity: designation_entity,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
                PopAction {
                    current: ActionType::Work,
                    current_utility: 0.8,
                    ticks_committed: 10,
                },
                // Needs required for morale check in work_execution_system?
                // It uses Option<&Needs>, defaulting to 0.5 morale if missing.
            ))
            .id();

        // 4. Run work_execution_system
        // This should trigger the demolish logic (if implemented)
        work_execution_system(&mut world);

        // 5. Assertions

        // A. Building should be despawned
        assert!(
            world.get_entity(building_entity).is_err(),
            "Building should be despawned after demolition"
        );

        // B. Designation should be despawned
        assert!(
            world.get_entity(designation_entity).is_err(),
            "Designation should be despawned after demolition"
        );

        // C. OccupiedTiles should be cleared
        let occupied = world.resource::<OccupiedTiles>();
        assert!(
            !occupied.0.contains(&(5, 5)),
            "Tile (5,5) should no longer be occupied"
        );

        // D. Pop should be reset (MovementTarget removed)
        assert!(
            world.get::<MovementTarget>(pop_entity).is_none(),
            "Pop should have MovementTarget removed"
        );
    }
}
