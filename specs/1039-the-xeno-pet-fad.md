# 1039: The Xeno-Pet Fad

## Overview

A cute, seemingly harmless alien creature becomes the must-have companion, until its biology proves incompatible with human logistics. Traders sell these pets which provide a massive, immediate mood boost to their owners. However, they rapidly reproduce when exposed to standard colony food rations and have no natural predators in the colony environment. The pets spread through the residential sectors, maxing out morale. Suddenly, the colony's entire food stockpile vanishes overnight as the population of pets explodes, leading to mass starvation among incredibly happy colonists who refuse to butcher their beloved companions.

## Dependencies

- None

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use scale::layer1::needs::{Morale, Hunger};

    #[test]
    fn test_xeno_pet_increases_morale() {
        let mut app = App::new();
        let entity = app.world_mut().spawn((Morale { value: 50.0, modifiers: vec![] },)).id();

        // Act: Add a xeno pet
        app.world_mut().entity_mut(entity).insert(XenoPetOwner);

        // Run system that applies pet morale
        app.add_systems(Update, apply_xeno_pet_morale);
        app.update();

        let morale = app.world().get::<Morale>(entity).unwrap();
        assert!(morale.value > 50.0, "Morale should increase when owning a Xeno-Pet");
    }

    #[test]
    fn test_xeno_pet_reproduction_consumes_food() {
        let mut app = App::new();
        // Setup food stockpile and a pet
        app.world_mut().insert_resource(FoodStockpile { amount: 100.0 });
        app.world_mut().spawn(XenoPet { hunger: 0.0, reproduction_progress: 90.0 });

        // Run reproduction system
        app.add_systems(Update, xeno_pet_reproduction_system);
        app.update();

        let food = app.world().get_resource::<FoodStockpile>().unwrap();
        assert!(food.amount < 100.0, "Food should be consumed during reproduction");

        // Check if a new pet spawned (should be 2 now)
        let mut query = app.world_mut().query::<&XenoPet>();
        let count = query.iter(app.world()).count();
        assert_eq!(count, 2, "A new Xeno-Pet should have been spawned");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use scale::layer1::needs::Morale;

#[derive(Component)]
pub struct XenoPetOwner;

#[derive(Component)]
pub struct XenoPet {
    pub hunger: f32,
    pub reproduction_progress: f32,
}

#[derive(Resource)]
pub struct FoodStockpile {
    pub amount: f32,
}

pub fn apply_xeno_pet_morale(mut query: Query<&mut Morale, With<XenoPetOwner>>) {
    for mut morale in query.iter_mut() {
        morale.value += 10.0;
    }
}

pub fn xeno_pet_reproduction_system(
    mut commands: Commands,
    mut food: ResMut<FoodStockpile>,
    mut query: Query<&mut XenoPet>
) {
    for mut pet in query.iter_mut() {
        if pet.reproduction_progress >= 90.0 && food.amount >= 10.0 {
            food.amount -= 10.0;
            pet.reproduction_progress = 0.0;
            commands.spawn(XenoPet { hunger: 0.0, reproduction_progress: 0.0 });
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- The morale boost should probably be handled via a continuous modifier or an event rather than directly modifying the value in every tick.
- The `FoodStockpile` might already be represented differently in the `ColonyResources` or similar resource management. Integrate with the actual food system.
- Need a way to represent the colonists refusing to butcher the pets. This could involve adding a specific "Protected" tag or adjusting the butchering AI logic to ignore entities with a `XenoPet` component.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pets consume food and reproduce as expected
- [ ] Pets increase owner morale

## Technical Guidance

- Look at existing systems for resource consumption and morale modifiers to integrate cleanly.
- The refusal to butcher should be implemented by modifying the selection criteria for the butchering task or job.

## Questions
*Builder: add questions here if spec is unclear.*
