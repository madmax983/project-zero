# 008: Farm Building and Food Production

## Overview

Farms produce food when worked by pops. Food is a global colony resource. Pops eat food to restore hunger. This completes the survival loop.

## Dependencies

- `005` — Pop needs (hunger need exists)
- `006` — Building placement

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer2/farm.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_farm_default() {
        let farm = Farm::default();
        assert_eq!(farm.capacity, 2);
        assert!(farm.workers.is_empty());
    }

    #[test]
    fn test_colony_resources_default() {
        let resources = ColonyResources::default();
        assert_eq!(resources.food, 0.0);
    }

    #[test]
    fn test_produce_food_system() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let worker = world.spawn(Pop).id();
        let mut farm = Farm::default();
        farm.workers.push(worker);
        world.spawn(farm);

        produce_food_system(&mut world);

        let resources = world.resource::<ColonyResources>();
        assert!(resources.food > 0.0, "Food should be produced");
    }

    #[test]
    fn test_produce_food_multiple_workers() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let worker1 = world.spawn(Pop).id();
        let worker2 = world.spawn(Pop).id();
        let mut farm = Farm::default();
        farm.workers.push(worker1);
        farm.workers.push(worker2);
        world.spawn(farm);

        produce_food_system(&mut world);

        let resources = world.resource::<ColonyResources>();
        assert!(resources.food >= 0.6, "Two workers should produce more food");
    }

    #[test]
    fn test_produce_food_multiple_ticks() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let worker = world.spawn(Pop).id();
        let mut farm = Farm::default();
        farm.workers.push(worker);
        world.spawn(farm);

        for _ in 0..10 {
            produce_food_system(&mut world);
        }

        let resources = world.resource::<ColonyResources>();
        assert!(resources.food >= 3.0, "10 ticks should accumulate food");
    }

    #[test]
    fn test_consume_food_system() {
        let mut world = World::new();
        world.insert_resource(ColonyResources { food: 1.0 });

        world.spawn((
            Pop,
            Needs { hunger: 0.5, rest: 0.8 },
        ));

        let food_before = world.resource::<ColonyResources>().food;
        consume_food_system(&mut world);
        let food_after = world.resource::<ColonyResources>().food;

        assert!(food_after < food_before, "Food should be consumed");
    }

    #[test]
    fn test_consume_food_restores_hunger() {
        let mut world = World::new();
        world.insert_resource(ColonyResources { food: 1.0 });

        let pop = world.spawn((
            Pop,
            Needs { hunger: 0.5, rest: 0.8 },
        )).id();

        consume_food_system(&mut world);

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.hunger > 0.5, "Hunger should increase");
    }

    #[test]
    fn test_consume_food_only_when_hungry() {
        let mut world = World::new();
        world.insert_resource(ColonyResources { food: 1.0 });

        world.spawn((
            Pop,
            Needs { hunger: 0.9, rest: 0.8 },
        ));

        let food_before = world.resource::<ColonyResources>().food;
        consume_food_system(&mut world);
        let food_after = world.resource::<ColonyResources>().food;

        assert_eq!(food_after, food_before, "High hunger pop should not eat");
    }

    #[test]
    fn test_consume_food_stops_when_depleted() {
        let mut world = World::new();
        world.insert_resource(ColonyResources { food: 0.05 }); // Less than meal cost

        world.spawn((Pop, Needs { hunger: 0.5, rest: 0.8 }));
        world.spawn((Pop, Needs { hunger: 0.4, rest: 0.8 }));

        consume_food_system(&mut world);

        // At most one pop should eat
        let resources = world.resource::<ColonyResources>();
        assert!(resources.food >= 0.0, "Food should not go negative");
    }

    #[test]
    fn test_clean_dead_workers_system() {
        let mut world = World::new();

        let worker1 = world.spawn(Pop).id();
        let worker2 = world.spawn(Pop).id();

        let mut farm = Farm::default();
        farm.workers.push(worker1);
        farm.workers.push(worker2);
        let farm_entity = world.spawn(farm).id();

        world.despawn(worker1);

        clean_dead_workers_system(&mut world);

        let farm = world.get::<Farm>(farm_entity).unwrap();
        assert_eq!(farm.workers.len(), 1);
        assert_eq!(farm.workers[0], worker2);
    }

    #[test]
    fn test_farm_component_with_building() {
        let mut world = World::new();

        world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 5, y: 5 },
            Farm::default(),
        ));

        let count = world.query::<(&Building, &Farm)>().iter(&world).count();
        assert_eq!(count, 1);
    }
}
```

**Test Coverage Requirements:**
- Farm: default values
- ColonyResources: default, accumulation
- produce_food_system: single worker, multiple workers, accumulation
- consume_food_system: consumption, hunger restoration, threshold check, depletion
- clean_dead_workers_system: removes despawned entities
- Coverage ≥85% for layer2/farm.rs
- All tests must pass before spec is considered complete

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Components and Resources

```rust
// src/layer2/farm.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::needs::Needs;

/// Colony-wide resources.
#[derive(Resource, Default)]
pub struct ColonyResources {
    pub food: f32,
}

/// Farm component - produces food when worked.
#[derive(Component)]
pub struct Farm {
    pub capacity: usize,
    pub workers: Vec<Entity>,
}

impl Default for Farm {
    fn default() -> Self {
        Self {
            capacity: 2,
            workers: Vec::new(),
        }
    }
}
```

### Food Production System

```rust
// src/layer2/farm.rs

const FOOD_PER_WORKER_PER_TICK: f32 = 0.3;

/// Produces food from all farms with workers.
pub fn produce_food_system(world: &mut World) {
    let mut total_production = 0.0;

    for farm in world.query::<&Farm>().iter(world) {
        // Only count workers that still exist
        let valid_workers = farm.workers.iter()
            .filter(|&&e| world.get_entity(e).is_some())
            .count();
        total_production += valid_workers as f32 * FOOD_PER_WORKER_PER_TICK;
    }

    world.resource_mut::<ColonyResources>().food += total_production;
}
```

### Food Consumption System

```rust
// src/layer2/farm.rs

const FOOD_HUNGER_THRESHOLD: f32 = 0.7;  // Eat when below this
const FOOD_PER_MEAL: f32 = 0.1;          // Food consumed per meal
const HUNGER_PER_MEAL: f32 = 0.3;        // Hunger restored per meal

/// Pops eat food when hungry.
pub fn consume_food_system(world: &mut World) {
    let mut food = world.resource::<ColonyResources>().food;

    // Collect hungry pops
    let hungry_pops: Vec<Entity> = world
        .query_filtered::<(Entity, &Needs), With<Pop>>()
        .iter(world)
        .filter(|(_, needs)| needs.hunger < FOOD_HUNGER_THRESHOLD)
        .map(|(e, _)| e)
        .collect();

    for entity in hungry_pops {
        if food >= FOOD_PER_MEAL {
            if let Some(mut needs) = world.get_mut::<Needs>(entity) {
                food -= FOOD_PER_MEAL;
                needs.hunger = (needs.hunger + HUNGER_PER_MEAL).min(1.0);
            }
        }
    }

    world.resource_mut::<ColonyResources>().food = food;
}
```

### Cleanup System

```rust
// src/layer2/farm.rs

/// Removes dead workers from farms.
///
/// **System Ordering Note:** This system should run AFTER kill_starving_pops_system.
/// See spec 007 for cleanup ordering details.
pub fn clean_dead_workers_system(world: &mut World) {
    let mut farm_query = world.query::<&mut Farm>();

    for mut farm in farm_query.iter_mut(world) {
        farm.workers.retain(|&entity| world.get_entity(entity).is_some());
    }
}
```

### Building Spawn Integration

```rust
// src/layer2/building.rs - Update try_place_building

use super::farm::Farm;

pub fn try_place_building(
    world: &mut World,
    x: i32,
    y: i32,
    building_type: BuildingType,
) -> bool {
    if !can_place_building(world, x, y) {
        return false;
    }

    let mut entity = world.spawn((
        Building { building_type },
        GridPosition { x, y },
    ));

    match building_type {
        BuildingType::Housing => {
            entity.insert(Housing::default());
        }
        BuildingType::Farm => {
            entity.insert(Farm::default());
        }
    }

    world.resource_mut::<OccupiedTiles>().0.insert((x, y));

    true
}
```

### Info Panel Update

```rust
// src/main.rs - Update render_info_panel

fn render_info_panel(frame: &mut Frame, area: Rect, world: &World) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Info ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let pop_count = world.query::<&Pop>().iter(world).count();
    let resources = world.resource::<ColonyResources>();

    let (housing_count, housing_capacity, housing_used) = {
        let mut count = 0;
        let mut capacity = 0;
        let mut used = 0;
        for housing in world.query::<&Housing>().iter(world) {
            count += 1;
            capacity += housing.capacity;
            used += housing.residents.len();
        }
        (count, capacity, used)
    };

    let (farm_count, farm_capacity, farm_used) = {
        let mut count = 0;
        let mut capacity = 0;
        let mut used = 0;
        for farm in world.query::<&Farm>().iter(world) {
            count += 1;
            capacity += farm.capacity;
            used += farm.workers.len();
        }
        (count, capacity, used)
    };

    let text = format!(
        "Population: {}\n\n\
         Food: {:.1}\n\n\
         Housing: {}\n\
         Beds: {}/{}\n\n\
         Farms: {}\n\
         Workers: {}/{}",
        pop_count,
        resources.food,
        housing_count,
        housing_used,
        housing_capacity,
        farm_count,
        farm_used,
        farm_capacity,
    );

    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, inner);
}
```

### System Execution Order

```rust
// src/main.rs - Update simulation tick

if *world.resource::<GameState>() == GameState::Running {
    let speed = world.resource::<SimulationTime>().speed;
    if speed != SimSpeed::Paused {
        // Production and restoration (BEFORE decay)
        produce_food_system(&mut world);
        restore_rest_in_housing_system(&mut world);

        // Consumption
        consume_food_system(&mut world);

        // Decay
        decay_needs_system(&mut world);

        // Death and cleanup
        kill_starving_pops_system(&mut world);
        clean_dead_workers_system(&mut world);
        clean_dead_residents_system(&mut world);

        world.resource_mut::<SimulationTime>().tick += 1;
    }
}
```

### Module Integration

```rust
// src/layer2/mod.rs
pub mod building;
pub mod housing;
pub mod farm;

pub use building::{Building, BuildingType, BuildMode, OccupiedTiles, can_place_building, try_place_building};
pub use housing::{Housing, restore_rest_in_housing_system, clean_dead_residents_system};
pub use farm::{Farm, ColonyResources, produce_food_system, consume_food_system, clean_dead_workers_system};
```

```rust
// src/main.rs - Add to main()

world.insert_resource(ColonyResources::default());
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1. **Manual worker assignment**: Like spec 007, requires manual testing
   - Spec 009 will implement automatic job assignment
   - Current: Builder must manually add pops to `workers` vector

2. **Constants hardcoded**: Production/consumption rates are magic numbers
   - Future: Extract to config or balance file
   - Current: Tuned for survival equilibrium with 5 pops, 1 farm

3. **No food spoilage**: Food accumulates indefinitely
   - Future: Add decay or storage limits
   - Current: Infinite stockpile acceptable for MVP

4. **Consumption is first-come-first-served**: No priority
   - Future: Prioritize critical hunger levels
   - Current: Simple iteration order

5. **No production time**: Food appears instantly
   - Future: Add growth cycles
   - Current: Immediate production simpler for testing

### Performance Considerations

- **Production is O(farms × workers)**: Linear
- **Consumption is O(hungry_pops)**: Linear subset
- **Both acceptable**: Early game <100 entities total

### System Execution Order (Critical!)

**Correct order:**
1. `produce_food_system` - Generate food
2. `restore_rest_in_housing_system` - Restore rest
3. `consume_food_system` - Eat food (BEFORE decay!)
4. `decay_needs_system` - Decay all needs
5. `kill_starving_pops_system` - Despawn dead
6. `clean_dead_workers_system` - Remove references
7. `clean_dead_residents_system` - Remove references

**Why this order:**
- Production before consumption: Food available to eat
- Consumption before decay: Pops eat before losing hunger
- Cleanup after death: No dangling references

## Acceptance Criteria (Testable!)

- [x] All tests in RED phase pass
- [x] `cargo test` returns 0 failures
- [x] `cargo clippy -- -D warnings` passes
- [x] Test coverage ≥85% for layer2/farm.rs
- [x] Placing Farm via build mode adds `Farm` component
- [x] Farm has capacity of 2
- [x] `ColonyResources` resource tracks food
- [x] If pop is in `workers`, food increases each tick
- [x] Pops automatically eat when hunger < 0.7 (if food available)
- [x] Info panel shows food amount and farm stats
- [x] Dead pops are removed from farm workers
- [x] Systems run in correct order

## Technical Guidance

### Survival Equilibrium Math

With 5 pops:
- Hunger decay: 5 × 0.02 = 0.1 hunger/tick total
- To feed 5 pops: need 0.1 / 0.3 = 0.33 meals/tick
- Meals cost: 0.33 × 0.1 = 0.033 food/tick
- Production needed: 0.033 / 0.3 = 0.11 workers
- **Result:** 1 worker can feed ~9 pops

This is very generous. Intentional for MVP - focus is on mechanics, not difficulty.

### Food Threshold (0.7)

Pops eat when hunger < 0.7, consuming 0.1 food to restore 0.3 hunger.
- Starting hunger: 0.8
- After 5 ticks decay: 0.7 (eat!)
- After eating: 1.0 (full)
- After 15 more ticks: 0.7 (eat again)

Eating frequency: ~15-20 ticks at equilibrium.

### Common Pitfalls

1. **Wrong system order**: Decay before consumption = pops starve despite food
2. **Forgetting ColonyResources**: Food production has nowhere to go
3. **Negative food**: Consumption doesn't check for going negative (it does - early return)
4. **Assuming auto-assignment**: Spec 009 implements this

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
