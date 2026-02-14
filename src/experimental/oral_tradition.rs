//! Experimental module for oral tradition and storytelling.
//!
//! Pops gather in taverns to share stories from the Chronicle, boosting morale.
//! Founders are more likely to tell stories.

use crate::layer1::chronicle::Chronicle;
use crate::layer1::pop::Pop;
use crate::layer1::social::Tavern;
use crate::layer1::social::old_guard::{Generation, MoodModifierEntry, MoodModifiers};
use bevy_ecs::prelude::*;
use rand::Rng;
use rand::seq::SliceRandom;

/// Component for a Pop telling a story.
#[derive(Component, Debug, Clone)]
pub struct Storyteller {
    /// Ticks remaining until the story finishes.
    pub ticks_remaining: u32,
    /// Quality multiplier for the story (1.0 = standard).
    pub quality: f32,
    /// The event being recounted.
    pub event_idx: usize,
}

/// Component for a Pop listening to a story.
#[derive(Component, Debug, Clone)]
pub struct Listener {
    /// The entity ID of the storyteller.
    pub storyteller: Entity,
}

/// System to initiate storytelling in taverns.
pub fn initiate_storytelling_system(
    mut commands: Commands,
    mut tavern_query: Query<&Tavern>,
    pop_query: Query<(Entity, Option<&Generation>), With<Pop>>,
    storyteller_query: Query<&Storyteller>,
    listener_query: Query<&Listener>,
    chronicle: Res<Chronicle>,
) {
    if chronicle.events.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();

    for tavern in &mut tavern_query {
        if tavern.visitors.len() < 3 {
            continue;
        }

        // Check if anyone is already busy telling/listening
        let busy = tavern
            .visitors
            .iter()
            .any(|&e| storyteller_query.contains(e) || listener_query.contains(e));

        if busy {
            continue;
        }

        // Select a storyteller
        // Prioritize Founders: 3x weight
        let mut candidates = Vec::new();
        for &visitor in &tavern.visitors {
            if let Ok((entity, generation)) = pop_query.get(visitor) {
                let weight = match generation {
                    Some(Generation::Founder) => 3,
                    _ => 1,
                };
                for _ in 0..weight {
                    candidates.push(entity);
                }
            }
        }

        if let Some(&storyteller_entity) = candidates.choose(&mut rng) {
            // Pick a random event
            let event_idx = rng.gen_range(0..chronicle.events.len());

            // Calculate quality
            let quality =
                if let Ok((_, Some(Generation::Founder))) = pop_query.get(storyteller_entity) {
                    1.5 // Founders tell better stories
                } else {
                    1.0
                };

            commands.entity(storyteller_entity).insert(Storyteller {
                ticks_remaining: 200, // ~10 seconds at 20 TPS? Or just 200 ticks.
                quality,
                event_idx,
            });

            // Mark others as listeners
            for &visitor in &tavern.visitors {
                if visitor != storyteller_entity {
                    commands.entity(visitor).insert(Listener {
                        storyteller: storyteller_entity,
                    });
                }
            }
        }
    }
}

/// System to process ongoing stories.
pub fn perform_story_system(
    mut commands: Commands,
    mut storytellers: Query<(Entity, &mut Storyteller)>,
    mut listeners: Query<(Entity, &Listener, Option<&mut MoodModifiers>)>,
    chronicle: Res<Chronicle>,
) {
    // Collect finished stories and their details
    let mut finished_stories = std::collections::HashMap::new();

    for (entity, mut storyteller) in &mut storytellers {
        if storyteller.ticks_remaining > 0 {
            storyteller.ticks_remaining -= 1;
        } else {
            // Finished
            if let Some(event) = chronicle.events.get(storyteller.event_idx) {
                finished_stories.insert(entity, (storyteller.quality, event.text.clone()));
            }
            commands.entity(entity).remove::<Storyteller>();
        }
    }

    // Iterate listeners and check if their storyteller finished
    for (listener_entity, listener, mut modifiers_opt) in &mut listeners {
        if let Some((quality, text)) = finished_stories.get(&listener.storyteller) {
            let mood_value = 2.0 * quality;
            let description = format!("Heard a story about: {text}");

            let entry = MoodModifierEntry {
                value: mood_value,
                source: description,
                duration: 500.0,
            };

            if let Some(ref mut mods) = modifiers_opt {
                mods.entries.push(entry);
            } else {
                commands.entity(listener_entity).insert(MoodModifiers {
                    entries: vec![entry],
                });
            }

            commands.entity(listener_entity).remove::<Listener>();
        } else if storytellers.get(listener.storyteller).is_err() {
            // Storyteller died or stopped telling a story unexpectedly
            commands.entity(listener_entity).remove::<Listener>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::chronicle::{Chronicle, EventImportance};
    use crate::layer1::pop::Pop;
    use crate::layer1::social::Tavern;
    use crate::layer1::social::old_guard::Generation;

    #[test]
    fn test_storyteller_selection() {
        let mut world = World::new();
        world.insert_resource(Chronicle::default());
        world.resource_mut::<Chronicle>().add_event(
            0,
            "Test Event".to_string(),
            EventImportance::Standard,
        );

        let mut tavern = Tavern::default();
        let p1 = world.spawn(Pop).id();
        let p2 = world.spawn(Pop).id();
        let p3 = world.spawn((Pop, Generation::Founder)).id();

        tavern.visitors = vec![p1, p2, p3];
        world.spawn(tavern);

        let mut schedule = Schedule::default();
        schedule.add_systems(initiate_storytelling_system);
        schedule.run(&mut world);

        // One should be storyteller, two listeners
        let storytellers = world.query::<&Storyteller>().iter(&world).count();
        let listeners = world.query::<&Listener>().iter(&world).count();

        assert_eq!(storytellers, 1);
        assert_eq!(listeners, 2);
    }

    #[test]
    fn test_story_completion_effect() {
        let mut world = World::new();
        let mut chronicle = Chronicle::default();
        chronicle.add_event(0, "Epic Win".to_string(), EventImportance::Legendary);
        world.insert_resource(chronicle);

        let storyteller = world
            .spawn((
                Pop,
                Storyteller {
                    ticks_remaining: 0,
                    quality: 2.0,
                    event_idx: 0,
                },
            ))
            .id();

        let listener = world.spawn((Pop, Listener { storyteller })).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(perform_story_system);
        schedule.run(&mut world);

        // Listener should have mood modifier
        let modifiers = world.get::<MoodModifiers>(listener).unwrap();
        assert_eq!(modifiers.entries.len(), 1);
        assert!(modifiers.entries[0].source.contains("Epic Win"));
        assert!((modifiers.entries[0].value - 4.0).abs() < f32::EPSILON); // 2.0 * 2.0

        // Components removed
        assert!(world.get::<Storyteller>(storyteller).is_none());
        assert!(world.get::<Listener>(listener).is_none());
    }

    #[test]
    fn test_listener_cleanup_on_storyteller_despawn() {
        use bevy_ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.insert_resource(Chronicle::default());

        // Create a storyteller and then kill them
        let storyteller = world.spawn(Pop).id();
        world.despawn(storyteller);

        // Setup listener pointing to invalid storyteller
        let listener = world.spawn((Pop, Listener { storyteller })).id();

        // Run system once
        world.run_system_once(perform_story_system).unwrap();
        // apply deferred commands
        world.run_system_once(apply_deferred).unwrap();

        assert!(world.get::<Listener>(listener).is_none());
    }
}
