use crate::layer1::building::{Building, BuildingType};
use crate::layer1::energy::PowerConsumer;
use crate::shared::view_mode::ViewMode;
use bevy_ecs::prelude::*;

/// Defines the visibility state of the system view (Layer 2).
#[derive(Resource, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum SystemVisibility {
    /// No visibility. System view is inaccessible.
    #[default]
    None,
    /// Full visibility. System view is accessible.
    Full,
}

/// Updates the `SystemVisibility` resource based on the presence of a powered Command Center.
pub fn update_visibility_system(
    mut visibility: ResMut<SystemVisibility>,
    query: Query<(&Building, &PowerConsumer)>,
) {
    let mut has_active_cc = false;

    for (building, power) in &query {
        if building.building_type == BuildingType::CommandCenter && power.active {
            has_active_cc = true;
            break;
        }
    }

    *visibility = if has_active_cc {
        SystemVisibility::Full
    } else {
        SystemVisibility::None
    };
}

/// Forces the view mode back to Colony if system visibility is lost.
pub fn enforce_view_mode_system(
    visibility: Res<SystemVisibility>,
    mut view_mode: ResMut<ViewMode>,
) {
    if *view_mode == ViewMode::System && *visibility == SystemVisibility::None {
        *view_mode = ViewMode::Colony;
        // TODO: Add "SIGNAL LOST" notification
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::energy::PowerConsumer;
    use crate::shared::view_mode::ViewMode;

    #[test]
    fn test_visibility_defaults_to_none() {
        let mut world = World::new();
        world.init_resource::<SystemVisibility>();
        assert_eq!(
            *world.resource::<SystemVisibility>(),
            SystemVisibility::None
        );
    }

    #[test]
    fn test_powered_command_center_grants_visibility() {
        let mut world = World::new();
        world.init_resource::<SystemVisibility>();

        // Spawn a powered Command Center
        world.spawn((
            Building {
                building_type: BuildingType::CommandCenter,
            },
            PowerConsumer {
                active: true,
                demand: 10.0,
            },
        ));

        // Run visibility update system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_visibility_system);
        schedule.run(&mut world);

        assert_eq!(
            *world.resource::<SystemVisibility>(),
            SystemVisibility::Full
        );
    }

    #[test]
    fn test_unpowered_command_center_denies_visibility() {
        let mut world = World::new();
        world.init_resource::<SystemVisibility>();

        // Spawn an unpowered Command Center
        world.spawn((
            Building {
                building_type: BuildingType::CommandCenter,
            },
            PowerConsumer {
                active: false,
                demand: 10.0,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_visibility_system);
        schedule.run(&mut world);

        assert_eq!(
            *world.resource::<SystemVisibility>(),
            SystemVisibility::None
        );
    }

    #[test]
    fn test_loss_of_visibility_forces_colony_view() {
        let mut world = World::new();
        world.insert_resource(SystemVisibility::None);
        world.insert_resource(ViewMode::System); // User is currently looking at system

        // Run enforcement system
        let mut schedule = Schedule::default();
        schedule.add_systems(enforce_view_mode_system);
        schedule.run(&mut world);

        // Should be forced back to Colony view
        assert_eq!(*world.resource::<ViewMode>(), ViewMode::Colony);
    }
}
