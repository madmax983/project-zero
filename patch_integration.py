import sys

with open("src/layer1/core/integration.rs", "r") as f:
    content = f.read()

new_system = """

/// INT-1306: Architectural Superstition Bridge
///
/// Listens to `PopDied`, `BuildingRemovedEvent`, and `PopDiedInAccidentEvent`.
/// When these occur, it looks for nearby buildings and adds a `NegativeEvent` to their `NegativeEventHistory`.
#[allow(clippy::type_complexity)]
pub fn track_negative_events_bridge_system(
    mut pop_died_events: bevy_ecs::event::EventReader<crate::layer1::pop::PopDied>,
    mut building_removed_events: bevy_ecs::event::EventReader<crate::layer1::core::events::BuildingRemovedEvent>,
    mut pop_died_accident_events: bevy_ecs::event::EventReader<crate::layer1::haunted_assembly_lines::PopDiedInAccidentEvent>,
    pops_query: bevy_ecs::system::Query<&crate::layer1::map::GridPosition, bevy_ecs::query::With<crate::layer1::pop::Pop>>,
    pos_query: bevy_ecs::system::Query<&crate::layer1::map::GridPosition>,
    mut building_query: bevy_ecs::system::Query<(
        &crate::layer1::map::GridPosition,
        &mut crate::layer1::architecture_superstition::NegativeEventHistory,
    ), bevy_ecs::query::With<crate::layer1::architecture::Building>>,
    time: bevy_ecs::system::Res<crate::shared::time::SimulationTime>,
) {
    use crate::layer1::architecture_superstition::NegativeEvent;

    let mut negative_locations = Vec::new();

    for event in pop_died_events.read() {
        if let Ok(pos) = pops_query.get(event.entity) {
            negative_locations.push((*pos, 5.0)); // Base severity for death
        }
    }

    for event in building_removed_events.read() {
        negative_locations.push((event.position, 3.0)); // Base severity for destruction
    }

    for event in pop_died_accident_events.read() {
        if let Ok(pos) = pos_query.get(event.location) {
            negative_locations.push((*pos, 7.0)); // Higher severity for workplace accident
        }
    }

    if negative_locations.is_empty() {
        return;
    }

    for (b_pos, mut history) in building_query.iter_mut() {
        for (event_pos, severity) in &negative_locations {
            let dx = b_pos.x.abs_diff(event_pos.x);
            let dy = b_pos.y.abs_diff(event_pos.y);
            // If within 5 tiles
            if dx <= 5 && dy <= 5 {
                history.events.push(NegativeEvent {
                    severity: *severity,
                    #[allow(clippy::cast_precision_loss)]
                    time: time.tick as f32,
                });
            }
        }
    }
}
"""

with open("src/layer1/core/integration.rs", "w") as f:
    f.write(content + new_system)

with open("src/layer1/systems/observation.rs", "r") as f:
    obs = f.read()

obs = obs.replace("crate::layer1::architecture_superstition::evaluate_architectural_superstition,", "crate::layer1::core::integration::track_negative_events_bridge_system,\n            crate::layer1::architecture_superstition::evaluate_architectural_superstition,")

with open("src/layer1/systems/observation.rs", "w") as f:
    f.write(obs)
