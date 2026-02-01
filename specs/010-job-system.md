# 010: Job Assignment System

## Overview

Pops need to be assigned to buildings to work or rest. This system automatically assigns idle pops to available jobs based on their needs. It's the brain that makes the colony self-managing.

## Dependencies

- `004` — Pop entity
- `006` — Pop needs
- `008` — Housing building
- `009` — Farm building

## Requirements

### Must Have
- Pop state: `Idle`, `Working`, `Resting`
- Idle pops get assigned each tick
- Assignment priority: urgent needs first (hunger > rest)
- If pop is starving (hunger < 0.3), prioritize farm work to produce food
- If pop is exhausted (rest < 0.3), prioritize housing for rest
- Otherwise, assign to farm if available
- Pops stay assigned until need is satisfied or building is full
- When need satisfied (> 0.9), pop returns to Idle

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
    Working(Entity),   // Entity = the farm/worksite
    Resting(Entity),   // Entity = the housing
}
```

### Assignment Logic

```rust
fn assign_jobs(
    mut tick_events: EventReader<TickEvent>,
    mut pop_query: Query<(Entity, &Needs, &mut PopState), With<Pop>>,
    mut housing_query: Query<(Entity, &mut Housing)>,
    mut farm_query: Query<(Entity, &mut Farm)>,
) {
    for _ in tick_events.read() {
        // Collect idle pops
        let mut idle_pops: Vec<(Entity, Needs)> = pop_query
            .iter()
            .filter(|(_, _, state)| **state == PopState::Idle)
            .map(|(e, n, _)| (e, n.clone()))
            .collect();
        
        // Sort by urgency (lowest need value first = most urgent)
        idle_pops.sort_by(|a, b| {
            a.1.worst().partial_cmp(&b.1.worst()).unwrap()
        });
        
        for (pop_entity, needs) in idle_pops {
            // Determine what this pop needs most
            let needs_rest = needs.rest < 0.3;
            let needs_food = needs.hunger < 0.3;
            
            if needs_rest {
                // Try to assign to housing
                if let Some((housing_entity, mut housing)) = housing_query
                    .iter_mut()
                    .find(|(_, h)| h.residents.len() < h.capacity)
                {
                    housing.residents.push(pop_entity);
                    if let Ok((_, _, mut state)) = pop_query.get_mut(pop_entity) {
                        *state = PopState::Resting(housing_entity);
                    }
                    continue;
                }
            }
            
            // Default or needs_food: assign to farm
            if let Some((farm_entity, mut farm)) = farm_query
                .iter_mut()
                .find(|(_, f)| f.workers.len() < f.capacity)
            {
                farm.workers.push(pop_entity);
                if let Ok((_, _, mut state)) = pop_query.get_mut(pop_entity) {
                    *state = PopState::Working(farm_entity);
                }
            }
        }
    }
}
```

### Release Jobs When Satisfied

```rust
fn release_satisfied_pops(
    mut tick_events: EventReader<TickEvent>,
    mut pop_query: Query<(Entity, &Needs, &mut PopState), With<Pop>>,
    mut housing_query: Query<&mut Housing>,
    mut farm_query: Query<&mut Farm>,
) {
    for _ in tick_events.read() {
        for (pop_entity, needs, mut state) in &mut pop_query {
            match *state {
                PopState::Resting(housing_entity) => {
                    if needs.rest > 0.9 {
                        // Release from housing
                        if let Ok(mut housing) = housing_query.get_mut(housing_entity) {
                            housing.residents.retain(|&e| e != pop_entity);
                        }
                        *state = PopState::Idle;
                    }
                }
                PopState::Working(farm_entity) => {
                    // Release if needs are urgent
                    if needs.hunger < 0.2 || needs.rest < 0.2 {
                        if let Ok(mut farm) = farm_query.get_mut(farm_entity) {
                            farm.workers.retain(|&e| e != pop_entity);
                        }
                        *state = PopState::Idle;
                    }
                }
                PopState::Idle => {}
            }
        }
    }
}
```

### Visual Feedback

Change pop sprite slightly based on state:

```rust
fn update_pop_state_visuals(
    query: Query<(&PopState, &mut Sprite), (With<Pop>, Changed<PopState>)>,
) {
    for (state, mut sprite) in &query {
        // Add subtle tint based on activity
        // Working = slight green tint
        // Resting = slight blue tint
        // Idle = normal
    }
}
```

### System Ordering

```rust
app.add_systems(Update, (
    decay_needs,
    release_satisfied_pops,
    assign_jobs,
    produce_food,
    restore_rest_in_housing,
    consume_food,
    kill_starving_pops,
    update_pop_visuals,
).chain().run_if(in_state(GameState::Playing)));
```

## Acceptance Criteria

- [ ] Idle pops automatically get assigned to buildings
- [ ] Starving pops prioritize farms
- [ ] Exhausted pops prioritize housing
- [ ] Pops release from housing when rested (> 0.9)
- [ ] Pops release from work when needs critical (< 0.2)
- [ ] With 1 farm + 1 housing + 5 pops, colony stabilizes (not everyone dies)
- [ ] PopState component reflects current activity
- [ ] `cargo check` passes

## Victory Condition (MVP Complete!)

With all 10 specs implemented:
- Place a farm and housing
- Watch pops auto-assign
- Colony should reach equilibrium if buildings are sufficient
- Colony dies if buildings are insufficient

## Questions

*Builder: add questions here if spec is unclear.*
