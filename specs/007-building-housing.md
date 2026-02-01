# 007: Housing Building

## Overview

Housing provides shelter and rest for pops. A pop assigned to housing restores their rest need. This is the first building with gameplay function.

## Dependencies

- `005` — Pop needs (rest need exists)
- `006` — Building placement

## Requirements

### Must Have
- Housing component with capacity (max 2 pops)
- Housing has resident slots (list of pop entities)
- Pops in housing restore rest need each tick
- Housing renders as `⌂` (already in 006)
- Info panel shows housing count and total capacity

### Must NOT Have
- Automatic pop assignment (job system does this)
- Construction time
- Upgrades

## Technical Guidance

### Components

```rust
#[derive(Component)]
pub struct Housing {
    pub capacity: usize,
    pub residents: Vec<Entity>,
}

impl Default for Housing {
    fn default() -> Self {
        Self {
            capacity: 2,
            residents: Vec::new(),
        }
    }
}
```

### Update Building Spawn

When placing a Housing building:

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
            // Will be handled in next spec
        }
    }
}
```

### Rest Restoration System

```rust
const REST_RESTORE_PER_TICK: f32 = 0.05;  // Full rest in ~20 ticks

fn restore_rest_in_housing_system(world: &mut World) {
    // Collect housing with residents
    let housing_residents: Vec<Vec<Entity>> = world
        .query::<&Housing>()
        .iter(world)
        .map(|h| h.residents.clone())
        .collect();
    
    // Restore rest for each resident
    for residents in housing_residents {
        for resident in residents {
            if let Some(mut needs) = world.get_mut::<Needs>(resident) {
                needs.rest = (needs.rest + REST_RESTORE_PER_TICK).min(1.0);
            }
        }
    }
}
```

### Clean Up Dead Residents

If a pop dies, remove them from housing:

```rust
fn clean_dead_residents_system(world: &mut World) {
    let mut housing_query = world.query::<&mut Housing>();
    
    for mut housing in housing_query.iter_mut(world) {
        housing.residents.retain(|&entity| world.get_entity(entity).is_some());
    }
}
```

### Info Panel Update

```rust
fn render_info_panel(frame: &mut Frame, area: Rect, world: &World) {
    let pop_count = world.query::<&Pop>().iter(world).count();
    
    let (housing_count, housing_capacity, housing_used): (usize, usize, usize) = {
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
    
    let text = format!(
        "Population: {}\n\n\
         Housing: {}\n\
         Beds: {}/{}\n",
        pop_count,
        housing_count,
        used,
        housing_capacity,
    );
    
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, inner);
}
```

## Acceptance Criteria

- [ ] Placing Housing via build mode adds `Housing` component
- [ ] Housing has capacity of 2
- [ ] If a pop is in `residents`, their rest increases each tick
- [ ] Rest caps at 1.0
- [ ] Dead pops are removed from housing residents
- [ ] Info panel shows housing count and bed usage
- [ ] `cargo check` passes

## Notes

Actual assignment of pops to housing will be handled by the job system (spec 009). For now, a builder could manually add a pop to `residents` in code to verify rest restoration works.

## Questions

*Builder: add questions here if spec is unclear.*
