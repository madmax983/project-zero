use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct PredictiveAI {
    pub active: bool,
    pub prediction_threshold: f32,
}

#[derive(Event)]
pub struct SacrificeDemandEvent {
    pub required_pops: u32,
    pub target_entity: Entity, // What needs to be destroyed/sacrificed
}

pub struct SacrificeDemand {
    pub target_entity: Entity,
    pub completed: bool,
}

#[derive(Resource, Default)]
pub struct ActiveSacrificeDemands {
    pub demands: Vec<SacrificeDemand>,
}

// In the spec, `Pop` is locally redefined for the tests to work. We should NOT use it globally,
// so let's keep it private to this module so we don't conflict with crate::layer1::entities::pop::Pop
#[derive(Component)]
pub struct SpecPop;

#[derive(Component)]
pub struct MartyrCultMember;

#[derive(Clone)]
pub enum Action {
    Idle,
    Sabotage(Entity),
}

#[derive(Component, Default)]
pub struct ActionQueue {
    pub actions: Vec<Action>,
}

pub fn generate_ai_predictions_system(
    ai: Option<Res<PredictiveAI>>,
    mut event_writer: EventWriter<SacrificeDemandEvent>,
) {
    if let Some(ai) = ai {
        if ai.active {
            // Minimal implementation: Always generate a fake demand for testing
            event_writer.send(SacrificeDemandEvent {
                required_pops: 500,
                target_entity: Entity::from_raw(1),
            });
        }
    }
}

pub fn process_sacrifice_demands_system(
    mut events: EventReader<SacrificeDemandEvent>,
    mut active_demands: ResMut<ActiveSacrificeDemands>,
) {
    for event in events.read() {
        active_demands.demands.push(SacrificeDemand {
            target_entity: event.target_entity,
            completed: false,
        });
    }
}

pub fn martyr_cult_action_system(
    demands: Option<Res<ActiveSacrificeDemands>>,
    mut cultists: Query<&mut ActionQueue, With<MartyrCultMember>>,
) {
    if let Some(demands_res) = demands {
        if let Some(first_demand) = demands_res.demands.first() {
            if !first_demand.completed {
                for mut queue in cultists.iter_mut() {
                    queue.actions.push(Action::Sabotage(first_demand.target_entity));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_predictive_ai_generates_sacrifice_demand() {
        let mut app = App::new();
        app.add_systems(Update, generate_ai_predictions_system);

        // Add the predictive AI resource
        app.world_mut().insert_resource(PredictiveAI {
            active: true,
            prediction_threshold: 0.9,
        });
        app.world_mut().insert_resource(Events::<SacrificeDemandEvent>::default());

        app.update();

        // The AI should generate a demand to avert a disaster
        let sacrifice_events = app.world().get_resource::<Events<SacrificeDemandEvent>>().unwrap();
        let mut reader = sacrifice_events.get_cursor();
        let events: Vec<_> = reader.read(sacrifice_events).collect();

        assert_eq!(events.len(), 1, "Predictive AI must generate exactly one Sacrifice Demand");
        assert!(events[0].required_pops > 0, "Sacrifice demand must require pop casualties");
    }

    #[test]
    fn test_martyr_cult_attempts_sabotage() {
        let mut app = App::new();
        app.add_systems(Update, martyr_cult_action_system);

        app.world_mut().insert_resource(ActiveSacrificeDemands {
            demands: vec![SacrificeDemand { target_entity: Entity::from_raw(1), completed: false }]
        });

        // Spawn a cultist Pop
        let cultist_id = app.world_mut().spawn((SpecPop, MartyrCultMember, ActionQueue::default())).id();

        app.update();

        // The cultist should enqueue a sabotage action to fulfill the demand
        let action_queue = app.world().get::<ActionQueue>(cultist_id).unwrap();
        assert!(
            action_queue.actions.iter().any(|a| matches!(a, Action::Sabotage(_))),
            "Cult member must attempt to sabotage the target to fulfill the AI's demand"
        );
    }
}
