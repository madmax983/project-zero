# 703 Xeno-Gastronomy

## 1. Overview
Eating survival paste is efficient, but eating the glowing purple moss is an adventure. The "Chef" job allows Pops to experiment with alien flora and fauna. These culinary experiments yield randomized effects—from massive morale buffs and improved work speed to sudden poisonings or hallucinogenic addiction. Over time, successful recipes are recorded in the colony's "Cookbook," providing permanent bonuses.

## 2. Dependencies
- Layer 1 `Job` and `Workplace` mechanics.
- `Morale`, `Health`, and `Trait` systems on Pops.
- A `Flora`/`Fauna` resource or component on the map.
- Knowledge/Tech system to store the "Cookbook".

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_chef_experiments_and_discovers_recipe() {
        // Arrange
        let mut app = App::new();
        app.world_mut().insert_resource(Cookbook::default());
        let chef = app.world_mut().spawn((
            Pop { ..default() },
            Job::Chef,
            Inventory { resources: vec![Resource::XenoMoss] }
        )).id();

        // Act
        // Simulate a completed cooking cycle
        let mut events = app.world_mut().resource_mut::<Events<CookingCompletedEvent>>();
        events.send(CookingCompletedEvent { chef, ingredient: Resource::XenoMoss });

        app.add_systems(Update, resolve_xeno_gastronomy);
        app.update();

        // Assert
        let cookbook = app.world().get_resource::<Cookbook>().unwrap();
        assert!(cookbook.known_recipes.contains_key(&Resource::XenoMoss), "Chef should discover a recipe for the new ingredient");
    }

    #[test]
    fn test_eating_experimental_food_applies_effect() {
        // Arrange
        let mut app = App::new();
        let pop = app.world_mut().spawn((
            Pop { ..default() },
            Morale { value: 50.0 },
            Health { value: 100.0 }
        )).id();

        let meal = Meal { effect: FoodEffect::Hallucination };

        // Act
        let mut events = app.world_mut().resource_mut::<Events<ConsumeMealEvent>>();
        events.send(ConsumeMealEvent { consumer: pop, meal });

        app.add_systems(Update, process_digestion);
        app.update();

        // Assert
        let pop_tags = app.world().get::<Hallucinating>(pop);
        assert!(pop_tags.is_some(), "Pop should receive the hallucinatory effect from the meal");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Resource, Default)]
pub struct Cookbook {
    pub known_recipes: HashMap<Resource, FoodEffect>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FoodEffect {
    Nourishing,
    Poisonous,
    Hallucination,
}

pub struct CookingCompletedEvent {
    pub chef: Entity,
    pub ingredient: Resource,
}

#[derive(Clone)]
pub struct Meal {
    pub effect: FoodEffect,
}

pub struct ConsumeMealEvent {
    pub consumer: Entity,
    pub meal: Meal,
}

#[derive(Component)]
pub struct Hallucinating;

pub fn resolve_xeno_gastronomy(
    mut events: EventReader<CookingCompletedEvent>,
    mut cookbook: ResMut<Cookbook>,
) {
    for event in events.read() {
        // Randomly assign an effect if unknown (simplified to static for pass)
        cookbook.known_recipes.entry(event.ingredient.clone()).or_insert(FoodEffect::Hallucination);
    }
}

pub fn process_digestion(
    mut commands: Commands,
    mut events: EventReader<ConsumeMealEvent>,
) {
    for event in events.read() {
        if event.meal.effect == FoodEffect::Hallucination {
            commands.entity(event.consumer).insert(Hallucinating);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**: The random generation of `FoodEffect` should use a deterministic RNG seeded by the world's seed, so the same ingredient on the same planet type yields the same result in subsequent playthroughs if desired, or keep it per-run.
- **Code Smells**: Using `Resource` as a hashmap key might be brittle if resources are highly variant. Consider a specific `IngredientType`.
- **Performance Considerations**: Fast and event-driven.
- **API Improvements**: When a recipe is discovered, fire a `RecipeDiscoveredEvent` to notify the UI and log the discovery in the Chronicle.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/gastronomy.rs`.
- [ ] Processing an unknown xeno-ingredient adds it to the `Cookbook` with a generated effect.
- [ ] Consuming the produced meal applies the correct buff, debuff, or tag to the consuming Pop.

## 7. Technical Guidance
- **Code Structure Suggestions**: Create `src/layer1/gastronomy.rs` for the cooking and digestion mechanics.
- **Integration Points**: Tie the `CookingCompletedEvent` to the end of a work cycle at a Kitchen/Lab workstation.
- **Gotchas**: Ensure there is a UI to display the `Cookbook` so players know what they are feeding their colonists. Pops should prioritize `Nourishing` food but might be forced to eat `Poisonous` food during a famine.

## 8. Questions
*Builder: add questions here if spec is unclear.*
