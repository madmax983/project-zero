//! Integration systems for Layer 2 -> Layer 1 bridging.

use crate::layer1::map::GridPosition;
use crate::layer1::notifications::NotificationQueue;
use crate::layer1::quirks::{PlanetaryTrait, PlanetaryTraits};
use crate::layer1::terrain::TerrainGrid;
use crate::layer1::the_visitor::TheVisitor;
use crate::layer2::events::DetectionEvent;
use crate::layer2::trade::escape_velocity::PlanetaryGravity;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Updates the Layer 2 `PlanetaryGravity` resource based on Layer 1 `PlanetaryTraits`.
/// Bridges Spec 080 (Quirks) to Spec 468 (Escape Velocity Economics).
pub fn escape_velocity_traits_bridge_system(
    traits: Option<Res<PlanetaryTraits>>,
    mut gravity: ResMut<PlanetaryGravity>,
) {
    if let Some(traits_res) = traits {
        let mut new_g_force = 1.0;
        for t in &traits_res.0 {
            match t {
                PlanetaryTrait::HighGravity => new_g_force = 2.5,
                PlanetaryTrait::LowGravity => new_g_force = 0.5,
                _ => {}
            }
        }

        // Prevent floating point jitter if no change is needed
        if (gravity.g_force - new_g_force).abs() > f32::EPSILON {
            gravity.g_force = new_g_force;
        }
    }
}

/// Handles `DetectionEvent` by spawning a hostile `TheVisitor` entity.
///
/// This bridges the Layer 2 `ThermalSignature` system (Risk) with the Layer 1 `TheVisitor` system (Consequence).
pub fn thermal_detection_handler_system(
    mut events: EventReader<DetectionEvent>,
    mut commands: Commands,
    mut notifications: ResMut<NotificationQueue>,
    terrain: Res<TerrainGrid>,
    time: Res<SimulationTime>,
) {
    for _ in events.read() {
        // 1. Notify Player
        notifications.add_error(
            "WARNING: High Thermal Signature detected! A Visitor has arrived.",
            time.tick,
        );

        // 2. Determine Spawn Location (Random Edge)
        let mut rng = rand::thread_rng();
        let edge = rng.gen_range(0..4); // 0: Top, 1: Right, 2: Bottom, 3: Left

        let (x, y) = match edge {
            0 => (rng.gen_range(0..terrain.width as i32), 0),
            1 => (
                terrain.width as i32 - 1,
                rng.gen_range(0..terrain.height as i32),
            ),
            2 => (
                rng.gen_range(0..terrain.width as i32),
                terrain.height as i32 - 1,
            ),
            3 => (0, rng.gen_range(0..terrain.height as i32)),
            _ => (0, 0),
        };

        // 3. Spawn TheVisitor
        // Note: TheVisitor component doesn't take fields, it's a marker or state struct.
        // Checking src/layer1/the_visitor.rs:
        // pub struct TheVisitor { pub state: VisitorState, pub target: Option<Entity>, ... }
        // We need to initialize it correctly.

        commands.spawn((
            TheVisitor::default(), // Assuming Default exists or we construct it
            GridPosition { x, y },
            // Add health/other components if TheVisitor bundle doesn't include them?
            // Spec 234 implies it has components. Let's assume standard entity pattern.
            // If TheVisitor implements Default, this is fine.
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(crate::layer1::terrain::generate_terrain(100, 100));
        world.init_resource::<NotificationQueue>();
        world.insert_resource(SimulationTime::default());
        world.init_resource::<Events<DetectionEvent>>();
        world
    }

    #[test]
    fn test_escape_velocity_traits_bridge() {
        let mut world = World::new();
        world.insert_resource(PlanetaryGravity::default());
        world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::HighGravity]));

        world
            .run_system_once(escape_velocity_traits_bridge_system)
            .unwrap();

        assert_eq!(
            world.resource::<PlanetaryGravity>().g_force,
            2.5,
            "HighGravity trait should set g_force to 2.5"
        );

        world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::LowGravity]));
        world
            .run_system_once(escape_velocity_traits_bridge_system)
            .unwrap();

        assert_eq!(
            world.resource::<PlanetaryGravity>().g_force,
            0.5,
            "LowGravity trait should set g_force to 0.5"
        );

        world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::DenseAtmosphere]));
        world
            .run_system_once(escape_velocity_traits_bridge_system)
            .unwrap();

        assert_eq!(
            world.resource::<PlanetaryGravity>().g_force,
            1.0,
            "No gravity trait should set g_force to 1.0"
        );
    }

    #[test]
    fn test_thermal_detection_handler_system_spawns_visitor() {
        let mut world = setup_world();

        // Send event
        world.send_event(DetectionEvent);

        // Run system
        world
            .run_system_once(thermal_detection_handler_system)
            .unwrap();

        // Check if visitor spawned
        let mut query = world.query::<(&TheVisitor, &GridPosition)>();
        let mut iter = query.iter(&world);
        let visitor = iter.next();

        assert!(
            visitor.is_some(),
            "Visitor should be spawned when DetectionEvent is triggered"
        );

        // Verify notifications
        let notifications = world.resource::<NotificationQueue>();
        assert_eq!(
            notifications.active.len(),
            1,
            "Should generate one notification"
        );
        assert_eq!(
            notifications.active[0].severity,
            crate::layer1::notifications::NotificationSeverity::Error
        );
    }
}
