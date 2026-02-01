# 009: Job Assignment System

## Overview

Pops need to be assigned to buildings to work or rest. This system automatically assigns idle pops to available jobs based on their needs. It's the brain that makes the colony self-managing.

## Dependencies

- `004` — Pop entity
- `005` — Pop needs
- `007` — Housing building
- `008` — Farm building

## Requirements

### Must Have
- Pop state: `Idle`, `Working`, `Resting`
- Idle pops get assigned each tick
- Assignment priority: urgent needs first (hunger > rest)
- If pop is starving (hunger < 0.3), prioritize farm work to produce food
- If pop is exhausted (rest < 0.3), prioritize housing for rest
- Otherwise, assign to farm if available
- Pops stay assigned until need is satisfied or slot needed
- When need satisfied (> 0.9), pop returns to Idle
- Pop character shows state (`☺` idle, `☻` working, `⚒` resting)

### Must NOT Have
- Player-controlled assignment (full automation for MVP)
- Job skill preferences
- Pathfinding or travel time

## Technical Guidance

### Components

```rust
#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
pub enum PopState {
    #[default]
    Idle,
    Working(Entity),   // Entity = the farm
    Resting(Entity),   // Entity = the housing
}
```

### Update Pop Spawning

Add PopState to spawned pops:

```rust
world.spawn((
    Pop,
    GridPosition { x, y },
    Needs::default(),
    PopState::default(),
));
```

### Assignment System

```rust
fn assign_jobs_system(world: &mut World) {
    // Collect idle pops with their needs
    let mut idle_pops: Vec<(Entity, f32, f32)> = world
        .query_filtered::<(Entity, &Needs, &PopState), With<Pop>>()
        .iter(world)
        .filter(|(_, _, state)| **state == PopState::Idle)
        .map(|(e, n, _)| (e, n.hunger, n.rest))
        .collect();
    
    // Sort by urgency (lowest need value = most urgent)
    idle_pops.sort_by(|a, b| {
        let a_worst = a.1.min(a.2);
        let b_worst = b.1.min(b.2);
        a_worst.partial_cmp(&b_worst).unwrap()
    });
    
    for (pop_entity, hunger, rest) in idle_pops {
        let needs_rest = rest < 0.3;
        let needs_food = hunger < 0.3;
        
        if needs_rest {
            // Try to assign to housing
            if try_assign_to_housing(world, pop_entity) {
                continue;
            }
        }
        
        // Default or needs_food: try to assign to farm
        if try_assign_to_farm(world, pop_entity) {
            continue;
        }
        
        // If still idle and needs rest badly, try housing even if not critical
        if rest < 0.5 {
            try_assign_to_housing(world, pop_entity);
        }
    }
}

fn try_assign_to_housing(world: &mut World, pop: Entity) -> bool {
    // Find housing with space
    let available: Option<Entity> = world
        .query::<(Entity, &Housing)>()
        .iter(world)
        .find(|(_, h)| h.residents.len() < h.capacity)
        .map(|(e, _)| e);
    
    if let Some(housing_entity) = available {
        // Add to housing
        if let Some(mut housing) = world.get_mut::<Housing>(housing_entity) {
            housing.residents.push(pop);
        }
        // Update pop state
        if let Some(mut state) = world.get_mut::<PopState>(pop) {
            *state = PopState::Resting(housing_entity);
        }
        return true;
    }
    false
}

fn try_assign_to_farm(world: &mut World, pop: Entity) -> bool {
    // Find farm with space
    let available: Option<Entity> = world
        .query::<(Entity, &Farm)>()
        .iter(world)
        .find(|(_, f)| f.workers.len() < f.capacity)
        .map(|(e, _)| e);
    
    if let Some(farm_entity) = available {
        // Add to farm
        if let Some(mut farm) = world.get_mut::<Farm>(farm_entity) {
            farm.workers.push(pop);
        }
        // Update pop state
        if let Some(mut state) = world.get_mut::<PopState>(pop) {
            *state = PopState::Working(farm_entity);
        }
        return true;
    }
    false
}
```

### Release System

```rust
fn release_satisfied_pops_system(world: &mut World) {
    // Collect pops to potentially release
    let check_pops: Vec<(Entity, PopState, f32, f32)> = world
        .query::<(Entity, &PopState, &Needs)>()
        .iter(world)
        .filter(|(_, state, _)| **state != PopState::Idle)
        .map(|(e, s, n)| (e, *s, n.hunger, n.rest))
        .collect();
    
    for (pop_entity, state, hunger, rest) in check_pops {
        let should_release = match state {
            PopState::Resting(housing_entity) => {
                if rest > 0.9 {
                    // Fully rested, release
                    if let Some(mut housing) = world.get_mut::<Housing>(housing_entity) {
                        housing.residents.retain(|&e| e != pop_entity);
                    }
                    true
                } else {
                    false
                }
            }
            PopState::Working(farm_entity) => {
                // Release if needs are critical
                if hunger < 0.2 || rest < 0.2 {
                    if let Some(mut farm) = world.get_mut::<Farm>(farm_entity) {
                        farm.workers.retain(|&e| e != pop_entity);
                    }
                    true
                } else {
                    false
                }
            }
            PopState::Idle => false,
        };
        
        if should_release {
            if let Some(mut state) = world.get_mut::<PopState>(pop_entity) {
                *state = PopState::Idle;
            }
        }
    }
}
```

### Visual Update

Update pop display based on state:

```rust
fn pop_display(needs: &Needs, state: &PopState) -> (char, Color) {
    let health = needs.worst();
    
    let base_color = if health > 0.6 {
        Color::Yellow
    } else if health > 0.3 {
        Color::Rgb(255, 165, 0)  // Orange
    } else {
        Color::Red
    };
    
    let ch = match state {
        PopState::Idle => '☺',
        PopState::Working(_) => '⚒',
        PopState::Resting(_) => '☻',
    };
    
    (ch, base_color)
}
```

### System Execution Order

```rust
// Simulation tick order:
1. release_satisfied_pops_system  // Free up slots first
2. assign_jobs_system             // Fill available slots
3. produce_food_system            // Workers produce food
4. restore_rest_in_housing_system // Resting pops recover
5. consume_food_system            // Everyone eats
6. decay_needs_system             // Needs decrease
7. kill_starving_pops_system      // Remove dead
8. clean_dead_workers_system      // Clean references
9. clean_dead_residents_system
```

## Acceptance Criteria

- [ ] Idle pops automatically get assigned to buildings
- [ ] Starving pops (hunger < 0.3) prioritize farms
- [ ] Exhausted pops (rest < 0.3) prioritize housing
- [ ] Pops release from housing when rested (> 0.9)
- [ ] Pops release from work when needs critical (< 0.2)
- [ ] Pop character changes based on state (☺/⚒/☻)
- [ ] With 1 farm + 1 housing + 5 pops, colony stabilizes
- [ ] `PopState` component reflects current activity
- [ ] `cargo check` passes

## Victory Condition (MVP Complete!)

With all 9 specs implemented:
- Start game, see terrain and pops
- Pops begin dying (no food!)
- Build a farm — pops auto-assign, food appears
- Build housing — tired pops rest
- Colony reaches equilibrium if buildings are sufficient
- Colony dies if buildings are insufficient

**You have a game.**

## Questions

*Builder: add questions here if spec is unclear.*
