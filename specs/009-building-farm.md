# 009: Farm Building and Food Production

## Overview

Farms produce food when worked by pops. Food is a global colony resource. Pops eat food to restore hunger. This completes the survival loop.

## Dependencies

- `006` — Pop needs (hunger need exists)
- `007` — Building placement

## Requirements

### Must Have
- Farm building type
- Farm has worker slots (max 2 workers)
- Workers produce food each tick
- Global food stockpile resource
- Pops automatically eat from stockpile when hungry (< 0.7)
- Visual distinction (yellow rectangle)

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

### Food Production

```rust
const FOOD_PER_WORKER_PER_TICK: f32 = 0.3;

fn produce_food(
    mut tick_events: EventReader<TickEvent>,
    farm_query: Query<&Farm>,
    mut resources: ResMut<ColonyResources>,
) {
    for _ in tick_events.read() {
        for farm in &farm_query {
            let worker_count = farm.workers.len() as f32;
            resources.food += worker_count * FOOD_PER_WORKER_PER_TICK;
        }
    }
}
```

### Food Consumption

```rust
const FOOD_HUNGER_THRESHOLD: f32 = 0.7;  // Eat when below this
const FOOD_PER_MEAL: f32 = 0.1;          // Food consumed per meal
const HUNGER_PER_MEAL: f32 = 0.3;        // Hunger restored per meal

fn consume_food(
    mut tick_events: EventReader<TickEvent>,
    mut resources: ResMut<ColonyResources>,
    mut pop_query: Query<&mut Needs, With<Pop>>,
) {
    for _ in tick_events.read() {
        for mut needs in &mut pop_query {
            if needs.hunger < FOOD_HUNGER_THRESHOLD && resources.food >= FOOD_PER_MEAL {
                resources.food -= FOOD_PER_MEAL;
                needs.hunger = (needs.hunger + HUNGER_PER_MEAL).min(1.0);
            }
        }
    }
}
```

### Spawning Farm

```rust
fn spawn_farm(
    commands: &mut Commands,
    position: GridPosition,
) -> Entity {
    let world_pos = position.to_world();
    commands.spawn((
        Building { building_type: BuildingType::Farm },
        Farm::default(),
        position,
        Sprite {
            color: BuildingType::Farm.color(),
            custom_size: Some(Vec2::splat(TILE_SIZE * 0.9)),
            ..default()
        },
        Transform::from_xyz(world_pos.x, world_pos.y, 0.5),
    )).id()
}
```

### UI: Food Display

Add food count to the UI, near the speed indicator:

```rust
#[derive(Component)]
struct FoodDisplay;

fn update_food_display(
    resources: Res<ColonyResources>,
    mut query: Query<&mut Text, With<FoodDisplay>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        **text = format!("Food: {:.1}", resources.food);
    }
}
```

## Acceptance Criteria

- [ ] Can place Farm via build mode (need to add to building selection)
- [ ] Farm has `Farm` component with capacity 2
- [ ] Farm renders as yellow rectangle
- [ ] `ColonyResources` resource tracks food
- [ ] If pop manually added to `workers`, food increases each tick
- [ ] Pops automatically eat when hunger < 0.7 (if food available)
- [ ] Food display shows current stockpile
- [ ] `cargo check` passes

## Notes

Like housing, actual worker assignment will be job system (spec 010). Builder can manually test by adding pop entity to `workers` vec.

## Questions

*Builder: add questions here if spec is unclear.*
