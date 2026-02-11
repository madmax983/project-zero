# 084: Pop Traits

## Overview

Pops are currently identical clones aside from skills. **Pop Traits** introduce distinct personalities that affect gameplay mechanics. A "Hard Worker" digs faster; a "Glutton" eats more; a "Night Owl" is happier at night. This adds variety and forces players to consider *who* is doing a job, not just *what* the job is.

## Dependencies

- `004` — Pop Entity (for component attachment)
- `005` — Pop Needs (for hunger/rest modifiers)
- `065` — Day/Night Cycle (for time-based traits)
- `066` — Building Work AI (for work speed modifiers)

## RED Phase: Tests First

Write these tests in `src/layer1/traits_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Speed};
    use crate::layer1::needs::Needs;
    use crate::layer1::traits::{Trait, Traits, get_work_speed_modifier, get_hunger_decay_modifier};
    use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
    use std::collections::HashSet;

    #[test]
    fn test_trait_initialization() {
        let mut world = World::new();
        // Assume spawn_initial_pops is updated to add Traits
        crate::layer1::pop::spawn_initial_pops(&mut world);

        // Verify pops have Traits component
        let mut query = world.query::<&Traits>();
        assert!(query.iter(&world).count() > 0);
    }

    #[test]
    fn test_work_speed_modifiers() {
        let mut hard_worker = Traits(HashSet::from([Trait::HardWorker]));
        let mut lazy = Traits(HashSet::from([Trait::Lazy]));
        let normal = Traits(HashSet::new());

        assert!(get_work_speed_modifier(&hard_worker) > 1.0);
        assert!(get_work_speed_modifier(&lazy) < 1.0);
        assert_eq!(get_work_speed_modifier(&normal), 1.0);
    }

    #[test]
    fn test_hunger_decay_modifiers() {
        let mut glutton = Traits(HashSet::from([Trait::Glutton]));
        let mut ascetic = Traits(HashSet::from([Trait::Ascetic]));
        let normal = Traits(HashSet::new());

        // Glutton decays faster (modifier > 1.0)
        assert!(get_hunger_decay_modifier(&glutton) > 1.0);
        // Ascetic decays slower (modifier < 1.0)
        assert!(get_hunger_decay_modifier(&ascetic) < 1.0);
        assert_eq!(get_hunger_decay_modifier(&normal), 1.0);
    }

    #[test]
    fn test_night_owl_mood_modifier() {
        let night_owl = Traits(HashSet::from([Trait::NightOwl]));

        // Night time
        let mood_night = crate::layer1::traits::get_trait_mood_modifier(&night_owl, TimeOfDay::Night);
        assert!(mood_night > 0.0);

        // Day time
        let mood_day = crate::layer1::traits::get_trait_mood_modifier(&night_owl, TimeOfDay::Day);
        assert!(mood_day < 0.0);
    }

    #[test]
    fn test_conflicting_traits() {
        // Ensure mutually exclusive traits don't spawn together (logic in generation, not component)
        // This might be a test for the generation logic.
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let traits = Traits::random(&mut rng);
            let has_lazy = traits.0.contains(&Trait::Lazy);
            let has_hard_worker = traits.0.contains(&Trait::HardWorker);
            assert!(!(has_lazy && has_hard_worker), "Should not be both Lazy and HardWorker");
        }
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Traits (`src/layer1/traits.rs`)

```rust
use bevy_ecs::prelude::*;
use std::collections::HashSet;
use rand::Rng;
use crate::layer1::day_night::TimeOfDay;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trait {
    HardWorker, // +20% Work Speed
    Lazy,       // -20% Work Speed
    Glutton,    // +20% Hunger Decay
    Ascetic,    // -20% Hunger Decay
    NightOwl,   // +Mood at Night, -Mood at Day
    EarlyBird,  // +Mood at Morning, -Mood at Night
    FastWalker, // +10% Move Speed
}

impl Trait {
    pub fn label(&self) -> &'static str {
        match self {
            Self::HardWorker => "Hard Worker",
            Self::Lazy => "Lazy",
            Self::Glutton => "Glutton",
            Self::Ascetic => "Ascetic",
            Self::NightOwl => "Night Owl",
            Self::EarlyBird => "Early Bird",
            Self::FastWalker => "Fast Walker",
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct Traits(pub HashSet<Trait>);

impl Traits {
    pub fn random<R: Rng>(rng: &mut R) -> Self {
        let mut set = HashSet::new();
        // Simple logic: 50% chance to get 1 trait, 20% for 2.
        let count = if rng.gen_bool(0.2) { 2 } else if rng.gen_bool(0.5) { 1 } else { 0 };

        // Pool of all traits
        let pool = [
            Trait::HardWorker, Trait::Lazy,
            Trait::Glutton, Trait::Ascetic,
            Trait::NightOwl, Trait::EarlyBird,
            Trait::FastWalker
        ];

        while set.len() < count {
            let t = pool[rng.gen_range(0..pool.len())];

            // Check conflicts
            if t == Trait::HardWorker && set.contains(&Trait::Lazy) { continue; }
            if t == Trait::Lazy && set.contains(&Trait::HardWorker) { continue; }
            if t == Trait::Glutton && set.contains(&Trait::Ascetic) { continue; }
            if t == Trait::Ascetic && set.contains(&Trait::Glutton) { continue; }
            if t == Trait::NightOwl && set.contains(&Trait::EarlyBird) { continue; }
            if t == Trait::EarlyBird && set.contains(&Trait::NightOwl) { continue; }

            set.insert(t);
        }

        Traits(set)
    }
}

// -- Helper Functions for Systems --

pub fn get_work_speed_modifier(traits: &Traits) -> f32 {
    let mut modifier = 1.0;
    if traits.0.contains(&Trait::HardWorker) { modifier += 0.2; }
    if traits.0.contains(&Trait::Lazy) { modifier -= 0.2; }
    modifier
}

pub fn get_hunger_decay_modifier(traits: &Traits) -> f32 {
    let mut modifier = 1.0;
    if traits.0.contains(&Trait::Glutton) { modifier += 0.2; }
    if traits.0.contains(&Trait::Ascetic) { modifier -= 0.2; }
    modifier
}

pub fn get_move_speed_modifier(traits: &Traits) -> f32 {
    if traits.0.contains(&Trait::FastWalker) { 1.1 } else { 1.0 }
}

pub fn get_trait_mood_modifier(traits: &Traits, time_of_day: TimeOfDay) -> f32 {
    let mut modifier = 0.0;
    if traits.0.contains(&Trait::NightOwl) {
        match time_of_day {
            TimeOfDay::Night => modifier += 0.1,
            TimeOfDay::Day => modifier -= 0.05,
            _ => {}
        }
    }
    if traits.0.contains(&Trait::EarlyBird) {
        match time_of_day {
            TimeOfDay::Dawn | TimeOfDay::Day => modifier += 0.05,
            TimeOfDay::Night => modifier -= 0.1,
            _ => {}
        }
    }
    modifier
}
```

### 2. Integrate with Systems

**Update `spawn_initial_pops` in `src/layer1/pop.rs`**:
- Add `Traits::random(&mut rng)` to the spawn bundle.

**Update `decay_needs_system` in `src/layer1/needs.rs`**:
```rust
// In decay_needs_system
query.par_iter_mut().for_each(|(mut needs, traits_opt)| {
    let trait_mod = traits_opt.map_or(1.0, |t| get_hunger_decay_modifier(t));
    let hunger_decay = BASE_DECAY * policy_mod * trait_mod;
    // ...
});
```
(Note: You'll need to change the query signature to include `Option<&Traits>`)

**Update `evaluate_work` / `execute_work`**:
- Apply `get_work_speed_modifier` to progress calculation.

## REFACTOR Phase: Quality & Design

- **UI**: Traits should be visible in the Inspection window (`015`).
- **Optimization**: Trait modifiers are constant per pop (except mood). We could cache them in a `StatModifiers` component if recalculating every tick becomes expensive. For now, HashSet lookups are fast enough.
- **Conflicts**: The conflict resolution in `random()` is hardcoded. Moving conflict logic to `Trait::conflicts_with(other)` would be cleaner.

## Acceptance Criteria

- [ ] `Traits` component exists.
- [ ] Pops spawn with random, non-conflicting traits.
- [ ] `HardWorker` pops complete jobs faster than normal.
- [ ] `Glutton` pops get hungry faster.
- [ ] `NightOwl` pops have higher mood at night.
- [ ] Tests pass.

## Technical Guidance

- Register `Traits` component in `lib.rs` / `main.rs` if utilizing reflection.
- Be careful with `par_iter_mut` in `decay_needs_system`. Adding a read-only component access (`&Traits`) is fine.
- Ensure `TimeOfDay` is accessible where mood is calculated.
