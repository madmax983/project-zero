# 569 The Syzygy

## 1. Overview
This feature introduces "The Syzygy", a rare planetary alignment event occurring on Layer 2 that dramatically alters physics and biology on Layer 1. For a short period, gravitational forces combine to reduce hauling times, increase crop yields, and trigger spontaneous, bizarre mutations in Pops or wildlife. It forces players to aggressively capitalize on the temporary boon while managing the unpredictable, potentially disastrous side effects of altered natural laws.

## 2. Dependencies
- Layer 2 planetary movement/alignment tracking
- Layer 1 `ColonyResources` (for gathering/hauling speed boosts)
- `Chronicle` system to announce the alignment

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use scale::layer2::alignment::{SyzygyEvent, PlanetaryAlignment};

    #[test]
    fn test_syzygy_trigger() {
        let mut world = setup_test_world();
        let alignment = world.spawn(PlanetaryAlignment { days_until: 0 }).id();

        trigger_syzygy(&mut world, alignment);

        assert!(world.contains_resource::<SyzygyActive>());
        assert!(get_chronicle_events(&world).contains("syzygy_begins"));
    }

    #[test]
    fn test_syzygy_hauling_boost() {
        let mut world = setup_test_world();
        world.insert_resource(SyzygyActive { duration: 10 });
        let hauler = world.spawn(Hauler { speed: 10 }).id();

        apply_syzygy_effects(&mut world);

        let boosted_speed = world.get::<Hauler>(hauler).unwrap().speed;
        assert!(boosted_speed > 10);
    }

    #[test]
    fn test_syzygy_mutation_chance() {
        let mut world = setup_test_world();
        world.insert_resource(SyzygyActive { duration: 10 });
        let pop = world.spawn((Pop, GridPosition { x: 5, y: 5 })).id();

        trigger_random_mutation(&mut world, pop);

        assert!(world.get::<MutatedTrait>(pop).is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct SyzygyActive {
    pub duration: u32,
}

#[derive(Component)]
pub struct PlanetaryAlignment {
    pub days_until: u32,
}

#[derive(Component)]
pub struct Hauler {
    pub speed: u32,
}

#[derive(Component)]
pub struct MutatedTrait;

pub fn trigger_syzygy(world: &mut World, entity: Entity) {
    if let Some(alignment) = world.get::<PlanetaryAlignment>(entity) {
        if alignment.days_until == 0 {
            world.insert_resource(SyzygyActive { duration: 30 });
            let mut chronicle = world.get_resource_mut::<Chronicle>().unwrap();
            chronicle.add_event("syzygy_begins".to_string());
        }
    }
}

pub fn apply_syzygy_effects(world: &mut World) {
    if world.contains_resource::<SyzygyActive>() {
        for mut hauler in world.query::<&mut Hauler>().iter_mut() {
            hauler.speed += 5;
        }
    }
}

pub fn trigger_random_mutation(world: &mut World, entity: Entity) {
    if world.contains_resource::<SyzygyActive>() {
        world.entity_mut(entity).insert(MutatedTrait);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:** Replace the hardcoded `hauler.speed += 5` with a configurable buff modifier system that gracefully expires when the Syzygy ends.
- **Code Smells:** Direct mutation assignment lacks nuance; implement a weighted pool of possible mutations (both beneficial and harmful).
- **Performance:** Do not iterate over every Pop for mutation chances every tick; process mutations sporadically or during specific actions (like harvesting) while the Syzygy is active.
- **API Improvements:** Create `SyzygyStartEvent` and `SyzygyEndEvent` so disparate systems (hauling, farming, mutation) can independently subscribe and apply their specific modifiers.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] The Syzygy activates correctly and temporarily alters hauling speeds and triggers mutations.

## 7. Technical Guidance
- **Code Structure:** Place alignment logic in `src/layer2/alignment.rs` and the applied effects in relevant Layer 1 modules (e.g., `src/layer1/hauling.rs`, `src/layer1/biology.rs`).
- **Integration Points:** Ensure the Syzygy duration decrements with the global `SimulationTime` tick.
- **Gotchas:** Remember to revert the speed boosts when the Syzygy ends. A modifier component (e.g., `SpeedModifier`) is safer than permanently altering base stats.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
