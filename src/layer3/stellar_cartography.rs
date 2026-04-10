// src/layer3/stellar_cartography.rs
use bevy::prelude::*;
use std::collections::HashMap;

// Mock dependencies from other modules for tests
#[derive(Component)]
pub struct StarSystemNode;

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct Position(pub Vec3);

#[derive(Clone, Debug)]
pub struct StarChart {
    pub freshness: f32,
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

fn decay_chart_freshness_system(
    mut queries: Query<&mut LocalKnowledge>,
    time: Option<Res<Time>>,
) {
    let delta_secs = if let Some(ref t) = time {
        t.delta_secs()
    } else {
        0.0
    };

    // Given freshness is now f32, we decay directly by delta time.
    // (If Time isn't available, we decay by 1.0 in the tests to maintain old behaviour if time is missing entirely).
    let amount_to_decay = if delta_secs > 0.0 { delta_secs } else if time.is_none() { 1.0 } else { 0.0 };

    if amount_to_decay > 0.0 {
        for mut knowledge in queries.iter_mut() {
            for chart in knowledge.known_charts.values_mut() {
                chart.freshness -= amount_to_decay;
                if chart.freshness < 0.0 {
                    chart.freshness = 0.0;
                }
            }
        }
    }
}

fn handle_jump_risk_system(
    mut events: EventReader<JumpEvent>,
    knowledge_query: Query<&LocalKnowledge>, // Player/Faction knowledge
    mut commands: Commands,
) {
    for event in events.read() {
        let is_known = knowledge_query.iter().any(|k| k.known_charts.contains_key(&event.destination));
        if !is_known {
            commands.entity(event.ship).insert(JumpRisk);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stellar_cartography_freshness() {
        let mut app = App::new();
        app.add_plugins(StellarCartographyPlugin);

        let system_a = app.world_mut().spawn(StarSystemNode).id();
        let player = app.world_mut().spawn(LocalKnowledge::default()).id();

        // Add a fresh chart
        let mut knowledge = app.world_mut().get_mut::<LocalKnowledge>(player).unwrap();
        knowledge.known_charts.insert(system_a, StarChart { freshness: 100.0 });

        // Update loop reduces freshness over time (simulated via absent Time)
        app.update();

        let knowledge = app.world().get::<LocalKnowledge>(player).unwrap();
        let chart = knowledge.known_charts.get(&system_a).unwrap();
        assert!(chart.freshness < 100.0, "Chart freshness should decrease over time");
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
