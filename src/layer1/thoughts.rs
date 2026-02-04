use crate::layer1::needs::Needs;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use rand::Rng;

/// A component representing the current thought of a Pop.
#[derive(Component, Debug, Clone)]
pub struct Thought {
    /// The text of the thought.
    pub text: String,
    /// The tick when this thought was generated.
    pub tick: u64,
}

/// System to generate thoughts for Pops based on their needs.
pub fn generate_thoughts_system(world: &mut World) {
    let current_tick = world.resource::<SimulationTime>().tick;

    // We can't iterate and spawn/insert at the same time easily in Bevy 0.15 without commands or exclusive world access.
    // Since we have exclusive world access here, we can collect then update.

    let mut to_update = Vec::new();

    let mut query = world.query::<(Entity, &Needs, Option<&Thought>)>();
    for (entity, needs, thought) in query.iter(world) {
        // Update thought every 100 ticks or if missing
        if thought.map_or(true, |t| current_tick > t.tick + 100) {
            let new_thought_text = generate_thought_text(needs);
            to_update.push((entity, new_thought_text));
        }
    }

    for (entity, text) in to_update {
        world.entity_mut(entity).insert(Thought {
            text,
            tick: current_tick,
        });
    }
}

fn generate_thought_text(needs: &Needs) -> String {
    let mut rng = rand::thread_rng();
    let roll = rng.gen_range(0..10);

    if needs.hunger < 0.2 {
        match roll {
            0..=3 => "My stomach hurts...".to_string(),
            4..=7 => "I would kill for a burger.".to_string(),
            _ => "Is that a rock? It looks tasty.".to_string(),
        }
    } else if needs.rest < 0.2 {
        match roll {
            0..=3 => "So... tired...".to_string(),
            4..=7 => "Can't keep my eyes open.".to_string(),
            _ => "Zzz...".to_string(),
        }
    } else if needs.hunger < 0.5 {
        "Could use a snack.".to_string()
    } else if needs.rest < 0.5 {
        "Yawn.".to_string()
    } else {
        match roll {
            0 => "Nice day today.".to_string(),
            1 => "I wonder what's for dinner?".to_string(),
            2 => "Work, work.".to_string(),
            3 => "The colony is looking good.".to_string(),
            4 => "I should build something.".to_string(),
            _ => "Just happy to be here.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_thought_generation() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

        let entity = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.1,
                    rest: 0.8,
                },
            ))
            .id();

        generate_thoughts_system(&mut world);

        let thought = world
            .get::<Thought>(entity)
            .expect("Should have generated thought");
        assert!(
            thought.text.contains("stomach")
                || thought.text.contains("kill")
                || thought.text.contains("rock")
        );
    }

    #[test]
    fn test_thought_update_timing() {
        let mut world = World::new();
        let time = SimulationTime::default();
        world.insert_resource(time);

        let entity = world.spawn((Pop, Needs::default())).id();

        // Initial thought
        generate_thoughts_system(&mut world);
        let initial_tick = world.get::<Thought>(entity).unwrap().tick;
        assert_eq!(initial_tick, 0);

        // Advance time slightly, should not update
        world.resource_mut::<SimulationTime>().tick = 50;
        generate_thoughts_system(&mut world);
        let tick_50 = world.get::<Thought>(entity).unwrap().tick;
        assert_eq!(tick_50, 0); // Still 0

        // Advance time past threshold
        world.resource_mut::<SimulationTime>().tick = 101;
        generate_thoughts_system(&mut world);
        let tick_101 = world.get::<Thought>(entity).unwrap().tick;
        assert_eq!(tick_101, 101);
    }
}
