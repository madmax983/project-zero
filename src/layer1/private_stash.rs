use crate::layer1::resources::{ColonyResources, ResourceType};
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;
use rand::Rng;
use std::collections::HashMap;

/// Component storing resources stolen by a Pop.
#[derive(Component, Debug, Clone, Default)]
pub struct PrivateStash {
    /// The inventory of stolen goods.
    pub inventory: HashMap<ResourceType, f32>,
}

impl PrivateStash {
    /// Adds a resource to the stash.
    pub fn add(&mut self, res: ResourceType, amount: f32) {
        *self.inventory.entry(res).or_insert(0.0) += amount;
    }

    /// Gets the amount of a resource in the stash.
    #[must_use]
    pub fn get(&self, res: ResourceType) -> f32 {
        *self.inventory.get(&res).unwrap_or(&0.0)
    }

    /// Takes all resources from the stash, clearing it.
    pub fn take_all(&mut self) -> HashMap<ResourceType, f32> {
        std::mem::take(&mut self.inventory)
    }
}

/// System that makes Pops with specific traits steal resources.
pub fn hoarding_system(
    mut resources: ResMut<ColonyResources>,
    mut query: Query<(&Traits, &mut PrivateStash)>,
) {
    let mut rng = rand::thread_rng();

    for (traits, mut stash) in &mut query {
        // Small chance to steal per tick (e.g., 0.1%)
        if !rng.gen_bool(0.001) {
            continue;
        }

        if traits.0.contains(&Trait::Anxious) {
            // Steal Food
            let cost = ColonyResources {
                food: 1.0,
                ..ColonyResources::zeroed()
            };
            if resources.try_deduct(&cost) {
                stash.add(ResourceType::Food, 1.0);
            }
        }

        if traits.0.contains(&Trait::Greedy) {
            // Steal Metal
            let cost = ColonyResources {
                metal: 1.0,
                ..ColonyResources::zeroed()
            };
            if resources.try_deduct(&cost) {
                stash.add(ResourceType::Metal, 1.0);
            }
        }
    }
}

/// Inspects a pop and recovers any stolen resources.
pub fn inspect_pop(world: &mut World, pop_entity: Entity) {
    // 1. Get stash content
    let stolen_goods = if let Some(mut stash) = world.get_mut::<PrivateStash>(pop_entity) {
        stash.take_all()
    } else {
        return;
    };

    if stolen_goods.is_empty() {
        return;
    }

    // 2. Return to global resources
    let mut resources = world.resource_mut::<ColonyResources>();
    for (res_type, amount) in stolen_goods {
        match res_type {
            ResourceType::Food => resources.add_food(amount),
            ResourceType::Wood => resources.add_wood(amount),
            ResourceType::Stone => resources.add_stone(amount),
            ResourceType::Planks => resources.add_planks(amount),
            ResourceType::Blocks => resources.add_blocks(amount),
            ResourceType::Ore => resources.add_ore(amount),
            ResourceType::Metal => resources.add_metal(amount),
            ResourceType::Waste => resources.add_waste(amount),
        }
    }

    // 3. Log notification?
    // "Recovered 5 Food from Pop X." - (To be implemented with Notification system)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::{ColonyResources, ResourceType};
    use crate::layer1::traits::{Trait, Traits};
    use std::collections::HashSet;

    #[test]
    fn test_private_stash_component() {
        let mut stash = PrivateStash::default();
        stash.add(ResourceType::Food, 5.0);
        assert!((stash.get(ResourceType::Food) - 5.0).abs() < f32::EPSILON);
        assert!((stash.get(ResourceType::Wood) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_hoarding_system_anxious_steals_food() {
        let mut world = World::new();
        // Setup resources
        let mut resources = ColonyResources::default();
        resources.food = 100.0;
        world.insert_resource(resources);

        // Setup Anxious Pop
        let pop = world
            .spawn((
                Pop,
                Traits(HashSet::from([Trait::Anxious])), // New trait
                PrivateStash::default(),
            ))
            .id();

        // Run system multiple times to ensure probability hits
        // With 0.1% chance, 1000 runs gives ~63% chance of at least one hit.
        // To be safe for tests, we might want to iterate until hit or cap at a high number.
        // Or we can just check if it CAN steal by modifying the rng/probability in test context?
        // But `hoarding_system` uses internal RNG.
        // Let's run enough times. 1000 * 0.001 = 1 expected.
        // Let's run 5000 times.
        for _ in 0..5000 {
            let mut schedule = Schedule::default();
            schedule.add_systems(hoarding_system);
            schedule.run(&mut world);

            // Break early if successful to save time
            let stash = world.get::<PrivateStash>(pop).unwrap();
            if stash.get(ResourceType::Food) > 0.0 {
                break;
            }
        }

        // Check global resources decreased
        let res = world.resource::<ColonyResources>();
        assert!(res.food < 100.0, "Food should be stolen");

        // Check stash increased
        let stash = world.get::<PrivateStash>(pop).unwrap();
        assert!(stash.get(ResourceType::Food) > 0.0, "Stash should contain Food");
    }

    #[test]
    fn test_hoarding_system_greedy_steals_valuables() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.metal = 50.0;
        world.insert_resource(resources);

        let pop = world
            .spawn((
                Pop,
                Traits(HashSet::from([Trait::Greedy])), // New trait
                PrivateStash::default(),
            ))
            .id();

        for _ in 0..5000 {
            let mut schedule = Schedule::default();
            schedule.add_systems(hoarding_system);
            schedule.run(&mut world);

            let stash = world.get::<PrivateStash>(pop).unwrap();
            if stash.get(ResourceType::Metal) > 0.0 {
                break;
            }
        }

        let res = world.resource::<ColonyResources>();
        assert!(res.metal < 50.0, "Metal should be stolen");

        let stash = world.get::<PrivateStash>(pop).unwrap();
        assert!(
            stash.get(ResourceType::Metal) > 0.0,
            "Stash should contain Metal"
        );
    }

    #[test]
    fn test_discovery_returns_resources() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let mut stash = PrivateStash::default();
        stash.inventory.insert(ResourceType::Food, 10.0);

        let pop = world.spawn((Pop, stash, Traits(HashSet::new()))).id();

        // Inspect the pop
        inspect_pop(&mut world, pop);

        // Stash should be empty
        let stash = world.get::<PrivateStash>(pop).unwrap();
        assert!((stash.get(ResourceType::Food) - 0.0).abs() < f32::EPSILON);

        // Global resources should have the food back
        let res = world.resource::<ColonyResources>();
        // Default starts at 10.0 + 10.0 recovered = 20.0
        assert!((res.food - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_inspect_pop_empty_stash() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        let pop = world
            .spawn((Pop, PrivateStash::default(), Traits(HashSet::new())))
            .id();

        inspect_pop(&mut world, pop);

        // Should not panic, resources unchanged
        let res = world.resource::<ColonyResources>();
        assert!((res.food - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_inspect_pop_returns_metal() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let mut stash = PrivateStash::default();
        stash.add(ResourceType::Metal, 5.0);

        let pop = world.spawn((Pop, stash, Traits(HashSet::new()))).id();

        inspect_pop(&mut world, pop);

        let res = world.resource::<ColonyResources>();
        // Default metal is 0.0 + 5.0 = 5.0
        assert!((res.metal - 5.0).abs() < f32::EPSILON);
    }
}
