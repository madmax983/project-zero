//! Integration systems that bridge multiple domains in Layer 1.

use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::fire::Fire;
use crate::layer1::health::Health;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::rumor::{Knowledge, Rumor, RumorTopic};
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use rand::seq::SliceRandom;
use std::collections::HashSet;

/// Creates rumors from significant chronicle events.
///
/// Bridges the Chronicle system (History) and Rumor system (Social).
pub fn chronicle_rumor_bridge_system(
    mut events: EventReader<AddChronicleEvent>,
    mut query: Query<(Entity, &mut Knowledge), With<Pop>>,
    time: Res<SimulationTime>,
) {
    // Collect all pops to pick random witnesses
    let pop_entities: Vec<Entity> = query.iter().map(|(e, _)| e).collect();
    if pop_entities.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();

    for event in events.read() {
        if matches!(
            event.importance,
            EventImportance::Major | EventImportance::Legendary
        ) {
            // Create Rumor
            let rumor = Rumor {
                topic: RumorTopic::EventNews(event.text.clone()),
                source: Entity::PLACEHOLDER, // Originated from "The World"
                timestamp: time.tick,
                strength: 1.0,
            };

            // Pick 3 random witnesses (or all if < 3)
            let count = pop_entities.len().min(3);
            let witnesses: Vec<_> = pop_entities
                .choose_multiple(&mut rng, count)
                .copied()
                .collect();

            for witness in witnesses {
                if let Ok((_, mut knowledge)) = query.get_mut(witness) {
                    knowledge.add_rumor(rumor.clone());
                }
            }
        }
    }
}

/// Applies damage to pops standing on fire.
///
/// Bridges the Fire system (Environment) and Pop Health system (Simulation).
pub fn fire_damage_pops_system(
    fire_query: Query<(&GridPosition, &Fire)>,
    mut pop_query: Query<(&GridPosition, &mut Health), With<Pop>>,
) {
    // 1. Identify dangerous tiles
    let fire_tiles: HashSet<GridPosition> = fire_query.iter().map(|(pos, _)| *pos).collect();

    if fire_tiles.is_empty() {
        return;
    }

    // 2. Apply damage to pops on those tiles
    for (pos, mut health) in &mut pop_query {
        if fire_tiles.contains(pos) {
            // Apply 5.0 damage per tick (20 ticks to die)
            let damage = 5.0;
            health.take_damage(damage);
        }
    }
}
