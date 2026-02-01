# 008: Farm Building and Food Production

## Overview

Farms produce food when worked by pops. Food is a global colony resource. Pops eat food to restore hunger. This completes the survival loop.

## Dependencies

- `005` — Pop needs (hunger need exists)
- `006` — Building placement

## Requirements

### Must Have
- Farm component with worker slots (max 2 workers)
- Workers produce food each tick
- Global food stockpile resource
- Pops automatically eat from stockpile when hungry (hunger < 0.7)
- Farm renders as `♣` (already in 006)
- Status bar or info panel shows food count

### Must NOT Have
- Crop growth cycles
- Seasonal variation
- Multiple food types

## Technical Guidance

### Resources and Components

```rust
#[derive(Resource, Default)]
pub struct ColonyResources {
    pub food: f32,
}

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

### Update Building Spawn

```rust
fn spawn_building(world: &mut World, pos: GridPosition, building_type: BuildingType) {
    let mut entity = world.spawn((
        Building { building_type },
        pos,
    ));
    
    match building_type {
        BuildingType::Housing => {
            entity.insert(Housing::default());
        }
        BuildingType::Farm => {
            entity.insert(Farm::default());
        }
    }
}
```

### Food Production System

```rust
const FOOD_PER_WORKER_PER_TICK: f32 = 0.3;

fn produce_food_system(world: &mut World) {
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
const FOOD_HUNGER_THRESHOLD: f32 = 0.7;  // Eat when below this
const FOOD_PER_MEAL: f32 = 0.1;          // Food consumed per meal
const HUNGER_PER_MEAL: f32 = 0.3;        // Hunger restored per meal

fn consume_food_system(world: &mut World) {
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

### Clean Up Dead Workers

```rust
fn clean_dead_workers_system(world: &mut World) {
    let mut farm_query = world.query::<&mut Farm>();
    
    for mut farm in farm_query.iter_mut(world) {
        farm.workers.retain(|&entity| world.get_entity(entity).is_some());
    }
}
```

### Info Panel Update

```rust
fn render_info_panel(/* ... */) {
    let resources = world.resource::<ColonyResources>();
    
    let (farm_count, farm_capacity, farm_used): (usize, usize, usize) = {
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

### System Order

These systems should run in order:
1. `produce_food_system` — Generate food from farms
2. `consume_food_system` — Pops eat food
3. `decay_needs_system` — Needs decrease
4. `kill_starving_pops_system` — Remove dead pops
5. `clean_dead_workers_system` — Clean farm references
6. `clean_dead_residents_system` — Clean housing references

## Acceptance Criteria

- [ ] Placing Farm via build mode adds `Farm` component
- [ ] Farm has capacity of 2
- [ ] `ColonyResources` resource tracks food
- [ ] If pop is in `workers`, food increases each tick
- [ ] Pops automatically eat when hunger < 0.7 (if food available)
- [ ] Info panel shows food amount and farm stats
- [ ] Dead pops are removed from farm workers
- [ ] `cargo check` passes

## Notes

Like housing, actual worker assignment will be job system (spec 009). Builder can manually test by adding pop entity to `workers` vec.

## Questions

*Builder: add questions here if spec is unclear.*
