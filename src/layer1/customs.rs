use crate::layer1::execution::components::MovementTarget;
use crate::layer1::map::GridPosition;
use crate::layer1::utility_types::ActionType;
use crate::layer1::zone::{ZoneGrid, ZoneType};
use bevy_ecs::prelude::*;

/// Status of an entity entering the colony.
#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub enum ImmigrationStatus {
    /// Newly arrived, needs vetting.
    #[default]
    Pending,
    /// Currently being vetted (0-100%).
    Processing(u32),
    /// Successfully vetted and allowed entry.
    Vetted,
    /// Rejected due to hidden traits or policy.
    Rejected(String),
    /// Detained for further processing.
    Detained,
}

/// Hidden traits that are revealed during vetting.
#[derive(Component, Debug, Clone, Default)]
pub struct HiddenTraits(pub Vec<String>);

/// Job component for customs officers.
#[derive(Component, Default)]
pub struct CustomsOfficer;

/// Directs pending immigrants to the nearest Customs zone.
pub fn immigration_interception_system(
    zone_grid: Res<ZoneGrid>,
    query: Query<(Entity, &ImmigrationStatus), Without<MovementTarget>>,
    mut commands: Commands,
) {
    if query.is_empty() {
        return;
    }

    let customs_zones: Vec<GridPosition> = zone_grid
        .grid
        .iter()
        .enumerate()
        .filter_map(|(i, z)| {
            if *z == ZoneType::Customs {
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                Some(GridPosition {
                    x: (i % zone_grid.width) as i32,
                    y: (i / zone_grid.width) as i32,
                })
            } else {
                None
            }
        })
        .collect();

    if customs_zones.is_empty() {
        return;
    }

    for (entity, status) in &query {
        if *status == ImmigrationStatus::Pending {
            // Find closest customs zone
            // For GREEN phase, just pick the first one.
            // TODO: Implement closest distance logic in Refactor phase.
            let target_pos = customs_zones[0];

            commands.entity(entity).insert(MovementTarget {
                target_entity: entity,
                target_position: target_pos,
                for_action: ActionType::Idle,
            });
        }
    }
}

/// Simulates the vetting process.
pub fn vetting_work_system(
    mut query: Query<(&mut ImmigrationStatus, Option<&HiddenTraits>, &GridPosition)>,
    zone_grid: Res<ZoneGrid>,
    _officers: Query<&GridPosition, With<CustomsOfficer>>,
) {
    // In a real system, we'd check if an officer is working on this entity.
    // For GREEN phase, we simulate progress automatically.

    for (mut status, hidden, pos) in &mut query {
        // Ensure entity is at a Customs checkpoint
        if zone_grid.get(pos.x, pos.y) != ZoneType::Customs {
            continue;
        }

        match *status {
            ImmigrationStatus::Pending => {
                *status = ImmigrationStatus::Processing(0);
            }
            ImmigrationStatus::Processing(progress) => {
                if progress >= 100 {
                    // Vetting complete
                    let is_smuggler = hidden.is_some_and(|h| h.0.contains(&"Smuggler".to_string()));

                    if is_smuggler {
                        *status = ImmigrationStatus::Rejected("Smuggler".to_string());
                    } else {
                        *status = ImmigrationStatus::Vetted;
                    }
                } else {
                    *status = ImmigrationStatus::Processing(progress + 10);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::execution::components::MovementTarget;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::zone::{ZoneGrid, ZoneType};

    #[test]
    fn test_new_visitor_is_pending() {
        // Arrange
        let mut world = World::new();
        // Simulate spawning a visitor (usually done by visitor system, but we test the component default)
        let entity = world
            .spawn((Pop::default(), ImmigrationStatus::default()))
            .id();

        // Assert
        let status = world.get::<ImmigrationStatus>(entity).unwrap();
        assert_eq!(*status, ImmigrationStatus::Pending);
    }

    #[test]
    fn test_interception_logic() {
        // Arrange: Entity is Pending and not in Customs Zone
        let mut world = World::new();
        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(5, 5, ZoneType::Customs);
        world.insert_resource(zone_grid);

        let entity = world
            .spawn((
                Pop::default(),
                ImmigrationStatus::Pending,
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Act: Run interception system
        let mut schedule = Schedule::default();
        schedule.add_systems(immigration_interception_system);
        schedule.run(&mut world);

        // Assert: MovementTarget should be set to Customs Zone (5, 5)
        let target = world
            .get::<MovementTarget>(entity)
            .expect("Should have MovementTarget");
        assert_eq!(target.target_position, GridPosition { x: 5, y: 5 });
    }

    #[test]
    fn test_vetting_process_success() {
        // Arrange: Officer and Pending entity in Customs Zone
        let mut world = World::new();
        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(5, 5, ZoneType::Customs);
        world.insert_resource(zone_grid);

        let pending_pop = world
            .spawn((
                Pop::default(),
                ImmigrationStatus::Pending,
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let _officer = world
            .spawn((
                Pop::default(),
                CustomsOfficer::default(), // Marker for job
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Act: Run vetting system (simulate 1 tick of work)
        let mut schedule = Schedule::default();
        schedule.add_systems(vetting_work_system);

        // Run multiple ticks to complete vetting (assuming 10% per tick)
        for _ in 0..12 {
            // 0->Processing(0), +10 each tick... needs 11 ticks to reach 100+
            schedule.run(&mut world);
        }

        // Assert: Status changed to Vetted
        let status = world.get::<ImmigrationStatus>(pending_pop).unwrap();
        assert_eq!(*status, ImmigrationStatus::Vetted);
    }

    #[test]
    fn test_vetting_reveals_traits() {
        // Arrange: Pending pop with HiddenTraits
        let mut world = World::new();
        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(5, 5, ZoneType::Customs);
        world.insert_resource(zone_grid);

        let pending_pop = world
            .spawn((
                Pop::default(),
                ImmigrationStatus::Pending,
                HiddenTraits(vec!["Smuggler".to_string()]),
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let _officer = world
            .spawn((
                Pop::default(),
                CustomsOfficer::default(), // Marker for job
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Act: Vet
        let mut schedule = Schedule::default();
        schedule.add_systems(vetting_work_system);

        for _ in 0..12 {
            schedule.run(&mut world);
        }

        // Assert: HiddenTrait removed/converted to real Trait, Status is Rejected (if logic dictates)
        // For MVP, just check status is Rejected due to Smuggler
        let status = world.get::<ImmigrationStatus>(pending_pop).unwrap();
        match status {
            ImmigrationStatus::Rejected(reason) => assert_eq!(reason, "Smuggler"),
            _ => panic!("Should be rejected, got {:?}", status),
        }
    }
}
