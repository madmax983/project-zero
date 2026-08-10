use crate::layer2::fleet::{Fleet, FleetFaction};
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ProtocolRule {
    NoMiningRedPlanets,
}

#[derive(Component)]
pub struct DeadProtocol {
    pub rule: ProtocolRule,
}

#[derive(Event)]
pub struct ViolationEvent {
    pub target: Entity,
}

pub fn protocol_violation_system(
    mut commands: Commands,
    mut events: EventReader<ViolationEvent>,
    query: Query<&DeadProtocol>,
    mut log: Option<ResMut<MessageLog>>,
) {
    for event in events.read() {
        if let Ok(protocol) = query.get(event.target) {
            if protocol.rule == ProtocolRule::NoMiningRedPlanets {
                commands.spawn((Fleet, FleetFaction::AncientEnforcer));
                if let Some(ref mut l) = log {
                    l.add_colored(
                        "Dead protocol violated: ancient enforcers awakened!",
                        ratatui::style::Color::Red,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::fleet::FleetFaction;

    #[test]
    fn test_mining_red_planet_spawns_enforcers() {
        let mut app = bevy_app::App::new();
        app.add_event::<ViolationEvent>();
        app.add_systems(bevy_app::Update, protocol_violation_system);

        // Setup the protocol
        let planet = app
            .world_mut()
            .spawn(DeadProtocol {
                rule: ProtocolRule::NoMiningRedPlanets,
            })
            .id();

        // Trigger violation
        app.world_mut()
            .send_event(ViolationEvent { target: planet });

        app.update();

        // Check if enforcer fleet spawned
        let mut enforcers_spawned = false;
        let mut query = app.world_mut().query::<&FleetFaction>();
        for faction in query.iter(app.world()) {
            if *faction == FleetFaction::AncientEnforcer {
                enforcers_spawned = true;
            }
        }

        assert!(
            enforcers_spawned,
            "Enforcers should spawn on protocol violation."
        );
    }
}
