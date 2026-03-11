//! Genetic Memory Feature (Nova Expansion)
//!
//! # The Spark
//! We have a skills system and new pops are born. What if the colony tracks all accumulated
//! experience over time in a collective pool, and newly born pops inherit a small fraction
//! of this "genetic" or "institutional" memory?
//!
//! # The Feature
//! `ColonyGeneticMemory` resource. It passively absorbs a tiny fraction of all `XpGainEvent`s.
//! When a `PopBorn` event fires, the new pop is initialized with a small percentage of
//! the `ColonyGeneticMemory`. This creates a snowball effect where older, more established
//! colonies produce naturally more capable offspring.

use crate::layer1::pop::PopBorn;
use crate::layer1::skills::{SkillType, Skills, XpGainEvent};
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Global tracking of colony-wide accumulated experience.
#[derive(Resource, Default, Clone, Debug)]
pub struct ColonyGeneticMemory {
    pub pool: HashMap<SkillType, f32>,
}

const XP_ABSORPTION_RATE: f32 = 0.01; // 1% of all XP gained goes into the genetic memory
const OFFSPRING_INHERITANCE_RATE: f32 = 0.05; // Offspring inherit 5% of the total genetic memory

/// System to absorb a fraction of all gained XP into the global genetic memory
pub fn absorb_genetic_memory_system(
    mut events: EventReader<XpGainEvent>,
    mut memory: ResMut<ColonyGeneticMemory>,
) {
    for event in events.read() {
        let absorbed_amount = event.amount * XP_ABSORPTION_RATE;
        if absorbed_amount > 0.0 {
            let entry = memory.pool.entry(event.skill).or_insert(0.0);
            *entry += absorbed_amount;
        }
    }
}

/// System to grant a portion of the global genetic memory to newly born pops
pub fn inherit_genetic_memory_system(
    mut events: EventReader<PopBorn>,
    memory: Res<ColonyGeneticMemory>,
    mut skills_query: Query<&mut Skills>,
) {
    if memory.pool.is_empty() {
        return;
    }

    for event in events.read() {
        if let Ok(mut skills) = skills_query.get_mut(event.entity) {
            for (skill_type, total_xp) in &memory.pool {
                let inherited_amount = total_xp * OFFSPRING_INHERITANCE_RATE;
                if inherited_amount > 0.0 {
                    let entry = skills.xp.entry(*skill_type).or_insert(0.0);
                    *entry += inherited_amount;
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((absorb_genetic_memory_system, inherit_genetic_memory_system));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::skills::XpSource;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_absorb_genetic_memory() {
        let mut world = World::new();
        world.init_resource::<ColonyGeneticMemory>();
        world.insert_resource(Events::<XpGainEvent>::default());

        // Fire an XP gain event
        let entity = world.spawn_empty().id();
        world
            .resource_mut::<Events<XpGainEvent>>()
            .send(XpGainEvent {
                entity,
                skill: SkillType::Mining,
                amount: 100.0,
            source: XpSource::Action,
            });

        world.run_system_once(absorb_genetic_memory_system).unwrap();

        let memory = world.resource::<ColonyGeneticMemory>();
        assert_eq!(memory.pool.get(&SkillType::Mining), Some(&1.0)); // 1% of 100
    }

    #[test]
    fn test_inherit_genetic_memory() {
        let mut world = World::new();

        // Setup memory
        let mut memory = ColonyGeneticMemory::default();
        memory.pool.insert(SkillType::Farming, 200.0);
        world.insert_resource(memory);
        world.insert_resource(Events::<PopBorn>::default());

        // Spawn a new pop
        let new_pop = world.spawn(Skills::default()).id();

        world.resource_mut::<Events<PopBorn>>().send(PopBorn {
            entity: new_pop,
            name: "Clone".to_string(),
            tick: 1,
            source: "Vat".to_string(),
        });

        world
            .run_system_once(inherit_genetic_memory_system)
            .unwrap();

        let skills = world.get::<Skills>(new_pop).unwrap();
        assert_eq!(skills.xp.get(&SkillType::Farming), Some(&10.0)); // 5% of 200
    }
}
