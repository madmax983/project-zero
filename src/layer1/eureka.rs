use crate::layer1::resources::ColonyResources;
use crate::layer1::tech::{Tech, TechState};
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::utility_ai::ActionType;
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Configuration for Eureka Moments.
#[derive(Resource, Clone)]
pub struct EurekaConfig {
    /// Base probability per tick of triggering a Eureka moment.
    pub base_chance: f64,
    /// Amount of Knowledge granted if Tech is already unlocked.
    pub knowledge_reward: f32,
}

impl Default for EurekaConfig {
    fn default() -> Self {
        Self {
            base_chance: 0.0001, // 0.01% per tick
            knowledge_reward: 10.0,
        }
    }
}

/// Event triggered when a Eureka Moment occurs.
#[derive(Event, Debug)]
pub struct EurekaEvent {
    /// The action that triggered the moment.
    pub action: ActionType,
    /// The tech related to the action (if any).
    pub related_tech: Option<Tech>,
    /// The specific knowledge reward amount (from config).
    pub reward_amount: f32,
}

/// Checks for a Eureka Moment and emits an event if successful.
///
/// Use this when you have access to `EventWriter` (e.g. in a parallel system).
pub fn check_for_eureka(
    events: &mut EventWriter<EurekaEvent>,
    config: &EurekaConfig,
    action: ActionType,
    related_tech: Option<Tech>,
    traits: Option<&Traits>,
) -> bool {
    let mut chance = config.base_chance;

    // Apply Trait Modifiers
    if let Some(t) = traits {
        if t.has(Trait::Intellectual) {
            chance *= 1.2;
        }
        if t.has(Trait::Creative) {
            chance *= 1.5;
        }
    }

    // Simple RNG check
    let mut rng = rand::thread_rng();
    if !rng.gen_bool(chance) {
        return false;
    }

    // Success! Emit Event.
    events.send(EurekaEvent {
        action,
        related_tech,
        reward_amount: config.knowledge_reward,
    });

    true
}

/// Helper to check for Eureka moments when you have exclusive World access.
pub fn check_for_eureka_world(
    world: &mut World,
    action: ActionType,
    related_tech: Option<Tech>,
    traits: Option<Traits>,
) -> bool {
    // 1. Get Config
    let config = world
        .get_resource::<EurekaConfig>()
        .cloned()
        .unwrap_or_default();

    // 2. Calculate Chance
    let mut chance = config.base_chance;
    if let Some(t) = &traits {
        if t.has(Trait::Intellectual) {
            chance *= 1.2;
        }
        if t.has(Trait::Creative) {
            chance *= 1.5;
        }
    }

    // 3. RNG Check
    let mut rng = rand::thread_rng();
    if !rng.gen_bool(chance) {
        return false;
    }

    // 4. Send Event
    world.send_event(EurekaEvent {
        action,
        related_tech,
        reward_amount: config.knowledge_reward,
    });

    true
}

/// System that handles `EurekaEvent`s and applies their effects.
///
/// # Panics
///
/// This function panics if the `unwrap` on `related_tech` fails, but this path
/// is only reachable if `related_tech` is confirmed to be `Some`.
pub fn handle_eureka_events(
    mut events: EventReader<EurekaEvent>,
    mut tech_state: ResMut<TechState>,
    mut resources: ResMut<ColonyResources>,
    mut log: Option<ResMut<MessageLog>>,
) {
    for event in events.read() {
        let mut tech_unlocked = false;
        let mut knowledge_gained = 0.0;

        if let Some(tech) = event.related_tech {
            if tech_state.is_unlocked(tech) {
                knowledge_gained = event.reward_amount;
            } else {
                tech_state.unlock(tech);
                tech_unlocked = true;
            }
        } else {
            knowledge_gained = event.reward_amount;
        }

        if knowledge_gained > 0.0 {
            resources.knowledge += knowledge_gained;
        }

        if let Some(log) = &mut log {
            if tech_unlocked {
                if let Some(tech) = event.related_tech {
                    log.add(format!(
                        "EUREKA! Doing {:?} unlocked {:?}!",
                        event.action,
                        tech.label()
                    ));
                }
            } else {
                log.add(format!(
                    "Eureka! Gained insight ({}) while {:?}!",
                    knowledge_gained, event.action
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::tech::{Tech, TechState};
    use crate::layer1::utility_ai::ActionType;

    #[test]
    fn test_check_for_eureka_emits_event() {
        let mut world = World::new();
        world.insert_resource(EurekaConfig {
            base_chance: 1.0,
            ..Default::default()
        }); // Guaranteed
        world.insert_resource(Events::<EurekaEvent>::default());

        // We need an EventWriter. In tests we can use SystemState or just World.
        // Let's use check_for_eureka_world for simplicity in test setup without SystemState boilerplate
        let occurred =
            check_for_eureka_world(&mut world, ActionType::Work, Some(Tech::Masonry), None);

        assert!(occurred);

        // Verify event
        let events = world.resource::<Events<EurekaEvent>>();
        let mut reader = events.get_cursor();
        let event = reader.read(events).next();
        assert!(event.is_some());
        assert_eq!(event.unwrap().action, ActionType::Work);
    }

    #[test]
    fn test_handle_eureka_events_unlocks_tech() {
        let mut world = World::new();
        let tech_state = TechState {
            total_capacity: 100.0,
            ..Default::default()
        };
        world.insert_resource(tech_state);
        world.insert_resource(ColonyResources::default());
        world.insert_resource(Events::<EurekaEvent>::default());

        // Send event
        world.send_event(EurekaEvent {
            action: ActionType::Work,
            related_tech: Some(Tech::Masonry),
            reward_amount: 10.0,
        });

        // Run handler
        let mut schedule = Schedule::default();
        schedule.add_systems(handle_eureka_events);
        schedule.run(&mut world);

        // Assert
        assert!(world.resource::<TechState>().is_unlocked(Tech::Masonry));
    }

    #[test]
    fn test_trait_bonus() {
        let mut world = World::new();
        // Set chance to 0.5.
        // With Creative (1.5x) -> 0.75.
        // This is hard to test deterministically with RNG.
        // But we can check that it compiles and runs.
        world.insert_resource(EurekaConfig {
            base_chance: 0.5,
            ..Default::default()
        });

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Creative);
            t
        };

        // Just ensure it doesn't crash
        check_for_eureka_world(&mut world, ActionType::Work, None, Some(traits));
    }
}
