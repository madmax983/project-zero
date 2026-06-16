use crate::layer1::entities::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::Traits;
use bevy_ecs::prelude::*;

#[derive(Event, Debug, Clone)]
pub struct SurgeryEvent {
    pub patient: Entity,
    pub procedure: String,
}

#[derive(Component)]
pub struct IntegratedCollective;

pub fn process_integration_surgery_system(
    mut commands: Commands,
    mut events: EventReader<SurgeryEvent>,
    mut query: Query<&mut Traits, With<Pop>>,
) {
    for event in events.read() {
        if event.procedure == "XenoIntegration" {
            // Add collective flag
            commands.entity(event.patient).insert(IntegratedCollective);

            // Remove needs
            commands.entity(event.patient).remove::<Needs>();

            // Wipe personality
            if let Ok(mut traits) = query.get_mut(event.patient) {
                traits.0.clear();
            }
        }
    }
}

pub fn evaluate_feed_score(actor: Entity, target: Entity, world: &World) -> f32 {
    let mut base_score = 100.0;

    let actor_is_collective = world.get::<IntegratedCollective>(actor).is_some();
    let target_is_collective = world.get::<IntegratedCollective>(target).is_some();

    if actor_is_collective && !target_is_collective {
        // Massive penalty to helping the "inefficient"
        base_score -= 90.0;
    }

    base_score
}

pub fn score_feeding_action_system() {
    // Stub for the actual Utility AI system integration
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::psychology::needs::Needs;
    use crate::layer1::psychology::traits::{Trait, Traits};
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_xeno_integration_removes_needs_and_traits() {
        let mut app = App::new();
        app.add_event::<SurgeryEvent>();
        app.add_systems(Update, process_integration_surgery_system);

        let mut traits = Traits::default();
        traits.add(Trait::Lazy);
        traits.add(Trait::HardWorker);

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                traits,
                Needs {
                    hunger: 50.0,
                    rest: 50.0,
                    leisure: 50.0,
                    hygiene: 50.0,
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<SurgeryEvent>>()
            .send(SurgeryEvent {
                patient: pop,
                procedure: "XenoIntegration".to_string(),
            });

        app.update();

        assert!(
            app.world().get::<IntegratedCollective>(pop).is_some(),
            "Pop should gain the IntegratedCollective component."
        );
        assert!(
            app.world().get::<Needs>(pop).is_none(),
            "Integrated pops should not have Needs."
        );

        let traits = app.world().get::<Traits>(pop).unwrap();
        assert!(
            traits.0.is_empty(),
            "Integrated pops should lose individual traits."
        );
    }

    #[test]
    fn test_collective_evaluates_unintegrated_pops_as_inefficient() {
        let mut app = App::new();
        app.add_systems(Update, score_feeding_action_system);

        let integrated = app.world_mut().spawn((Pop, IntegratedCollective)).id();
        let unintegrated = app
            .world_mut()
            .spawn((
                Pop,
                Needs {
                    hunger: 0.0,
                    rest: 0.0,
                    leisure: 0.0,
                    hygiene: 0.0,
                },
            ))
            .id();
        let another_integrated = app
            .world_mut()
            .spawn((
                Pop,
                IntegratedCollective,
                Needs {
                    hunger: 0.0,
                    rest: 0.0,
                    leisure: 0.0,
                    hygiene: 0.0,
                },
            ))
            .id();

        // Simulate Utility AI evaluating a "Feed Others" action
        // For unintegrated target
        let score_unintegrated = evaluate_feed_score(integrated, unintegrated, app.world());
        // For integrated target
        let score_integrated = evaluate_feed_score(integrated, another_integrated, app.world());

        assert!(
            score_unintegrated < score_integrated,
            "Integrated pops should heavily penalize helping unintegrated pops."
        );
    }
}
