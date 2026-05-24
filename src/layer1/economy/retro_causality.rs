use bevy::prelude::*;


#[derive(Event)]
pub struct AcceptRetroContractEvent {
    pub credit_advance: i32,
    pub deadline_days: f32,
}

#[derive(Component)]
pub struct RetroContract {
    pub days_remaining: f32,
    pub penalty: i32,
    pub is_fulfilled: bool,
}

pub fn handle_retro_contract_acceptance(
    mut commands: Commands,
    mut events: EventReader<AcceptRetroContractEvent>,
    mut resources: ResMut<crate::layer1::economy::ColonyResources>,
) {
    for event in events.read() {
        // Immediate payout
        resources.credits += event.credit_advance as f32;

        // Create the obligation
        commands.spawn(RetroContract {
            days_remaining: event.deadline_days,
            penalty: event.credit_advance * 2, // Standard 2x penalty for MVP
            is_fulfilled: false,
        });
    }
}

pub fn evaluate_retro_contracts(
    mut commands: Commands,
    time: Res<Time>,
    mut contracts: Query<(Entity, &mut RetroContract)>,
    mut resources: ResMut<crate::layer1::economy::ColonyResources>,
) {
    // Standard conversion: 100 seconds = 1 day (for MVP logic)
    let day_delta = time.delta_secs() / 100.0;

    for (entity, mut contract) in contracts.iter_mut() {
        if contract.is_fulfilled {
            commands.entity(entity).despawn();
            continue;
        }

        contract.days_remaining -= day_delta;

        if contract.days_remaining <= 0.0 {
            // Failure! Apply penalty
            resources.credits -= contract.penalty as f32;
            commands.entity(entity).despawn();

            // Note: Future specs might trigger hostile faction events here instead of simple credit drain.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_accepting_retro_contract_grants_immediate_resources() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(crate::layer1::economy::ColonyResources { ..Default::default() });
        app.add_event::<AcceptRetroContractEvent>();
        app.add_systems(Update, handle_retro_contract_acceptance);

        // Act
        app.world_mut().send_event(AcceptRetroContractEvent {
            credit_advance: 1000,
            deadline_days: 10.0,
        });
        app.update();

        // Assert
        let resources = app.world().resource::<crate::layer1::economy::ColonyResources>();
        assert!((resources.credits - 1000.0).abs() < f32::EPSILON);

        let contracts = app.world_mut().query::<&RetroContract>().iter(app.world()).count();
        assert_eq!(contracts, 1);
    }

    #[test]
    fn test_failing_retro_contract_applies_penalty() {
        // Arrange
        let mut app = App::new();
        let mut virtual_time = Time::<Virtual>::default();
        virtual_time.unpause();
        virtual_time.advance_by(std::time::Duration::from_secs(100));
        app.insert_resource(virtual_time);
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs(100));
        app.insert_resource(time);

        let mut initial_resources = crate::layer1::economy::ColonyResources { ..Default::default() };
        initial_resources.credits = 500.0;
        app.insert_resource(initial_resources);

        app.add_systems(Update, evaluate_retro_contracts);

        app.world_mut().spawn(RetroContract {
            days_remaining: 1.0,
            penalty: 2000,
            is_fulfilled: false,
        });

        // Act: Advance time past the deadline
        app.update();

        // Assert: The penalty should be applied
        let resources = app.world().resource::<crate::layer1::economy::ColonyResources>();
        assert!((resources.credits - -1500.0).abs() < f32::EPSILON); // 500 - 2000
    }
}
