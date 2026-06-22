use crate::layer1::crafting::CraftEvent;
use crate::layer1::environment::disasters::DisasterEvent;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::{ResourceItem, ResourceType};
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct AlienObservers {
    pub entertainment_score: f32,
    pub threshold: f32,
}

#[derive(Component)]
pub struct DropPod {
    pub resource_amount: f32,
}

pub fn evaluate_colony_entertainment_system(
    mut disaster_events: EventReader<DisasterEvent>,
    mut craft_events: EventReader<CraftEvent>,
    mut observers: ResMut<AlienObservers>,
) {
    // Boredom penalty
    observers.entertainment_score -= 0.1;

    for event in disaster_events.read() {
        // Disasters are very entertaining
        observers.entertainment_score += 50.0 * event.severity;
    }

    for _event in craft_events.read() {
        // Art is mildly entertaining
        observers.entertainment_score += 10.0;
    }
}

pub fn trigger_alien_reward_system(mut commands: Commands, mut observers: ResMut<AlienObservers>) {
    if observers.entertainment_score >= observers.threshold {
        // Reward the colony by spawning a DropPod
        commands.spawn((
            DropPod {
                resource_amount: 500.0,
            },
            GridPosition { x: 50, y: 50 }, // Arbitrary drop location for MVP
            ResourceItem {
                resource_type: ResourceType::Food,
                amount: 500.0,
            },
        ));

        // Reset/reduce score
        observers.entertainment_score -= observers.threshold;
    }

    // Boredom Punishment
    if observers.entertainment_score < 0.0 {
        observers.entertainment_score = 0.0; // clamp to 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::environment::disasters::{DisasterEvent, DisasterType};
    use bevy_app::App;
    use bevy_app::Update;
    #[allow(unused_imports)]
    use bevy_ecs::prelude::*;

    #[test]
    fn test_interesting_events_increase_entertainment_score() {
        let mut app = App::new();
        app.insert_resource(AlienObservers {
            entertainment_score: 0.0,
            threshold: 100.0,
        });
        app.add_event::<DisasterEvent>();
        app.add_event::<CraftEvent>();
        app.add_systems(Update, evaluate_colony_entertainment_system);

        app.world_mut()
            .resource_mut::<Events<DisasterEvent>>()
            .send(DisasterEvent {
                location: crate::layer1::map::GridPosition { x: 0, y: 0 },
                severity: 1.0,
                disaster_type: DisasterType::LocalizedFire,
            });

        app.world_mut()
            .resource_mut::<Events<CraftEvent>>()
            .send(CraftEvent {
                crafter: Entity::from_raw(1),
                item_type: "Statue".to_string(),
            });

        app.update();

        let observers = app.world().resource::<AlienObservers>();
        // -0.1 decay + 50.0 boost
        assert!(
            observers.entertainment_score > 49.0,
            "Disasters should increase the alien entertainment score."
        );
    }

    #[test]
    fn test_high_entertainment_triggers_resource_reward() {
        let mut app = App::new();
        app.insert_resource(AlienObservers {
            entertainment_score: 150.0,
            threshold: 100.0,
        });
        app.add_systems(Update, trigger_alien_reward_system);

        app.update();

        let mut count = 0;
        for _ in app.world_mut().query::<&DropPod>().iter(app.world()) {
            count += 1;
        }

        let observers = app.world().resource::<AlienObservers>();

        assert_eq!(
            count, 1,
            "Hitting the entertainment threshold should spawn a drop pod."
        );
        assert_eq!(
            observers.entertainment_score, 50.0,
            "Score should reduce by the threshold amount after a reward."
        );
    }

    #[test]
    fn test_boredom_penalty_and_clamping() {
        let mut app = App::new();
        app.insert_resource(AlienObservers {
            entertainment_score: 0.0,
            threshold: 100.0,
        });
        app.add_event::<DisasterEvent>();
        app.add_event::<CraftEvent>();
        app.add_systems(
            Update,
            (
                evaluate_colony_entertainment_system,
                trigger_alien_reward_system,
            )
                .chain(),
        );

        app.update();

        let observers = app.world().resource::<AlienObservers>();
        assert_eq!(
            observers.entertainment_score, 0.0,
            "Score should not go below 0"
        );

        app.world_mut()
            .resource_mut::<AlienObservers>()
            .entertainment_score = 0.5;
        app.update();
        let observers = app.world().resource::<AlienObservers>();
        assert!(observers.entertainment_score > 0.3 && observers.entertainment_score < 0.45);
    }
}
