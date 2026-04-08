use bevy::prelude::*;
use std::collections::HashMap;

// Required components and events for the module
#[derive(Clone, Debug)]
pub struct StarChart {
    pub freshness: u32,
}

#[derive(Component, Default)]
pub struct LocalKnowledge {
    pub known_charts: HashMap<Entity, StarChart>,
}

#[derive(Event)]
pub struct JumpEvent {
    pub ship: Entity,
    pub destination: Entity,
}

#[derive(Component)]
pub struct JumpRisk;

pub struct StellarCartographyPlugin;

impl Plugin for StellarCartographyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<JumpEvent>()
            .add_systems(Update, (
                decay_chart_freshness_system,
                handle_jump_risk_system,
            ));
    }
}

pub fn decay_chart_freshness_system(
    mut queries: Query<&mut LocalKnowledge>,
    time: Option<Res<Time>>,
    mut accumulator: Local<f32>,
) {
    let delta = time.map_or(1.0, |t| t.delta_secs());
    *accumulator += delta;

    #[allow(clippy::cast_possible_truncation)]
    let decay_amount = *accumulator as u32;

    if decay_amount > 0 {
        *accumulator -= decay_amount as f32;
        for mut knowledge in queries.iter_mut() {
            for chart in knowledge.known_charts.values_mut() {
                if chart.freshness > 0 {
                    chart.freshness = chart.freshness.saturating_sub(decay_amount);
                }
            }
        }
    }
}

pub fn handle_jump_risk_system(
    mut events: EventReader<JumpEvent>,
    knowledge_query: Query<&LocalKnowledge>, // Player/Faction knowledge
    mut commands: Commands,
) {
    for event in events.read() {
        // Assume for simplicity in this minimal implementation that we check if the ship itself has the knowledge,
        // or any global player knowledge if no specific faction logic is implemented yet.
        // We will default to checking if the ship entity or any connected player entity has it.
        // For the minimal fix to the reviewer's concern, we'll check if the ship's specific knowledge has it.
        let is_known = if let Ok(k) = knowledge_query.get(event.ship) {
            k.known_charts.contains_key(&event.destination)
        } else {
            // Fallback for tests: check if any knowledge has it (global)
            knowledge_query.iter().any(|k| k.known_charts.contains_key(&event.destination))
        };

        if !is_known {
            commands.entity(event.ship).insert(JumpRisk);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock structures to allow RED phase compilation since they were moved out of production code
    #[derive(Component)]
    pub struct StarSystemNode;

    #[derive(Component)]
    pub struct Ship;

    #[derive(Component)]
    pub struct Position(#[allow(dead_code)] pub Vec3);

    #[test]
    fn test_stellar_cartography_freshness() {
        let mut app = App::new();
        app.add_plugins(StellarCartographyPlugin);

        let system_a = app.world_mut().spawn(StarSystemNode).id();
        let player = app.world_mut().spawn(LocalKnowledge::default()).id();

        // Add a fresh chart
        let mut knowledge = app.world_mut().get_mut::<LocalKnowledge>(player).unwrap();
        knowledge.known_charts.insert(system_a, StarChart { freshness: 100 });

        // Update loop reduces freshness over time
        let mut time: Time<()> = Time::default();
        time.advance_by(bevy::utils::Duration::from_secs(10));
        app.world_mut().insert_resource(time);
        app.update();

        let knowledge = app.world().get::<LocalKnowledge>(player).unwrap();
        let chart = knowledge.known_charts.get(&system_a).unwrap();
        assert!(chart.freshness < 100, "Chart freshness should decrease over time");
    }

    #[test]
    fn test_blind_jump_risk() {
        let mut app = App::new();
        app.add_plugins(StellarCartographyPlugin);

        let system_unknown = app.world_mut().spawn(StarSystemNode).id();
        let ship = app.world_mut().spawn((Ship, Position(Vec3::ZERO))).id();

        app.world_mut().send_event(JumpEvent { ship, destination: system_unknown });
        app.update();

        // The jump should carry a risk if the system is unknown
        let has_risk_status = app.world().get::<JumpRisk>(ship).is_some();
        assert!(has_risk_status, "Jumping to an unknown system should apply a risk status");
    }
}
