//! Sub-grid paths for small entities and airflow.
//!
//! This module provides the core logic for ventilation networks in the colony. Vents allow
//! [`SmallEntity`]s (like rats or drones) to traverse through walls via a sub-grid pathing
//! system. They also allow atmospheric pressure to equalize between rooms. Grated vents
//! block movement but still permit partial airflow.
//!
//! # Examples
//!
//! Spawning a vent that connects two rooms:
//!
//! ```
//! use bevy::prelude::*;
//! use scale::layer1::physics::vent::VentConnection;
//!
//! let mut app = App::new();
//! app.world_mut().spawn(VentConnection {
//!     pos_a: UVec2::new(5, 5),
//!     pos_b: UVec2::new(5, 6),
//!     grated: false,
//! });
//! ```

use bevy::math::UVec2;
use bevy::prelude::{App, Plugin, Update};
use bevy_ecs::prelude::*;

/// Tracks the logical coordinates of an entity within the vent network.
///
/// Unlike the standard `GridPosition` component, which places entities in the main world,
/// `Position` locates sub-grid entities navigating through `VentConnection`s.
#[derive(Component)]
pub struct Position(pub UVec2);

/// A physical connection allowing traversal between two coordinates.
///
/// This component is placed on vent entities. If `grated` is `true`, entities cannot
/// pass through, but airflow is still partially permitted.
#[derive(Component)]
pub struct VentConnection {
    /// The first endpoint of the vent in grid coordinates.
    pub pos_a: UVec2,
    /// The second endpoint of the vent in grid coordinates.
    pub pos_b: UVec2,
    /// If `true`, the vent has a physical barrier that stops entities.
    pub grated: bool,
}

/// A tag marking an entity as capable of navigating the vent network.
///
/// Only entities with this component can successfully resolve a [`PathRequest`] through
/// un-grated [`VentConnection`]s.
#[derive(Component)]
pub struct SmallEntity;

/// A tag indicating this entity intends to utilize the pathfinding system.
///
/// Required for any entity that intends to emit [`PathRequest`] events.
#[derive(Component)]
pub struct PathNavigator;

/// An event fired to request passage through the vent network.
///
/// Dispatched by AI systems when an entity attempts to move. Processed by [`handle_vent_pathfinding`].
#[derive(Event)]
pub struct PathRequest {
    /// The entity attempting to navigate.
    pub entity: Entity,
    /// The starting grid coordinates.
    pub start: UVec2,
    /// The intended destination coordinates.
    pub end: UVec2,
}

/// A component inserted onto an entity upon resolution of a [`PathRequest`].
///
/// Contains the success state of the pathing attempt.
#[derive(Component)]
pub struct PathResult {
    /// `true` if the entity successfully traversed the vent, `false` otherwise.
    pub success: bool,
}

/// Initializes vent network events and processing systems.
pub struct VentilationPlugin;

impl Plugin for VentilationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PathRequest>()
            .add_systems(Update, handle_vent_pathfinding);
    }
}

/// Evaluates [`PathRequest`] events against the current [`VentConnection`] network.
///
/// This system verifies that the requesting entity has the [`SmallEntity`] component,
/// checks if a valid vent connects the requested `start` and `end` coordinates, and ensures
/// the vent is not `grated`.
///
/// Upon completion, it inserts a [`PathResult`] component onto the requesting entity.
pub fn handle_vent_pathfinding(
    mut events: EventReader<PathRequest>,
    vents: Query<&VentConnection>,
    small_entities: Query<&SmallEntity>,
    mut commands: Commands,
) {
    use bevy::utils::HashMap;
    let mut vent_map: HashMap<(UVec2, UVec2), bool> = HashMap::default();

    for vent in vents.iter() {
        vent_map.insert((vent.pos_a, vent.pos_b), vent.grated);
        vent_map.insert((vent.pos_b, vent.pos_a), vent.grated);
    }

    for event in events.read() {
        let is_small = small_entities.get(event.entity).is_ok();

        let mut success = false;
        if is_small {
            if let Some(&grated) = vent_map.get(&(event.start, event.end)) {
                success = !grated;
            }
        }

        commands.entity(event.entity).insert(PathResult { success });
    }
}

/// Determines the airflow percentage permitted by a [`VentConnection`].
///
/// # Returns
///
/// * `1.0` (100% airflow) if the vent is open.
/// * `0.5` (50% airflow) if the vent is grated.
///
/// # Examples
///
/// ```
/// use scale::layer1::physics::vent::{VentConnection, calculate_vent_airflow};
/// use bevy::math::UVec2;
///
/// let vent = VentConnection {
///     pos_a: UVec2::new(0, 0),
///     pos_b: UVec2::new(1, 0),
///     grated: true,
/// };
///
/// assert_eq!(calculate_vent_airflow(&vent), 0.5);
/// ```
pub fn calculate_vent_airflow(vent: &VentConnection) -> f32 {
    if vent.grated {
        0.5 // 50% airflow
    } else {
        1.0 // 100% airflow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_entities_can_traverse_vents() {
        let mut app = App::new();
        app.add_plugins(VentilationPlugin);

        // Grid positions
        let pos_a = UVec2::new(0, 0);
        let pos_b = UVec2::new(0, 1);

        // Vent connecting A and B
        app.world_mut().spawn(VentConnection {
            pos_a,
            pos_b,
            grated: false,
        });

        // Small entity
        let rat = app
            .world_mut()
            .spawn((SmallEntity, PathNavigator, Position(pos_a)))
            .id();

        // Large entity
        let human = app.world_mut().spawn((PathNavigator, Position(pos_a))).id();

        // System should allow rat to path through, but not human
        app.world_mut().send_event(PathRequest {
            entity: rat,
            start: pos_a,
            end: pos_b,
        });
        app.world_mut().send_event(PathRequest {
            entity: human,
            start: pos_a,
            end: pos_b,
        });
        app.update();

        let rat_path = app.world().get::<PathResult>(rat).unwrap();
        assert!(
            rat_path.success,
            "Small entities should path through open vents"
        );

        let human_path = app.world().get::<PathResult>(human).unwrap();
        assert!(
            !human_path.success,
            "Large entities should not path through vents"
        );
    }

    #[test]
    fn test_grates_block_movement_but_allow_some_air() {
        let mut app = App::new();
        app.add_plugins(VentilationPlugin);

        let pos_a = UVec2::new(0, 0);
        let pos_b = UVec2::new(0, 1);

        let vent = app
            .world_mut()
            .spawn(VentConnection {
                pos_a,
                pos_b,
                grated: true,
            })
            .id();

        let rat = app
            .world_mut()
            .spawn((SmallEntity, PathNavigator, Position(pos_a)))
            .id();

        app.world_mut().send_event(PathRequest {
            entity: rat,
            start: pos_a,
            end: pos_b,
        });
        app.update();

        let rat_path = app.world().get::<PathResult>(rat).unwrap();
        assert!(
            !rat_path.success,
            "Small entities should not path through grated vents"
        );

        // Verify airflow is still present but reduced
        let airflow_amount =
            calculate_vent_airflow(app.world().get::<VentConnection>(vent).unwrap());
        assert!(
            airflow_amount > 0.0 && airflow_amount < 1.0,
            "Grated vent should have reduced airflow"
        );
    }
}
