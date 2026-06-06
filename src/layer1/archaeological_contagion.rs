use crate::layer1::deep_crust_resonance::ExcavationEvent;
use crate::layer1::mind::utility_types::ActionType;
use crate::layer1::mind::utility_types::PopAction;
use bevy::prelude::*;

#[derive(Component)]
pub struct AncientRoutineInfection {
    pub active: bool,
}

pub fn archaeological_infection_system(
    mut commands: Commands,
    mut events: EventReader<ExcavationEvent>,
) {
    for event in events.read() {
        if event.discovery_type == "AncientRuins" {
            commands
                .entity(event.miner)
                .insert(AncientRoutineInfection { active: true });
        }
    }
}

pub fn ancient_routine_observation_system(
    query: Query<(&AncientRoutineInfection, &PopAction)>,
    mut resources: ResMut<crate::layer1::resources::ColonyResources>,
) {
    for (infection, action) in query.iter() {
        if infection.active && action.current == ActionType::PerformAncientRoutine {
            resources.knowledge += 0.1;
        }
    }
}

pub fn evaluate_ancient_routine(
    query: Query<(Entity, &AncientRoutineInfection)>,
    mut action_query: Query<&mut crate::layer1::mind::utility_types::PopAction>,
) {
    for (entity, infection) in query.iter() {
        if infection.active {
            if let Ok(mut action) = action_query.get_mut(entity) {
                // Occasionally override current actions
                if rand::random::<f32>() < 0.05 {
                    action.current = ActionType::PerformAncientRoutine;
                    action.current_utility = 100.0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::deep_crust_resonance::ExcavationEvent;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::mind::utility_types::ActionType;
    use crate::layer1::mind::utility_types::PopAction;
    use crate::layer1::resources::ColonyResources;

    #[test]
    fn test_excavating_ancient_ruins_infects_miner_with_routines() {
        let mut app = App::new();
        app.add_event::<ExcavationEvent>();
        app.add_systems(Update, archaeological_infection_system);

        let miner = app.world_mut().spawn(Pop).id();

        app.world_mut()
            .resource_mut::<Events<ExcavationEvent>>()
            .send(ExcavationEvent {
                colony: Entity::PLACEHOLDER,
                miner,
                discovery_type: "AncientRuins".to_string(),
                target: Entity::PLACEHOLDER,
            });

        app.update();

        assert!(
            app.world().get::<AncientRoutineInfection>(miner).is_some(),
            "Excavating Ancient Ruins should infect the miner."
        );
    }

    #[test]
    fn test_infected_pop_generates_lost_tech_while_performing_routine() {
        let mut app = App::new();
        app.insert_resource(ColonyResources {
            knowledge: 0.0,
            ..Default::default()
        });
        app.add_systems(Update, ancient_routine_observation_system);

        app.world_mut().spawn((
            Pop,
            AncientRoutineInfection { active: true },
            PopAction {
                current: ActionType::PerformAncientRoutine,
                current_utility: 1.0,
                ticks_committed: 0,
            },
        ));

        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert!(
            resources.knowledge > 0.0,
            "Pops actively performing ancient routines should generate Knowledge points."
        );
    }

    #[test]
    fn test_evaluate_ancient_routine() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_ancient_routine);
        app.world_mut().spawn((
            Pop,
            AncientRoutineInfection { active: true },
            PopAction {
                current: ActionType::SatisfyHunger,
                current_utility: 50.0,
                ticks_committed: 0,
            },
        ));

        // Due to randomness we run it a few times
        for _ in 0..500 {
            app.update();
        }

        let mut changed = false;
        for action in app.world_mut().query::<&PopAction>().iter(app.world()) {
            if action.current == ActionType::PerformAncientRoutine {
                changed = true;
            }
        }
        assert!(
            changed,
            "Ancient routine evaluation should change action over 100 frames with 0.05 chance."
        );
    }
}
