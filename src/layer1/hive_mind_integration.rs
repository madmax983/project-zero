use crate::layer1::entities::pop::Pop;
use bevy::prelude::*;

#[derive(Component)]
pub struct IntegratedCollective;

#[derive(Event)]
pub struct SurgeryEvent {
    pub patient: Entity,
    pub procedure: String,
}

#[derive(Component)]
pub struct Sleep {
    pub value: f32,
}

#[derive(Component)]
pub struct Leisure {
    pub value: f32,
}

#[derive(Component)]
pub struct Hunger {
    pub value: f32,
    pub decay_rate: f32,
}

#[derive(Component)]
pub struct TraitList {
    pub traits: Vec<String>,
}

pub fn process_integration_surgery_system(
    mut commands: Commands,
    mut events: EventReader<SurgeryEvent>,
    mut query: Query<&mut TraitList, With<Pop>>,
) {
    for event in events.read() {
        if event.procedure == "XenoIntegration" {
            // Add collective flag
            commands.entity(event.patient).insert(IntegratedCollective);

            // Remove needs
            commands.entity(event.patient).remove::<Sleep>();
            commands.entity(event.patient).remove::<Leisure>();

            // Wipe personality
            if let Ok(mut traits) = query.get_mut(event.patient) {
                traits.traits.clear();
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

    #[test]
    fn test_xeno_integration_removes_needs_and_traits() {
        let mut app = App::new();
        app.add_event::<SurgeryEvent>();
        app.add_systems(Update, process_integration_surgery_system);

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                TraitList {
                    traits: vec!["Lazy".to_string(), "Brave".to_string()],
                },
                Sleep { value: 50.0 },
                Leisure { value: 50.0 },
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
            app.world().get::<Sleep>(pop).is_none(),
            "Integrated pops should not need sleep."
        );
        assert!(
            app.world().get::<Leisure>(pop).is_none(),
            "Integrated pops should not need leisure."
        );
        let traits = app.world().get::<TraitList>(pop).unwrap();
        assert!(
            traits.traits.is_empty(),
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
                Hunger {
                    value: 0.0,
                    decay_rate: 1.0,
                },
            ))
            .id();
        let another_integrated = app
            .world_mut()
            .spawn((
                Pop,
                IntegratedCollective,
                Hunger {
                    value: 0.0,
                    decay_rate: 1.0,
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
