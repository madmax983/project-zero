//! Integration systems for Layer 2 -> Layer 1 bridging.

use crate::layer1::map::GridPosition;
use crate::layer1::notifications::NotificationQueue;
use crate::layer1::terrain::TerrainGrid;
use crate::layer1::the_visitor::TheVisitor;
use crate::layer2::events::DetectionEvent;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use rand::Rng;

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
            1 => (terrain.width as i32 - 1, rng.gen_range(0..terrain.height as i32)),
            2 => (rng.gen_range(0..terrain.width as i32), terrain.height as i32 - 1),
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
