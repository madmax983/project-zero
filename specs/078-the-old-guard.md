# 078: The Old Guard

## Overview

The colony is defined by its history. **The Old Guard** introduces a system where Pops track their arrival date. This creates a natural social hierarchy:

- **Founders** (Tick 0): The original survivors. They have high authority but are resistant to change.
- **Old Guard** (Early Arrivals): Respect the Founders, distrust New Blood. (For MVP, this group is merged with Founders).
- **New Blood** (Recent Arrivals): Bring new skills but cause friction.

This feature adds:
- `Arrival` component tracking `tick` and `generation`.
- `Opinion` penalties between generations (Old vs New).
- `Mood` buffs/debuffs ("Founder's Pride", "Newcomer Anxiety").
- Authority bonuses for Founders (higher Social impact).

## Dependencies

- `004` — Pop Entity (Base entity)
- `047` — Pop Relationships (Social Affinity)
- `031` — Pop Morale (Mood Buffs)
- `010` — Chronicle (Time tracking)

## RED Phase: Tests First

Write these tests in `src/layer1/old_guard_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::social::{Relationships, AffinityChange, modify_affinity_system};
    use crate::layer1::old_guard::{Arrival, Generation, update_generational_friction_system, FounderBuff};
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_arrival_component_defaults() {
        // Default arrival should be roughly "now" if not specified,
        // or explicitly constructed.
        let arrival = Arrival::new(100, Generation::Immigrant);
        assert_eq!(arrival.tick, 100);
        assert_eq!(arrival.generation, Generation::Immigrant);
    }

    #[test]
    fn test_founder_generation_check() {
        // Founders are tick 0
        let arrival = Arrival::new(0, Generation::Founder);
        assert_eq!(arrival.generation, Generation::Founder);
    }

    #[test]
    fn test_generational_friction_system() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 5000, ..Default::default() });
        world.init_resource::<Events<AffinityChange>>();

        // Founder (Old)
        let founder = world.spawn((
            Pop,
            Arrival::new(0, Generation::Founder),
            FounderBuff, // Manually added for unit test (or run apply_founder_benefits_system first)
            Relationships::default(),
        )).id();

        // Newcomer (New)
        let newcomer = world.spawn((
            Pop,
            Arrival::new(4900, Generation::Immigrant), // Just arrived
            Relationships::default(),
        )).id();

        // Run system that applies friction
        let mut schedule = Schedule::default();
        schedule.add_systems((update_generational_friction_system, modify_affinity_system));
        schedule.run(&mut world);

        // Check Founder opinion of Newcomer
        let founder_rel = world.get::<Relationships>(founder).unwrap();
        let opinion = founder_rel.get_affinity(newcomer);

        // Should be negative (e.g., -5.0)
        assert!(opinion < 0.0);
    }

    #[test]
    fn test_founder_morale_buff() {
        let mut world = World::new();
        let founder = world.spawn((
            Pop,
            Arrival::new(0, Generation::Founder),
        )).id();

        // System should apply a "FounderBuff" component or modify Needs/Mood directly
        // For MVP, we apply a component that the morale system reads
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::old_guard::apply_founder_benefits_system);
        schedule.run(&mut world);

        assert!(world.get::<FounderBuff>(founder).is_some());
    }

    #[test]
    fn test_new_blood_decay() {
        // "New Blood" status should fade over time?
        // Or Generation is permanent, but "Newcomer" mood/friction applies only for X ticks.
        // Let's say friction applies if (current_tick - arrival_tick) < 1 year (1000 ticks).

        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 2000, ..Default::default() });
        world.init_resource::<Events<AffinityChange>>();

        let old_guy = world.spawn((Pop, Arrival::new(0, Generation::Founder), Relationships::default())).id();

        // Arrived 1500 ticks ago (longer than 1000)
        let integrated_guy = world.spawn((
            Pop,
            Arrival::new(500, Generation::Immigrant),
            Relationships::default()
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_generational_friction_system);
        schedule.run(&mut world);

        // Should be NO friction generated this tick
        // (We can't easily check "no event sent" without mocking,
        // so we check if affinity remains 0 after modify_affinity_system)

        let mut schedule2 = Schedule::default();
        schedule2.add_systems(modify_affinity_system);
        schedule2.run(&mut world);

        let rel = world.get::<Relationships>(old_guy).unwrap();
        assert_eq!(rel.get_affinity(integrated_guy), 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `Arrival` and `Generation`

In `src/layer1/old_guard.rs`:

```rust
use bevy_ecs::prelude::*;
use crate::layer1::balance::TICKS_PER_YEAR;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Generation {
    #[default]
    Immigrant,
    Founder,
    NativeBorn,
}

#[derive(Component, Debug, Clone, Default)]
pub struct Arrival {
    pub tick: u64,
    pub generation: Generation,
}

impl Arrival {
    pub fn new(tick: u64, generation: Generation) -> Self {
        Self { tick, generation }
    }
}

#[derive(Component)]
pub struct FounderBuff; // Marker for mood boost
```

### 2. Implement Systems

```rust
use crate::shared::time::SimulationTime;
use crate::layer1::social::AffinityChange;
use crate::layer1::pop::Pop;

pub fn apply_founder_benefits_system(
    mut commands: Commands,
    query: Query<(Entity, &Arrival), Without<FounderBuff>>,
) {
    for (entity, arrival) in query.iter() {
        if arrival.generation == Generation::Founder {
            commands.entity(entity).insert(FounderBuff);
        }
    }
}

pub fn update_generational_friction_system(
    time: Res<SimulationTime>,
    founders: Query<(Entity, &Arrival), With<FounderBuff>>,
    newcomers: Query<(Entity, &Arrival)>,
    mut events: EventWriter<AffinityChange>,
) {
    // Only run occasionally (e.g. once per day) to avoid spamming affinity changes
    if time.tick % 100 != 0 { return; }

    const NEWCOMER_PERIOD: u64 = TICKS_PER_YEAR; // 1000 ticks

    for (founder_entity, _) in founders.iter() {
        for (newcomer_entity, newcomer_arrival) in newcomers.iter() {
            if founder_entity == newcomer_entity { continue; }

            // Check if "New Blood" (arrived recently)
            if time.tick > newcomer_arrival.tick
               && (time.tick - newcomer_arrival.tick) < NEWCOMER_PERIOD
            {
                // Friction!
                events.send(AffinityChange {
                    source: founder_entity,
                    target: newcomer_entity,
                    amount: -1.0, // Slow degradation of opinion
                });
            }
        }
    }
}
```

### 3. Update Pop Spawning

In `src/layer1/pop.rs`, update `spawn_initial_pops`:
- Add `.insert(Arrival::new(0, Generation::Founder))` to the spawn bundle.

## REFACTOR Phase: Quality & Design

- **Optimization**: The O(N*M) loop in `update_generational_friction_system` is fine for small colonies but should be optimized later (e.g., query only `Newcomers`).
- **Mood Integration**: `FounderBuff` needs to be read by `needs.rs` or `morale_system` to actually apply the +5 mood.
- **Native Born**: `Generation::NativeBorn` should be set when a child is born (Spec 062).
- **Social Impact**: Founders should have a multiplier on their `Socialize` action effectiveness.

## Acceptance Criteria

- [ ] `Arrival` component exists.
- [ ] Initial Pops spawn as `Generation::Founder`.
- [ ] Founders get `FounderBuff`.
- [ ] Founders apply negative affinity to recent arrivals (New Blood).
- [ ] Friction stops after `NEWCOMER_PERIOD`.
- [ ] Tests pass.

## Technical Guidance

- Ensure `update_generational_friction_system` runs *before* `modify_affinity_system`.
- Remember to import `Arrival` in `pop.rs`.
