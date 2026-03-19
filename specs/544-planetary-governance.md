
# Specification: Planetary Governance

## 1. Overview
**Layer:** 2
**Fantasy:** You are the Emperor, not the Mayor.
**Mechanic:** Assign high-ranking Pops as "Governors" to planets. They automate build queues and provide passive bonuses based on traits, but may develop "Corruption" or "Ambition".
**Emergence:** Your most efficient governor declares independence because you let them get too powerful.
**Tension:** Micro-manage (efficiency) vs. Delegate (scale but risk).

## 2. Dependencies
- Layer 2 nodes/planets (`src/layer2/system.rs`)
- Pop traits/factions (`src/layer1/traits.rs`, `src/layer1/factions.rs`)

## 3. RED Phase: Tests First

```rust
use bevy_ecs::prelude::*;

#[test]
fn test_assign_governor_applies_bonuses() {
    let mut world = World::new();
    // Arrange: Create a Planet and a Pop with a "Logistics Expert" trait. Assign Pop as Governor.
    let pop = world.spawn((Pop, Trait::LogisticsExpert)).id();
    let planet = world.spawn((
        Planet,
        PlanetProduction { base_throughput: 1.0, current_throughput: 1.0 },
        Governor { entity: pop }
    )).id();

    // Act: Advance simulation.
    // apply_governor_effects_system(&mut world);

    // Assert: The Planet receives a bonus to logistics/production throughput.
    // let production = world.get::<PlanetProduction>(planet).unwrap();
    // assert!(production.current_throughput > production.base_throughput, "Governor trait should boost production throughput.");
}

#[test]
fn test_governor_accumulates_ambition_over_time() {
    let mut world = World::new();
    // Arrange: Assign a Governor to a highly prosperous planet.
    let pop = world.spawn((Pop, GovernorStats { ambition: 0.0, corruption: 0.0 })).id();
    let planet = world.spawn((Planet, ProsperityRating(10.0), Governor { entity: pop })).id();

    // Act: Advance simulation by a long period.
    // update_governor_ambition_system(&mut world);

    // Assert: The Governor's "Ambition" stat increases.
    // let stats = world.get::<GovernorStats>(pop).unwrap();
    // assert!(stats.ambition > 0.0, "Governor of a prosperous planet should accumulate ambition.");
}

#[test]
fn test_high_ambition_triggers_rebellion_event() {
    let mut world = World::new();
    // Arrange: Set a Governor's Ambition to maximum.
    let pop = world.spawn((Pop, GovernorStats { ambition: 100.0, corruption: 0.0 })).id();
    let planet = world.spawn((Planet, Governor { entity: pop })).id();

    // Add event reader
    // let mut events = world.resource_mut::<Events<RebellionEvent>>();
    // events.clear();

    // Act: Advance simulation.
    // check_governor_rebellion_system(&mut world);

    // Assert: A "Rebellion" or "Secession" event is generated for that planet.
    // let events = world.resource::<Events<RebellionEvent>>();
    // let mut reader = events.get_reader();
    // assert!(reader.iter(&events).any(|e| e.planet_entity == planet), "High ambition governor should trigger a RebellionEvent.");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// 1. Add a `Governor` component linked to a specific planet entity.
// 2. Add `GovernorStats` component (tracking Ambition/Corruption).
// 3. Create `apply_governor_effects_system` that reads the Governor's traits and applies modifiers to the planet's production/unrest.
// 4. Create `update_governor_ambition_system` that slowly increments Ambition based on planet prosperity or time.
// 5. Add a check to trigger a `RebellionEvent` when Ambition crosses a threshold.
```

## 5. REFACTOR Phase: Quality & Design
- Ensure traits applied to planets are clearly displayed in the UI (or generate an event so the player knows *why* production increased).
- The link between Layer 1 Pops and Layer 2 Governors needs careful lifetime management. If the Pop dies on Layer 1, they must be unassigned on Layer 2.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Governors can be assigned to planets and apply trait-based bonuses.
- [ ] Governor ambition increases over time, leading to rebellion events if unchecked.

## 7. Technical Guidance
- A Governor might technically exist as an entity on Layer 2 even if their "body" is on Layer 1. Decide if the Pop physically moves to a "Governor's Mansion" or if it's just a status tag. A simple status tag is easier for the MVP.

## 8. Questions
*Builder: add questions here if spec is unclear.*
