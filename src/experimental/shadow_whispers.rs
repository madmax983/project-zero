#![allow(clippy::type_complexity)]
//! Shadow Whispers (Nova Feature).
//!
//! # The Spark
//! We penalize speed and morale directly when Pops are in the dark (`LightMap < 0.1`). What if darkness bred actual social contagion?
//!
//! # The Feature
//! A system that gives Pops standing in the dark a chance to hallucinate and generate a "Doom Prophecy" rumor.
//! They then spread this paranoia through the existing Rumor system, meaning one dark corner can infect the colony's morale.

use crate::layer1::entities::pop::Pop;
use crate::layer1::lighting::LightMap;
use crate::layer1::map::GridPosition;
use crate::layer1::rumor::{Knowledge, Rumor, RumorTopic};
use bevy_ecs::prelude::*;
use rand::Rng;

/// System that generates rumors when Pops are in the dark.
pub fn shadow_whispers_system(
    light_map: Option<Res<LightMap>>,
    time: Option<Res<crate::shared::time::SimulationTime>>,
    mut pops: Query<(Entity, &GridPosition, &mut Knowledge), With<Pop>>,
) {
    let Some(light_map) = light_map else { return };
    let tick = time.map(|t| t.tick).unwrap_or(0);
    let mut rng = rand::thread_rng();

    for (entity, pos, mut knowledge) in &mut pops {
        let x = u32::try_from(pos.x).unwrap_or(0);
        let y = u32::try_from(pos.y).unwrap_or(0);

        let light_level = light_map.get(x, y);

        // If in pitch darkness, 1% chance per tick to generate paranoia
        if light_level < 0.1 && rng.gen_bool(0.01) {
            // Ensure they don't just spam the same doom prophecy infinitely if they already know it
            if !knowledge.knows(&RumorTopic::DoomProphecy) {
                knowledge.add_rumor(Rumor {
                    topic: RumorTopic::DoomProphecy,
                    source: entity,
                    timestamp: tick,
                    strength: 1.0,
                });
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((shadow_whispers_system,));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_shadow_whispers_generates_rumor_in_darkness() {
        let mut world = World::new();

        let mut light_map = LightMap::new(10, 10);
        light_map.tiles.fill(0.0); // Pitch black
        world.insert_resource(light_map);

        let pop = world
            .spawn((Pop, GridPosition { x: 5, y: 5 }, Knowledge::default()))
            .id();

        // Since it's a 1% chance, we'll run it enough times to practically guarantee it triggers in a test
        // Or we can just mock RNG, but running it a bunch is easier for a simple test.
        for _ in 0..1000 {
            world.run_system_once(shadow_whispers_system).unwrap();
        }

        let knowledge = world.get::<Knowledge>(pop).unwrap();
        assert!(knowledge.knows(&RumorTopic::DoomProphecy));
    }

    #[test]
    fn test_shadow_whispers_does_not_generate_in_light() {
        let mut world = World::new();

        let mut light_map = LightMap::new(10, 10);
        light_map.tiles.fill(1.0); // Fully lit
        world.insert_resource(light_map);

        let pop = world
            .spawn((Pop, GridPosition { x: 5, y: 5 }, Knowledge::default()))
            .id();

        for _ in 0..1000 {
            world.run_system_once(shadow_whispers_system).unwrap();
        }

        let knowledge = world.get::<Knowledge>(pop).unwrap();
        assert!(!knowledge.knows(&RumorTopic::DoomProphecy));
    }
}
