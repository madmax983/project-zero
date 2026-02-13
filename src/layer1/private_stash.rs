//! Private Stash system (Spec 103).
//!
//! Pops with specific traits (`Greedy`, `Anxious`) steal resources from the global stockpile
//! and hide them in their personal inventory.

use crate::layer1::resources::{ColonyResources, ResourceType};
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;
use rand::Rng;
use std::collections::HashMap;

/// Component storing resources hidden by a Pop.
#[derive(Component, Debug, Clone, Default)]
pub struct PrivateStash {
    /// The hidden inventory.
    pub inventory: HashMap<ResourceType, f32>,
}

impl PrivateStash {
    /// Adds resources to the stash.
    pub fn add(&mut self, res: ResourceType, amount: f32) {
        *self.inventory.entry(res).or_insert(0.0) += amount;
    }

    /// Gets the amount of a specific resource in the stash.
    #[must_use]
    pub fn get(&self, res: ResourceType) -> f32 {
        *self.inventory.get(&res).unwrap_or(&0.0)
    }

    /// Takes all resources from the stash, clearing it.
    pub fn take_all(&mut self) -> HashMap<ResourceType, f32> {
        std::mem::take(&mut self.inventory)
    }
}

/// System that allows Pops with hoarding traits to steal resources.
pub fn hoarding_system(
    mut resources: ResMut<ColonyResources>,
    mut query: Query<(&Traits, &mut PrivateStash)>,
) {
    let mut rng = rand::thread_rng();

    for (traits, mut stash) in query.iter_mut() {
        // Small chance to steal per tick (e.g., 1%).
        // 1000 ticks in test gives ~99.99% chance of success.
        if !rng.gen_bool(0.01) {
            continue;
        }

        if traits.0.contains(&Trait::Anxious) {
            // Steal Food
            if resources.food >= 1.0 {
                resources.food -= 1.0;
                stash.add(ResourceType::Food, 1.0);
            }
        }

        if traits.0.contains(&Trait::Greedy) {
            // Steal Metal
            if resources.metal >= 1.0 {
                resources.metal -= 1.0;
                stash.add(ResourceType::Metal, 1.0);
            }
        }
    }
}

/// Inspects a Pop and recovers any stolen goods.
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
            ResourceType::Metal => resources.add_metal(amount),
            ResourceType::Wood => resources.add_wood(amount),
            ResourceType::Stone => resources.add_stone(amount),
            ResourceType::Planks => resources.add_planks(amount),
            ResourceType::Blocks => resources.add_blocks(amount),
            ResourceType::Ore => resources.add_ore(amount),
            ResourceType::Waste => resources.add_waste(amount),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use std::collections::HashSet;

    #[test]
    fn test_private_stash_component() {
        let mut stash = PrivateStash::default();
        stash.add(ResourceType::Food, 5.0);
        assert_eq!(stash.get(ResourceType::Food), 5.0);
        assert_eq!(stash.get(ResourceType::Wood), 0.0);
    }

    #[test]
    fn test_hoarding_system_anxious_steals_food() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.food = 100.0;
        world.insert_resource(resources);

        let pop = world
            .spawn((
                Pop,
                Traits(HashSet::from([Trait::Anxious])),
                PrivateStash::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(hoarding_system);

        for _ in 0..1000 {
            schedule.run(&mut world);
        }

        let res = world.resource::<ColonyResources>();
        assert!(res.food < 100.0, "Food should be stolen");

        let stash = world.get::<PrivateStash>(pop).unwrap();
        assert!(
            stash.get(ResourceType::Food) > 0.0,
            "Stash should contain Food"
        );
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
                Traits(HashSet::from([Trait::Greedy])),
                PrivateStash::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(hoarding_system);

        for _ in 0..1000 {
            schedule.run(&mut world);
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
        let mut resources = ColonyResources::default();
        resources.food = 10.0;
        world.insert_resource(resources);

        let mut stash = PrivateStash::default();
        stash.add(ResourceType::Food, 10.0);

        let pop = world.spawn((Pop, stash, Traits(HashSet::new()))).id();

        inspect_pop(&mut world, pop);

        let stash = world.get::<PrivateStash>(pop).unwrap();
        assert_eq!(stash.get(ResourceType::Food), 0.0);

        let res = world.resource::<ColonyResources>();
        assert_eq!(res.food, 20.0);
    }
}
