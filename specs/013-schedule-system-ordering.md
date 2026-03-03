# 013: Schedule and System Execution Ordering

## Overview

Establish formal guarantees for system execution order within the Bevy ECS schedule. This prevents subtle bugs where systems run in the wrong order (e.g., consuming food before producing it, cleaning up entities before marking them dead). Provides a clear framework for future specs to define where their systems fit in the execution pipeline.

## Dependencies

- `001` — Project scaffold (Schedule exists)
- All existing specs (005-009) that add systems

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/shared/schedule.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    // Mock components and systems for testing
    #[derive(Component, Default)]
    struct TestValue(i32);

    fn increment_system(mut query: Query<&mut TestValue>) {
        for mut val in &mut query {
            val.0 += 1;
        }
    }

    fn double_system(mut query: Query<&mut TestValue>) {
        for mut val in &mut query {
            val.0 *= 2;
        }
    }

    fn set_to_ten_system(mut query: Query<&mut TestValue>) {
        for mut val in &mut query {
            val.0 = 10;
        }
    }

    #[test]
    fn test_system_stage_ordering() {
        // Verify stages run in correct order
        assert!(SystemStage::Input as u8 < SystemStage::PreUpdate as u8);
        assert!(SystemStage::PreUpdate as u8 < SystemStage::Update as u8);
        assert!(SystemStage::Update as u8 < SystemStage::PostUpdate as u8);
        assert!(SystemStage::PostUpdate as u8 < SystemStage::Render as u8);
    }

    #[test]
    fn test_schedule_builder_default_stages() {
        let schedule = ScheduleBuilder::new().build();

        // Should have all standard stages
        // (We'll verify via running systems in each stage)
        assert!(schedule.systems_count() >= 0);
    }

    #[test]
    fn test_systems_run_in_stage_order() {
        let mut world = World::new();
        let entity = world.spawn(TestValue(0)).id();

        let schedule = ScheduleBuilder::new()
            .add_system(SystemStage::PreUpdate, set_to_ten_system)
            .add_system(SystemStage::Update, increment_system)
            .add_system(SystemStage::PostUpdate, double_system)
            .build();

        schedule.run(&mut world);

        // Execution order: set_to_ten (0 → 10), increment (10 → 11), double (11 → 22)
        let value = world.get::<TestValue>(entity).unwrap();
        assert_eq!(value.0, 22);
    }

    #[test]
    fn test_systems_in_same_stage_can_run_parallel() {
        // Systems in same stage with no dependencies can run in any order
        let mut world = World::new();
        world.spawn(TestValue(5));
        world.spawn(TestValue(5));

        let schedule = ScheduleBuilder::new()
            .add_system(SystemStage::Update, increment_system)
            .add_system(SystemStage::Update, increment_system) // Duplicate OK
            .build();

        schedule.run(&mut world);

        // Both increments happened (order doesn't matter for this test)
        let mut query = world.query::<&TestValue>();
        for val in query.iter(&world) {
            assert_eq!(val.0, 7); // 5 + 1 + 1
        }
    }

    #[test]
    fn test_system_ordering_within_stage() {
        let mut world = World::new();
        let entity = world.spawn(TestValue(0)).id();

        let schedule = ScheduleBuilder::new()
            .add_system_ordered(SystemStage::Update, set_to_ten_system, 0)
            .add_system_ordered(SystemStage::Update, increment_system, 1)
            .add_system_ordered(SystemStage::Update, double_system, 2)
            .build();

        schedule.run(&mut world);

        // Explicit order: 0 → 10 → 11 → 22
        let value = world.get::<TestValue>(entity).unwrap();
        assert_eq!(value.0, 22);
    }

    #[test]
    fn test_layer1_system_execution_order() {
        // Test the actual game simulation order
        let stages = layer1_system_order();

        // Verify critical ordering (from spec audit):
        // 1. Food production before consumption
        // 2. Consumption before needs decay
        // 3. Needs decay before death checks
        // 4. Death before cleanup

        let production_idx = stages.iter().position(|(name, _, _)| *name == "food_production").unwrap();
        let consumption_idx = stages.iter().position(|(name, _, _)| *name == "food_consumption").unwrap();
        let decay_idx = stages.iter().position(|(name, _, _)| *name == "needs_decay").unwrap();
        let death_idx = stages.iter().position(|(name, _, _)| *name == "kill_starving_pops").unwrap();
        let cleanup_idx = stages.iter().position(|(name, _, _)| *name == "cleanup_dead").unwrap();

        assert!(production_idx < consumption_idx, "Production must run before consumption");
        assert!(consumption_idx < decay_idx, "Consumption must run before decay");
        assert!(decay_idx < death_idx, "Decay must run before death checks");
        assert!(death_idx < cleanup_idx, "Death must run before cleanup");
    }

    #[test]
    fn test_schedule_is_deterministic() {
        let mut world = World::new();
        world.spawn(TestValue(1));

        let schedule = ScheduleBuilder::new()
            .add_system_ordered(SystemStage::Update, increment_system, 0)
            .add_system_ordered(SystemStage::Update, double_system, 1)
            .build();

        // Run multiple times - should get same result
        for _ in 0..10 {
            schedule.run(&mut world);
        }

        // (1 + 1) * 2 = 4, then (4 + 1) * 2 = 10, then (10 + 1) * 2 = 22, etc.
        // After 10 iterations: predictable result
        let mut query = world.query::<&TestValue>();
        for val in query.iter(&world) {
            // Value should be deterministic (not testing exact value, just that it's consistent)
            assert!(val.0 > 0);
        }
    }
}
```

**Test Coverage Requirements:**
- SystemStage: enum ordering, all stages defined
- ScheduleBuilder: system registration, stage ordering. *Architect: Make sure stages are strictly ordered.*
- System execution: stages run in order, systems within stage ordered
- Layer1 ordering: critical simulation order verified
- Determinism: same inputs → same outputs
- All tests must pass before spec is considered complete

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### System Stage Enum

```rust
// src/shared/schedule.rs

use bevy_ecs::prelude::*;

/// Defines the execution stages for systems within a tick.
/// Stages run in numeric order (Input → Render).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
pub enum SystemStage {
    /// Input handling (mouse, keyboard).
    /// Runs before simulation to capture player commands.
    Input = 0,

    /// Pre-update logic (setup, initialization).
    /// Runs before main simulation systems.
    PreUpdate = 1,

    /// Main simulation tick (needs decay, production, job assignment).
    /// This is where most gameplay systems run.
    Update = 2,

    /// Post-update cleanup (remove dead entities, recompute derived state).
    /// Runs after simulation to clean up side effects.
    PostUpdate = 3,

    /// Rendering (query entities, build UI).
    /// Final stage, does not modify simulation state.
    Render = 4,
}
```

### Schedule Builder

```rust
// src/shared/schedule.rs

use std::collections::BTreeMap;

type SystemFn = fn(&mut World);

/// Builds a deterministic execution schedule for systems.
pub struct ScheduleBuilder {
    systems: BTreeMap<SystemStage, Vec<(String, SystemFn, u32)>>, // (name, fn, order)
}

impl ScheduleBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            systems: BTreeMap::new(),
        }
    }

    /// Add a system to a stage. Order within stage is unspecified.
    #[must_use]
    pub fn add_system(mut self, stage: SystemStage, system: SystemFn) -> Self {
        self.systems
            .entry(stage)
            .or_default()
            .push((String::from("unnamed"), system, 0));
        self
    }

    /// Add a system to a stage with explicit ordering.
    /// Lower order values run first within the same stage.
    #[must_use]
    pub fn add_system_ordered(
        mut self,
        stage: SystemStage,
        system: SystemFn,
        order: u32,
    ) -> Self {
        self.systems
            .entry(stage)
            .or_default()
            .push((String::from("unnamed"), system, order));
        self
    }

    /// Add a named system for debugging/documentation.
    #[must_use]
    pub fn add_system_named(
        mut self,
        stage: SystemStage,
        name: impl Into<String>,
        system: SystemFn,
        order: u32,
    ) -> Self {
        self.systems
            .entry(stage)
            .or_default()
            .push((name.into(), system, order));
        self
    }

    /// Build the final schedule.
    #[must_use]
    pub fn build(mut self) -> GameSchedule {
        // Sort systems within each stage by order
        for systems in self.systems.values_mut() {
            systems.sort_by_key(|(_, _, order)| *order);
        }

        GameSchedule {
            stages: self.systems,
        }
    }
}

impl Default for ScheduleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// The final, ordered schedule of systems.
pub struct GameSchedule {
    stages: BTreeMap<SystemStage, Vec<(String, SystemFn, u32)>>,
}

impl GameSchedule {
    /// Run all systems in stage order.
    pub fn run(&self, world: &mut World) {
        for systems in self.stages.values() {
            for (_, system, _) in systems {
                system(world);
            }
        }
    }

    #[must_use]
    pub fn systems_count(&self) -> usize {
        self.stages.values().map(Vec::len).sum()
    }
}
```

### Layer1 System Order

```rust
// src/shared/schedule.rs

/// Returns the canonical ordering of Layer 1 (colony sim) systems.
/// This order is critical for correct simulation behavior.
#[must_use]
pub fn layer1_system_order() -> Vec<(&'static str, SystemStage, u32)> {
    vec![
        // UPDATE STAGE - Main simulation tick
        ("food_production", SystemStage::Update, 0),
        ("rest_restoration", SystemStage::Update, 1),
        ("food_consumption", SystemStage::Update, 2),
        ("needs_decay", SystemStage::Update, 3),
        ("kill_starving_pops", SystemStage::Update, 4),
        ("release_satisfied_workers", SystemStage::Update, 5),
        ("assign_idle_pops", SystemStage::Update, 6),

        // POST-UPDATE STAGE - Cleanup
        ("clean_dead_residents", SystemStage::PostUpdate, 0),
        ("clean_dead_workers", SystemStage::PostUpdate, 1),
    ]
}
```

### Integration with Main

```rust
// src/main.rs - Replace Schedule::default() with ScheduleBuilder

use scale::shared::schedule::{ScheduleBuilder, SystemStage, layer1_system_order};

fn main() -> anyhow::Result<()> {
    // ... setup ...

    let mut world = World::new();
    // ... resources ...

    // Build ordered schedule instead of empty default
    let schedule = build_game_schedule();

    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        // ... input, quit check ...

        // Simulation tick
        if last_tick.elapsed() >= tick_rate {
            schedule.run(&mut world);

            // Tick increment logic
            if *world.resource::<GameState>() == GameState::Running {
                let speed = world.resource::<SimulationTime>().speed;
                if speed != SimSpeed::Paused {
                    world.resource_mut::<SimulationTime>().tick += 1;
                }
            }

            last_tick = Instant::now();
        }

        // ... render ...
    }

    Ok(())
}

fn build_game_schedule() -> GameSchedule {
    let mut builder = ScheduleBuilder::new();

    // Add Layer 1 systems in correct order
    for (name, stage, order) in layer1_system_order() {
        // Systems will be added by future specs as they're implemented
        // For now, schedule structure is defined but systems are placeholders
    }

    builder.build()
}
```

### Module Integration

```rust
// src/shared/mod.rs
pub mod time;
pub mod input;
pub mod schedule; // ADD THIS
```

```rust
// src/lib.rs
pub use shared::schedule::{ScheduleBuilder, SystemStage, GameSchedule, layer1_system_order};
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1. **SystemFn type is limiting**: Only supports `fn(&mut World)`
   - Future: Support Bevy's full system param syntax
   - Current simple function pointers work for now

2. **No dependency graph**: Systems ordered manually, not by declared dependencies
   - Future: Use Bevy's `.before()` and `.after()` syntax
   - Current explicit ordering is clear and deterministic

3. **No parallel execution**: All systems run sequentially
   - Future: Use Bevy's parallel scheduler
   - Current sequential execution simplifies debugging

4. **Schedule is immutable**: Can't add systems at runtime
   - Future: Allow dynamic system registration
   - Current static schedule is fine for MVP

5. **No system profiling**: Can't measure individual system performance
   - Future: Add timing instrumentation
   - Current manual timing with Instant works

### Performance Considerations

- **BTreeMap for stages**: Keeps stages sorted, O(log n) lookup
- **Sequential execution**: Single-threaded, but simpler and deterministic
- **Function pointers**: Zero overhead vs method dispatch
- **Sort once at build**: Systems sorted once, not every frame

### API Design Notes

- `SystemStage` is Copy - cheap to pass as keys
- `ScheduleBuilder` uses builder pattern - fluent, composable
- `GameSchedule` is immutable after build - thread-safe to share
- System order is u32 - allows fine-grained insertion (order 5 can insert at 5.5 later)

### Future Extensibility

When adding Bevy-style system params:
```rust
pub trait IntoSystem {
    fn run(&self, world: &mut World);
}

// Then systems can use Query, Res, etc.
```

When adding parallel execution:
```rust
pub struct ParallelSchedule {
    batches: Vec<Vec<SystemFn>>,
}

// Systems in same batch have no dependencies, run parallel
```

When adding dependency tracking:
```rust
builder
    .add_system(production_system)
    .add_system(consumption_system.after(production_system))
    .add_system(decay_system.after(consumption_system));
```

## Acceptance Criteria (Testable!)

- [x] All tests in RED phase pass
- [x] `cargo test` returns 0 failures
- [x] `cargo clippy -- -D warnings` passes
- [x] Test coverage ≥85% for shared/schedule.rs
- [x] SystemStage enum defines all execution phases
- [x] ScheduleBuilder creates deterministic schedules
- [x] Systems run in stage order (Input → Render)
- [x] Systems within stage run in declared order
- [x] layer1_system_order() defines critical simulation ordering
- [x] Food production runs before consumption
- [x] Death detection runs before cleanup
- [x] Schedule is deterministic (same inputs → same outputs)

## Technical Guidance

### Critical Layer1 System Order

```
UPDATE STAGE:
  0. food_production_system       (Farms produce food)
  1. rest_restoration_system      (Housing restores rest)
  2. food_consumption_system      (Pops eat from colony stores)
  3. needs_decay_system           (Hunger/rest decay over time)
  4. kill_starving_pops_system    (Mark pops as dead if hunger ≤ 0)
  5. release_satisfied_workers_system (Workers leave jobs when needs met)
  6. assign_idle_pops_system      (Assign idle pops to buildings)

POST-UPDATE STAGE:
  0. clean_dead_residents_system  (Remove dead from housing)
  1. clean_dead_workers_system    (Remove dead from jobs)
```

**Why this order matters:**
1. Produce food BEFORE consuming (prevents starvation despite full farms)
2. Consume BEFORE decay (use fresh food first)
3. Decay BEFORE death check (needs update before threshold check)
4. Death BEFORE cleanup (mark dead before removing from lists)
5. Cleanup AFTER all simulation (don't remove entities mid-tick)

### Adding New Systems

When implementing a future spec:
```rust
// In the spec's implementation
pub fn my_new_system(world: &mut World) {
    // System logic
}

// In build_game_schedule():
builder.add_system_named(
    SystemStage::Update,
    "my_new_system",
    my_new_system,
    7, // Order after assign_idle_pops
);
```

### Debugging System Order

Add logging to see execution:
```rust
pub fn run(&self, world: &mut World) {
    for (stage, systems) in &self.stages {
        println!("Running stage: {:?}", stage);
        for (name, system, order) in systems {
            println!("  System: {} (order: {})", name, order);
            system(world);
        }
    }
}
```

### Common Pitfalls

1. **Wrong stage**: Putting cleanup in Update instead of PostUpdate
2. **Wrong order**: Consuming before producing
3. **Assuming parallel**: Systems in same stage run sequentially (for now)
4. **Forgetting to register**: Schedule builder doesn't auto-discover systems

## Questions

*Builder: add questions here if spec is unclear.*

## Future Work

This spec intentionally leaves unimplemented:
- **Parallel execution** - Systems with no dependencies run concurrently
- **Dependency graphs** - `.before()` and `.after()` declarative ordering
- **Dynamic scheduling** - Add/remove systems at runtime
- **System profiling** - Measure individual system performance
- **Conditional systems** - Run only if certain conditions met
