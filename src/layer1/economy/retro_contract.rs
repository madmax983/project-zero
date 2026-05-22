use crate::layer1::economy::resources::ColonyResources;
use crate::shared::time::{SimSpeed, SimulationTime};
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct AcceptRetroContractEvent {
    pub credit_advance: f32,
    pub deadline_days: f32,
}

#[derive(Component)]
pub struct RetroContract {
    pub days_remaining: f32,
    pub penalty: f32,
    pub is_fulfilled: bool,
}

pub fn handle_retro_contract_acceptance(
    mut commands: Commands,
    mut events: EventReader<AcceptRetroContractEvent>,
    mut resources: ResMut<ColonyResources>,
) {
    for event in events.read() {
        resources.credits += event.credit_advance;
        commands.spawn(RetroContract {
            days_remaining: event.deadline_days,
            penalty: event.credit_advance * 2.0,
            is_fulfilled: false,
        });
    }
}

pub fn evaluate_retro_contracts(
    mut commands: Commands,
    time: Res<SimulationTime>,
    mut contracts: Query<(Entity, &mut RetroContract)>,
    mut resources: ResMut<ColonyResources>,
) {
    if time.speed == SimSpeed::Paused {
        return;
    }

    let day_delta = 0.001;

    for (entity, mut contract) in contracts.iter_mut() {
        if contract.is_fulfilled {
            commands.entity(entity).despawn();
            continue;
        }

        contract.days_remaining -= day_delta;

        if contract.days_remaining <= 0.0 {
            resources.credits -= contract.penalty;
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_accepting_retro_contract_grants_immediate_resources() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(ColonyResources::default());
        app.add_event::<AcceptRetroContractEvent>();
        app.add_systems(Update, handle_retro_contract_acceptance);

        // Act
        app.world_mut().send_event(AcceptRetroContractEvent {
            credit_advance: 1000.0,
            deadline_days: 10.0,
        });
        app.update();

        // Assert
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.credits, 1000.0);

        let contracts = app
            .world_mut()
            .query::<&RetroContract>()
            .iter(app.world())
            .count();
        assert_eq!(contracts, 1);
    }

    #[test]
    fn test_failing_retro_contract_applies_penalty() {
        // Arrange
        let mut app = App::new();
        let time = SimulationTime::default();
        app.insert_resource(time);
        let resources = ColonyResources {
            credits: 500.0,
            ..Default::default()
        };
        app.insert_resource(resources);
        app.add_systems(Update, evaluate_retro_contracts);

        app.world_mut().spawn(RetroContract {
            days_remaining: 0.001,
            penalty: 2000.0,
            is_fulfilled: false,
        });

        // Act: Advance time past the deadline
        app.update();

        // Assert: The penalty should be applied
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.credits, -1500.0); // 500 - 2000
    }

    #[test]
    fn test_fulfilled_retro_contract_despawns() {
        // Arrange
        let mut app = App::new();
        let time = SimulationTime::default();
        app.insert_resource(time);
        let resources = ColonyResources {
            credits: 500.0,
            ..Default::default()
        };
        app.insert_resource(resources);
        app.add_systems(Update, evaluate_retro_contracts);

        app.world_mut().spawn(RetroContract {
            days_remaining: 10.0,
            penalty: 2000.0,
            is_fulfilled: true,
        });

        app.update();

        // Assert: The penalty should NOT be applied, entity despawned
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.credits, 500.0);
        let contracts = app
            .world_mut()
            .query::<&RetroContract>()
            .iter(app.world())
            .count();
        assert_eq!(contracts, 0);
    }

    #[test]
    fn test_paused_simulation_does_not_evaluate() {
        // Arrange
        let mut app = App::new();
        let time = SimulationTime {
            speed: SimSpeed::Paused,
            ..Default::default()
        };
        app.insert_resource(time);
        let resources = ColonyResources {
            credits: 500.0,
            ..Default::default()
        };
        app.insert_resource(resources);
        app.add_systems(Update, evaluate_retro_contracts);

        app.world_mut().spawn(RetroContract {
            days_remaining: 0.001,
            penalty: 2000.0,
            is_fulfilled: false,
        });

        app.update();

        // Assert: days_remaining should be unchanged
        let contracts = app
            .world_mut()
            .query::<&RetroContract>()
            .iter(app.world())
            .count();
        assert_eq!(contracts, 1);
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.credits, 500.0);
    }
}
