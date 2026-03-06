# 353: Alien Domestication

## 1. Overview

This isn't just a mine; it's a ranch. Learning to live with the local biology, not just exploit it.

**Alien Domestication** allows Pops with specific skills to tame wild fauna. Tamed beasts provide passive bonuses (mood boosts), generate resources over time (like milk or wool), or can be assigned to simple labor tasks (like hauling). However, they require food upkeep and can revert to instinct under stress, potentially causing havoc inside the colony.

## 2. Dependencies

- `164` Modular Fauna (for the base animal entities)
- `075` Animal Husbandry (for foundational pen/pasture mechanics)
- `016` Utility AI System (for taming action and animal behavior)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::fauna::{FaunaBundle, Tameable, Tamed, FeralInstinct};
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Needs;

    #[test]
    fn test_taming_action_converts_wild_fauna() {
        let mut world = World::new();

        let animal = world.spawn((
            FaunaBundle::default(),
            Tameable { required_skill: 10, progress: 0.0 },
        )).id();

        let pop = world.spawn(Pop).id();

        world.spawn(TameAction {
            target: animal,
            actor: pop,
            taming_power: 15.0,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(execute_tame_action_system);
        schedule.run(&mut world);

        // Assuming 15.0 power pushes progress over 100% threshold
        let is_tamed = world.get::<Tamed>(animal).is_some();
        assert!(is_tamed);
        assert!(world.get::<Tameable>(animal).is_err()); // Removes Tameable
    }

    #[test]
    fn test_tamed_animals_consume_food() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(tamed_animal_metabolism_system);

        let animal = world.spawn((
            Tamed { owner: None },
            Needs { hunger: 100.0, ..Default::default() },
        )).id();

        schedule.run(&mut world);

        let needs = world.get::<Needs>(animal).unwrap();
        assert!(needs.hunger < 100.0); // Hunger decreases (gets hungrier)
    }

    #[test]
    fn test_starving_tamed_animals_revert_to_feral() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(feral_reversion_system);

        let animal = world.spawn((
            Tamed { owner: None },
            FeralInstinct { threshold: 20.0 }, // Reverts if hunger < 20
            Needs { hunger: 10.0, ..Default::default() }, // Starving
        )).id();

        schedule.run(&mut world);

        assert!(world.get::<Tamed>(animal).is_err());
        assert!(world.get::<Tameable>(animal).is_ok()); // Can be re-tamed
    }

    #[test]
    fn test_tamed_animals_produce_resources() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(animal_production_system);

        let animal = world.spawn((
            Tamed { owner: None },
            ResourceProducer { resource_type: ResourceType::Wool, progress: 99.0, max: 100.0 },
        )).id();

        // Assume system processes progress and spawns item when >= max
        schedule.run(&mut world);

        let producer = world.get::<ResourceProducer>(animal).unwrap();
        assert_eq!(producer.progress, 0.0); // Reset

        // Verify resource entity was spawned
        let resources = world.query::<&ResourceItem>().iter(&world).count();
        assert_eq!(resources, 1);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::needs::Needs;
use crate::layer1::resources::{ResourceItem, ResourceType};

#[derive(Component)]
pub struct Tameable {
    pub required_skill: u32,
    pub progress: f32,
}

#[derive(Component)]
pub struct Tamed {
    pub owner: Option<Entity>,
}

#[derive(Component)]
pub struct FeralInstinct {
    pub threshold: f32,
}

#[derive(Component)]
pub struct ResourceProducer {
    pub resource_type: ResourceType,
    pub progress: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct TameAction {
    pub target: Entity,
    pub actor: Entity,
    pub taming_power: f32,
}

pub fn execute_tame_action_system(
    mut commands: Commands,
    mut actions: Query<(Entity, &TameAction)>,
    mut targets: Query<&mut Tameable>,
) {
    for (action_entity, action) in actions.iter_mut() {
        if let Ok(mut tameable) = targets.get_mut(action.target) {
            tameable.progress += action.taming_power;
            if tameable.progress >= 100.0 {
                commands.entity(action.target)
                    .remove::<Tameable>()
                    .insert(Tamed { owner: Some(action.actor) });
            }
        }
        commands.entity(action_entity).despawn();
    }
}

pub fn tamed_animal_metabolism_system(
    mut query: Query<&mut Needs, With<Tamed>>,
) {
    for mut needs in query.iter_mut() {
        needs.hunger -= 0.5; // Arbitrary decay rate
    }
}

pub fn feral_reversion_system(
    mut commands: Commands,
    query: Query<(Entity, &Needs, &FeralInstinct), With<Tamed>>,
) {
    for (entity, needs, instinct) in query.iter() {
        if needs.hunger < instinct.threshold {
            commands.entity(entity)
                .remove::<Tamed>()
                .insert(Tameable { required_skill: 10, progress: 0.0 });
        }
    }
}

pub fn animal_production_system(
    mut commands: Commands,
    mut query: Query<(&mut ResourceProducer, &Tamed)>,
) {
    for (mut producer, _) in query.iter_mut() {
        producer.progress += 1.0;
        if producer.progress >= producer.max {
            producer.progress = 0.0;
            commands.spawn(ResourceItem { resource_type: producer.resource_type.clone(), amount: 1 });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Utility AI Integration:** `Tamed` animals should hook into the Utility AI to either pathfind to a designated "Pasture" zone or follow their `owner`.
- **Bonding:** If a `Tamed` animal has an `owner`, their proximity should grant a positive mood buff to the owner. If the animal dies, apply a massive `Grief` penalty to the owner.
- **Resource Drops:** `animal_production_system` should ideally drop the resource at the animal's current `Position` rather than floating in the void.
- **Feral Reversion:** When reverting to feral, if the animal has carnivorous or territorial tags from `Modular Fauna`, they should become immediately hostile to nearby Pops, creating the "tamed predator reverts during food shortage" emergence.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code
- [ ] Pops can convert `Tameable` entities to `Tamed`.
- [ ] Tamed animals consume food/needs.
- [ ] Tamed animals revert to feral (`Tameable`) when their needs fall below a threshold.
- [ ] Animals with `ResourceProducer` generate items over time when tamed.

## 7. Technical Guidance

- Ensure `ResourceType` matches the implementations in `src/layer1/resources.rs` (e.g., `Milk`, `Wool`, `Slime`).
- For the `TameAction`, you will need to add an `ActionType::Tame` to the `Utility AI` scoring system so that Pops with high Animal Handling skill prioritize taming wild beasts in designated zones.

## 8. Questions

*Builder: add questions here if spec is unclear.*
