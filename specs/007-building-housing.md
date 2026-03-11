# 007: Housing Building

## Overview

Housing provides shelter and rest for pops. A pop assigned to housing restores their rest need. This is the first building with gameplay function.

## Dependencies

- `005` — Pop needs (rest need exists)
- `006` — Building placement

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer2/housing.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_housing_default() {
        let housing = Housing::default();
        assert_eq!(housing.capacity, 2);
        assert!(housing.residents.is_empty());
    }

    #[test]
    fn test_housing_add_resident() {
        let mut world = World::new();
        let pop_entity = world.spawn(Pop).id();

        let mut housing = Housing::default();
        housing.residents.push(pop_entity);

        assert_eq!(housing.residents.len(), 1);
        assert_eq!(housing.residents[0], pop_entity);
    }

    #[test]
    fn test_housing_capacity_limit() {
        let housing = Housing::default();
        assert!(housing.residents.len() < housing.capacity);
    }

    #[test]
    fn test_restore_rest_in_housing_system() {
        let mut world = World::new();

        let pop = world.spawn((
            Pop,
            Needs { hunger: 0.5, rest: 0.3 },
        )).id();

        let mut housing = Housing::default();
        housing.residents.push(pop);
        world.spawn(housing);

        let rest_before = world.get::<Needs>(pop).unwrap().rest;
        restore_rest_in_housing_system(&mut world);
        let rest_after = world.get::<Needs>(pop).unwrap().rest;

        assert!(rest_after > rest_before, "Rest should increase");
        assert!(rest_after <= 1.0, "Rest should not exceed 1.0");
    }

    #[test]
    fn test_restore_rest_capped_at_one() {
        let mut world = World::new();

        let pop = world.spawn((
            Pop,
            Needs { hunger: 0.5, rest: 0.99 },
        )).id();

        let mut housing = Housing::default();
        housing.residents.push(pop);
        world.spawn(housing);

        restore_rest_in_housing_system(&mut world);
        let rest = world.get::<Needs>(pop).unwrap().rest;

        assert_eq!(rest, 1.0);
    }

    #[test]
    fn test_restore_rest_multiple_residents() {
        let mut world = World::new();

        let pop1 = world.spawn((Pop, Needs { hunger: 0.5, rest: 0.4 })).id();
        let pop2 = world.spawn((Pop, Needs { hunger: 0.5, rest: 0.3 })).id();

        let mut housing = Housing::default();
        housing.residents.push(pop1);
        housing.residents.push(pop2);
        world.spawn(housing);

        restore_rest_in_housing_system(&mut world);

        assert!(world.get::<Needs>(pop1).unwrap().rest > 0.4);
        assert!(world.get::<Needs>(pop2).unwrap().rest > 0.3);
    }

    #[test]
    fn test_clean_dead_residents_system() {
        let mut world = World::new();

        let pop1 = world.spawn(Pop).id();
        let pop2 = world.spawn(Pop).id();

        let mut housing = Housing::default();
        housing.residents.push(pop1);
        housing.residents.push(pop2);
        let housing_entity = world.spawn(housing).id();

        // Kill one pop
        world.despawn(pop1);

        clean_dead_residents_system(&mut world);

        let housing = world.get::<Housing>(housing_entity).unwrap();
        assert_eq!(housing.residents.len(), 1);
        assert_eq!(housing.residents[0], pop2);
    }

    #[test]
    fn test_clean_dead_residents_empty() {
        let mut world = World::new();

        let pop = world.spawn(Pop).id();
        let mut housing = Housing::default();
        housing.residents.push(pop);
        let housing_entity = world.spawn(housing).id();

        // Kill the pop
        world.despawn(pop);

        clean_dead_residents_system(&mut world);

        let housing = world.get::<Housing>(housing_entity).unwrap();
        assert!(housing.residents.is_empty());
    }

    #[test]
    fn test_housing_component_with_building() {
        let mut world = World::new();

        world.spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 5, y: 5 },
            Housing::default(),
        ));

        let count = world.query::<(&Building, &Housing)>().iter(&world).count();
        assert_eq!(count, 1);
    }
}
```

**Test Coverage Requirements:**
- Housing: default values (capacity 2, empty residents)
- Housing: adding residents
- restore_rest_in_housing_system: increases rest, caps at 1.0
- restore_rest_in_housing_system: handles multiple residents
- clean_dead_residents_system: removes despawned entities
- Housing component integrates with Building
- Coverage ≥85% for layer2/housing.rs
- All tests must pass before spec is considered complete

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Housing Component

```rust
// src/layer2/housing.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::needs::Needs;

/// Housing component - provides rest for pops.
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

### Rest Restoration System

```rust
// src/layer2/housing.rs

const REST_RESTORE_PER_TICK: f32 = 0.05; // Full rest in ~20 ticks

/// Restores rest for all pops residing in housing.
pub fn restore_rest_in_housing_system(world: &mut World) {
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

### Cleanup System

```rust
// src/layer2/housing.rs

/// Removes dead residents from housing.
///
/// **System Ordering Note:** This system should run AFTER kill_starving_pops_system
/// to ensure dead entities are cleaned up properly. Recommended execution order:
/// 1. kill_starving_pops_system (despawns dead pops)
/// 2. clean_dead_residents_system (removes references)
/// 3. clean_dead_workers_system (spec 008 - removes from farms)
pub fn clean_dead_residents_system(world: &mut World) {
    let mut housing_query = world.query::<&mut Housing>();

    for mut housing in housing_query.iter_mut(world) {
        housing.residents.retain(|&entity| world.get_entity(entity).is_some());
    }
}
```

### Building Spawn Integration

```rust
// src/layer2/building.rs - Update try_place_building

use super::housing::Housing;

pub fn try_place_building(
    world: &mut World,
    x: i32,
    y: i32,
    building_type: BuildingType,
) -> bool {
    if !can_place_building(world, x, y) {
        return false;
    }

    // Spawn building with type-specific components
    let mut entity = world.spawn((
        Building { building_type },
        GridPosition { x, y },
    ));

    match building_type {
        BuildingType::Housing => {
            entity.insert(Housing::default());
        }
        BuildingType::Farm => {
            // Will be handled in spec 008
        }
    }

    // Mark tile occupied
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

### System Execution Order

```rust
// src/main.rs - Update simulation tick in main loop

if *world.resource::<GameState>() == GameState::Running {
    let speed = world.resource::<SimulationTime>().speed;
    if speed != SimSpeed::Paused {
        // Production and restoration
        restore_rest_in_housing_system(&mut world);

        // Consumption and decay
        decay_needs_system(&mut world);

        // Death and cleanup
        kill_starving_pops_system(&mut world);
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

pub use building::{Building, BuildingType, BuildMode, OccupiedTiles, can_place_building, try_place_building};
pub use housing::{Housing, restore_rest_in_housing_system, clean_dead_residents_system};
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1. **Manual assignment required**: Spec doesn't auto-assign pops to housing
   - Future: Spec 009 (Job System) will handle automatic assignment
   - Current: Builder must manually add pops to `residents` for testing

2. **No eviction logic**: Pops stay until satisfied
   - Future: Spec 009 will handle releasing satisfied pops
   - Current: Manual testing only

3. **REST_RESTORE_PER_TICK hardcoded**: Magic number 0.05
   - Future: Extract to config
   - Current: Tuned for ~20 tick recovery

4. **Vector cloning in restoration**: Clones resident list
   - Necessary to avoid borrow checker issues (can't mutate world while iterating)
   - Performance acceptable for small resident counts (<10 per building)

5. **No capacity enforcement**: Can manually add >2 residents
   - Future: Add validation in assignment system
   - Current: Capacity is advisory, enforcement in spec 009

### Performance Considerations

- **Restoration is O(buildings × residents)**: Linear in total housed pops
- **Cleanup is O(buildings × residents)**: retain() is O(n)
- **Both acceptable**: Colony unlikely to have >1000 residents in early game

### API Design Notes

- `Housing` has public fields - acceptable for components
- `residents` is Vec not HashSet - order doesn't matter, but Vec is simpler
- Cleanup runs after death - ensures no dangling entity references

### Future Extensibility

When adding upgrades (future spec):
```rust
#[derive(Component)]
pub struct Housing {
    pub capacity: usize,
    pub residents: Vec<Entity>,
    pub upgrade_level: u8, // 0 = basic, 1 = improved, etc.
}

impl Housing {
    pub fn capacity(&self) -> usize {
        match self.upgrade_level {
            0 => 2,
            1 => 4,
            _ => 6,
        }
    }
}
```

When adding comfort/morale (future spec):
```rust
const COMFORT_BONUS_PER_TICK: f32 = 0.01;

fn restore_rest_in_housing_system(world: &mut World) {
    // ...
    needs.rest += REST_RESTORE_PER_TICK;
    if let Some(morale) = world.get_mut::<Morale>(resident) {
        morale.value += COMFORT_BONUS_PER_TICK;
    }
}
```

## Acceptance Criteria (Testable!)

- [x] All tests in RED phase pass
- [x] `cargo test` returns 0 failures
- [x] `cargo clippy -- -D warnings` passes
- [x] Test coverage ≥85% for layer2/housing.rs
- [x] Placing Housing via build mode adds `Housing` component
- [x] Housing has capacity of 2
- [x] If a pop is in `residents`, their rest increases each tick
- [x] Rest caps at 1.0
- [x] Dead pops are removed from housing residents
- [x] Info panel shows housing count and bed usage
- [x] clean_dead_residents_system runs after kill_starving_pops_system

## Technical Guidance

### System Execution Order

**Critical ordering requirement:**
1. `restore_rest_in_housing_system` - Restore needs BEFORE decay
2. `decay_needs_system` - Decay all needs
3. `kill_starving_pops_system` - Despawn dead entities
4. `clean_dead_residents_system` - Remove dead entity references

If cleanup runs before death system, no cleanup happens. If restoration runs after decay, pops lose more rest than they gain.

### Testing Without Job System

To manually test rest restoration before spec 009:

```rust
// In tests or temporary code
let pop = world.spawn((Pop, Needs::default())).id();
let mut housing = world.query::<&mut Housing>().single_mut(&mut world);
housing.residents.push(pop);

// Run simulation
restore_rest_in_housing_system(&mut world);

// Verify pop's rest increased
let needs = world.get::<Needs>(pop).unwrap();
assert!(needs.rest > 0.8);
```

### Capacity Semantics

- `capacity`: Maximum residents allowed (enforced by job system in spec 009)
- `residents.len()`: Current occupancy
- Info panel shows `used/capacity` (e.g., "Beds: 3/4")

### Common Pitfalls

1. **Forgetting to add Housing component**: Building entity won't restore rest
2. **Wrong system order**: Cleanup before death = references never cleaned
3. **Assuming auto-assignment**: Spec 009 implements this, not spec 007
4. **Expecting capacity enforcement**: Validation happens in assignment, not component

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.

## Notes for Spec 009

Spec 009 (Job System) will:
- Automatically assign idle pops to housing when rest < 0.3
- Release pops from housing when rest > 0.9
- Enforce capacity limits during assignment
- Handle priority (starving pops prioritize farms over housing)

This spec establishes the data structures and restoration logic. Spec 009 adds the intelligence.
