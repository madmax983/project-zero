# 034: Pop Health and Damage

## Overview

Introduces a `Health` component to Pops and standardizes damage handling. Currently, pops die instantly when hunger reaches zero. This feature decouples "Starvation" from "Death", allowing pops to take damage over time from starvation, fire, or accidents.

This lays the foundation for:
- Combat and Defense
- Fire Propagation damage (Spec 033)
- Tool Accidents
- Disease

## Dependencies

- `004` — Pop Entity
- `005` — Pop Needs (for starvation logic)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/health.rs (New file)

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use super::*;
    use crate::layer1::needs::Needs;

    #[test]
    fn test_health_defaults() {
        let health = Health::default();
        assert!((health.current - 100.0).abs() < f32::EPSILON);
        assert!((health.max - 100.0).abs() < f32::EPSILON);
        assert!(health.is_alive());
    }

    #[test]
    fn test_take_damage() {
        let mut health = Health::default();
        health.take_damage(10.0);
        assert!((health.current - 90.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_damage_clamped_at_zero() {
        let mut health = Health::default();
        health.take_damage(200.0);
        assert!((health.current - 0.0).abs() < f32::EPSILON);
        assert!(!health.is_alive());
    }

    #[test]
    fn test_death_check() {
        let mut health = Health::default();
        health.take_damage(100.0);
        assert!(!health.is_alive());
        assert!((health.current - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_starvation_deals_damage_system() {
        let mut world = World::new();
        let entity = world.spawn((
            Health::default(),
            Needs { hunger: 0.0, rest: 0.5, leisure: 0.5 }
        )).id();

        // Run system
        starvation_damage_system(&mut world);

        let health = world.get::<Health>(entity).unwrap();
        assert!(health.current < 100.0);
    }

    #[test]
    fn test_starvation_no_damage_if_fed() {
        let mut world = World::new();
        let entity = world.spawn((
            Health::default(),
            Needs { hunger: 0.1, rest: 0.5, leisure: 0.5 }
        )).id();

        starvation_damage_system(&mut world);

        let health = world.get::<Health>(entity).unwrap();
        assert!((health.current - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_death_system_despawns() {
        use crate::shared::log::MessageLog;
        let mut world = World::new();
        world.insert_resource(MessageLog::default());

        // Dead entity
        let entity = world.spawn(Health { current: -10.0, max: 100.0 }).id();
        // Alive entity
        let survivor = world.spawn(Health::default()).id();

        death_system(&mut world);

        assert!(world.get_entity(entity).is_none());
        assert!(world.get_entity(survivor).is_some());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Health Component

Create `src/layer1/health.rs`.

```rust
// src/layer1/health.rs
use bevy_ecs::prelude::*;
use crate::shared::log::MessageLog;

#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self { current: 100.0, max: 100.0 }
    }
}

impl Health {
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }
}
```

### 2. Implement Systems

In `src/layer1/health.rs`:

```rust
pub fn starvation_damage_system(world: &mut World) {
    let mut query = world.query::<(&crate::layer1::needs::Needs, &mut Health)>();
    for (needs, mut health) in query.iter_mut(world) {
        if needs.hunger <= 0.0 {
             // 1 damage per tick -> 100 ticks to die
             health.take_damage(1.0);
        }
    }
}

pub fn death_system(world: &mut World) {
    // Collect entities to despawn (can't modify world during iteration)
    let to_despawn: Vec<Entity> = world
        .query::<(Entity, &Health)>()
        .iter(world)
        .filter(|(_, h)| !h.is_alive())
        .map(|(e, _)| e)
        .collect();

    for entity in to_despawn {
        world.despawn(entity);
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
             log.add("DEATH: A colonist has died!");
        }
    }
}
```

### 3. Update `Pop` Bundle

Modify `src/layer1/pop.rs` to include `Health::default()` when spawning pops.

### 4. Update Main Schedule

In `src/main.rs`:
- Remove `kill_starving_entities_system`.
- Add `starvation_damage_system` and `death_system`.

## REFACTOR Phase: Quality & Design

- **Delete Old Code**: Ensure `kill_starving_entities_system` is removed from `src/layer1/needs.rs`.
- **UI Integration**: Update `inspect_entity` in `src/shared/selection.rs` to show Health % if the entity has a Health component.
- **Tuning**: Adjust starvation damage rate in `balance.rs` if needed (1.0 per tick is very fast if tick rate is high, considering `decay_needs` is 0.001 per tick). Maybe 0.5 or 0.1?
    - If hunger decays 0.001 per tick, it takes 1000 ticks to starve.
    - If damage is 1.0 per tick, they die in 100 ticks after starving. This gives a "grace period".
- **Visuals**: Future refactor could add a "Hurt" icon or color tint.

## Acceptance Criteria

- [ ] `Health` component exists and defaults to 100.
- [ ] Pops spawn with `Health`.
- [ ] Starving pops (hunger=0) take damage over time.
- [ ] Pops die (despawn) when health <= 0.
- [ ] `kill_starving_entities_system` is deleted.
- [ ] `cargo test` passes.
- [ ] `inspect_entity` shows Health.

## Technical Guidance

- Use `f32::EPSILON` for float comparisons in tests.
- Remember `world.query` vs `world.query_filtered`.
- `Health` should be a `Component`.
- When deleting `kill_starving_entities_system`, check if any tests in `needs.rs` relied on it and update them to use `death_system` or just remove them if redundant.

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
