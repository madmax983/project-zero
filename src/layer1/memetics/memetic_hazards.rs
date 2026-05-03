use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Researcher {
    pub xp: u32,
}

#[derive(Component)]
pub struct ActiveResearch {
    pub is_hazardous: bool,
    pub progress: f32,
}

#[derive(Component)]
pub struct VulnerableMind;


#[derive(Event)]
pub struct ConversationEvent {
    pub initiator: Entity,
    pub receiver: Entity,
}

#[derive(Component)]
pub struct TaskEfficiency {
    pub value: f32,
}

pub fn process_artifact_research_system(
    mut commands: Commands,
    mut query: Query<(Entity, &ActiveResearch, &mut Researcher)>,
) {
    for (entity, research, mut researcher) in query.iter_mut() {
        if research.progress >= 100.0 {
            researcher.xp += 1000;
            if research.is_hazardous {
                commands
                    .entity(entity)
                    .insert(crate::layer1::memetics::MemeticCarrier);
            }
            // Assuming we remove the research after completion
            commands.entity(entity).remove::<ActiveResearch>();
        }
    }
}

pub fn spread_memetic_hazard_system(
    mut commands: Commands,
    mut events: EventReader<ConversationEvent>,
    carrier_query: Query<&crate::layer1::memetics::MemeticCarrier>,
    vulnerable_query: Query<&VulnerableMind>,
) {
    for event in events.read() {
        if carrier_query.get(event.initiator).is_ok()
            && vulnerable_query.get(event.receiver).is_ok()
        {
            commands
                .entity(event.receiver)
                .insert(crate::layer1::memetics::MemeticCarrier);
        }
    }
}

pub fn apply_obsession_penalty_system(
    mut query: Query<&mut TaskEfficiency, With<crate::layer1::memetics::MemeticCarrier>>,
) {
    for mut efficiency in query.iter_mut() {
        efficiency.value = 0.5; // Simulate distraction
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    // Needs imports for MemeticCarrier, Pop
    use crate::layer1::memetics::MemeticCarrier;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_researching_alien_artifact_grants_memetic_virus() {
        let mut app = App::new();
        app.add_systems(Update, process_artifact_research_system);

        // Arrange
        let entity = app
            .world_mut()
            .spawn((
                Researcher { xp: 0 },
                ActiveResearch {
                    is_hazardous: true,
                    progress: 100.0,
                },
            ))
            .id();

        // Act
        app.update();

        // Assert
        assert!(
            app.world().get::<MemeticCarrier>(entity).is_some(),
            "Completing hazardous research should apply the MemeticCarrier trait"
        );
    }

    #[test]
    fn test_memetic_virus_spreads_via_conversation() {
        let mut app = App::new();
        app.add_systems(Update, spread_memetic_hazard_system);
        app.add_event::<ConversationEvent>();

        // Arrange
        let carrier = app.world_mut().spawn((Pop, MemeticCarrier)).id();
        let target = app.world_mut().spawn((Pop, VulnerableMind)).id();

        app.world_mut().send_event(ConversationEvent {
            initiator: carrier,
            receiver: target,
        });

        // Act
        app.update();

        // Assert
        assert!(
            app.world().get::<MemeticCarrier>(target).is_some(),
            "Target pop should contract the virus after conversation with a carrier"
        );
    }

    #[test]
    fn test_memetic_carrier_exhibits_obsessive_behavior() {
        let mut app = App::new();
        app.add_systems(Update, apply_obsession_penalty_system);

        // Arrange
        let carrier = app
            .world_mut()
            .spawn((Pop, MemeticCarrier, TaskEfficiency { value: 1.0 }))
            .id();

        // Act
        app.update();

        // Assert
        let efficiency = app.world().get::<TaskEfficiency>(carrier).unwrap();
        assert!(
            efficiency.value < 1.0,
            "Memetic carriers should suffer task efficiency penalties due to obsession"
        );
    }
}
