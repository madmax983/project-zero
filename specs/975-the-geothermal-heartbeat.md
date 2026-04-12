# Spec 975: The Geothermal Heartbeat

## 1. Overview
The "Geothermal Heartbeat" introduces a rhythmic cycle of energy and danger to colony planets. Certain map tiles function as "Geothermal Vents." These vents undergo cycles where they are either dormant or emitting a massive "Geothermal Pulse." When a pulse is active, any building located directly on a vent tile receives a significant boost to its power output or production efficiency (a `GeothermalBoost` component). However, during the pulse, the building will suffer rapid durability decay. If its health (`Structure`) reaches zero while active on a pulsing vent, it explodes, damaging adjacent structures. If the building is shut down (`active = false` or similar state), it does not receive the boost, nor does it suffer the decay.

This creates a high-risk, high-reward tension where players must actively micromanage or carefully reinforce industrial zones built over vents.

## 2. Dependencies
- `TerrainGrid` and map position systems (`GridPosition`).
- `Building`, `Structure` (durability), `PowerSource`, and `PowerConsumer` components.
- Explosion/damage system (likely via a deferred command or directly applying damage to `Structure` components in adjacent cells).

## 3. RED Phase: Tests First

```rust
// tests/geothermal_pulse_tests.rs
use bevy::prelude::*;
use crate::layer1::architecture::structure::Structure;
use crate::layer1::energy::PowerSource;
use crate::layer1::environment::geothermal::{
    GeothermalVent, GeothermalPulseEvent, GeothermalPulseState,
    geothermal_pulse_system, geothermal_boost_system, geothermal_decay_system
};
use crate::layer1::map::GridPosition;

#[test]
fn test_geothermal_vent_pulse_cycle() {
    let mut app = App::new();
    app.init_resource::<GeothermalPulseState>();
    app.add_systems(Update, geothermal_pulse_system);

    // Initial state: Dormant
    let state = app.world().resource::<GeothermalPulseState>();
    assert!(!state.is_pulsing);

    // Advance time or trigger event to start pulse
    // (Implementation detail depends on how cycles are timed, assume an event or tick threshold)
    app.world_mut().send_event(GeothermalPulseEvent { is_active: true });
    app.update();

    let state = app.world().resource::<GeothermalPulseState>();
    assert!(state.is_pulsing);
}

#[test]
fn test_building_on_vent_receives_boost_and_decay_during_pulse() {
    let mut app = App::new();
    app.init_resource::<GeothermalPulseState>();
    app.world_mut().resource_mut::<GeothermalPulseState>().is_pulsing = true;

    app.add_systems(Update, (geothermal_boost_system, geothermal_decay_system));

    let vent_pos = GridPosition { x: 5, y: 5 };

    // Spawn a vent
    app.world_mut().spawn((
        GeothermalVent,
        vent_pos.clone(),
    ));

    // Spawn an active power source building on the vent
    let building_entity = app.world_mut().spawn((
        PowerSource { output: 10.0, active: true },
        Structure { health: 100.0, max_health: 100.0 },
        vent_pos.clone(),
    )).id();

    app.update();

    let power = app.world().get::<PowerSource>(building_entity).unwrap();
    let structure = app.world().get::<Structure>(building_entity).unwrap();

    // Verify boost
    assert!(power.output > 10.0, "Building should receive power output boost");
    // Verify decay
    assert!(structure.health < 100.0, "Building should suffer rapid decay during pulse");
}

#[test]
fn test_inactive_building_on_vent_ignores_pulse() {
    let mut app = App::new();
    app.init_resource::<GeothermalPulseState>();
    app.world_mut().resource_mut::<GeothermalPulseState>().is_pulsing = true;

    app.add_systems(Update, (geothermal_boost_system, geothermal_decay_system));

    let vent_pos = GridPosition { x: 5, y: 5 };

    app.world_mut().spawn((GeothermalVent, vent_pos.clone()));

    let building_entity = app.world_mut().spawn((
        PowerSource { output: 10.0, active: false }, // Inactive!
        Structure { health: 100.0, max_health: 100.0 },
        vent_pos.clone(),
    )).id();

    app.update();

    let power = app.world().get::<PowerSource>(building_entity).unwrap();
    let structure = app.world().get::<Structure>(building_entity).unwrap();

    // Should not be boosted
    assert_eq!(power.output, 10.0, "Inactive building should not receive power output boost");
    // Should not decay
    assert_eq!(structure.health, 100.0, "Inactive building should not suffer decay during pulse");
}

#[test]
fn test_building_explodes_when_decay_reaches_zero() {
    let mut app = App::new();
    app.init_resource::<GeothermalPulseState>();
    app.world_mut().resource_mut::<GeothermalPulseState>().is_pulsing = true;

    app.add_systems(Update, geothermal_decay_system);

    let vent_pos = GridPosition { x: 5, y: 5 };
    let adj_pos = GridPosition { x: 5, y: 6 };

    app.world_mut().spawn((GeothermalVent, vent_pos.clone()));

    // Building about to explode
    let building_entity = app.world_mut().spawn((
        PowerSource { output: 10.0, active: true },
        Structure { health: 1.0, max_health: 100.0 }, // 1 health, will drop below 0
        vent_pos.clone(),
    )).id();

    // Adjacent building to take collateral damage
    let adj_building = app.world_mut().spawn((
        Structure { health: 100.0, max_health: 100.0 },
        adj_pos.clone(),
    )).id();

    app.update();

    // The vent building should be destroyed (or marked for destruction)
    assert!(app.world().get::<Structure>(building_entity).is_none(), "Building should be destroyed");

    // Adjacent building should take damage
    let adj_health = app.world().get::<Structure>(adj_building).unwrap().health;
    assert!(adj_health < 100.0, "Adjacent building should take collateral explosion damage");
}
```

## 4. GREEN Phase: Minimal Implementation
- Create `src/layer1/environment/geothermal.rs`.
- Define `GeothermalVent` (component), `GeothermalPulseState` (resource), and `GeothermalPulseEvent`.
- `geothermal_pulse_system`: Toggle `GeothermalPulseState::is_pulsing` based on events or a timer.
- `geothermal_boost_system`: Query `(&mut PowerSource, &GridPosition)` for active buildings, check if a `GeothermalVent` is at the same position, and if `is_pulsing` is true, temporarily increase the output (e.g., multiplier of 2.0). Remember to reset the output when the pulse ends or the building becomes inactive. This might require a temporary `GeothermalBoosted` component to track original values.
- `geothermal_decay_system`: Query `(&mut Structure, &PowerSource, &GridPosition, Entity)` for active buildings on vents during a pulse. Subtract health. If health <= 0, despawn the entity, and apply damage to any `Structure` at adjacent `GridPosition`s.

## 5. REFACTOR Phase: Quality & Design
- **Value Tracking:** Extract the boost application so that base output is preserved. Creating a temporary `GeothermalBoosted` component holding the original `f32` output is a good pattern.
- **Explosion Logic:** If an explosion system or event already exists in the codebase (e.g., `ExplosionEvent`), use that instead of manually looping over adjacent tiles.
- **Modularity:** Ensure the `geothermal_decay_system` logic handles different types of active buildings (not just `PowerSource`—perhaps `Production` or `ActiveBuilding` tags depending on how activity is checked across the codebase).

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new `geothermal` module.
- [ ] A Geothermal Pulse correctly applies a boost and rapid decay to active buildings on vents.
- [ ] Inactive buildings ignore the pulse.
- [ ] Buildings destroyed by the pulse explode, damaging adjacent tiles.

## 7. Technical Guidance
- **Explosions:** When despawning the zero-health building, use `commands.entity(entity).despawn()`. To deal damage to adjacent tiles, you can iterate over directions `[(0, 1), (1, 0), (0, -1), (-1, 0)]`, compute the new coordinates, and iterate through all `Structure` components to find matches, or use the `TerrainGrid` spatial index if one is available.
- **Activity Check:** Not all buildings use `PowerSource`. If you want to support generic factories, you may need a broader query (e.g., querying for a generic `Active` component or checking multiple specific components). For the MVP, supporting `PowerSource` and standard `Structure` decay is sufficient.
- **Timer/Events:** For the pulse cycle, you can use a `bevy::time::Timer` in the `GeothermalPulseState` resource to automatically flip between dormant and pulsing states every N ticks/seconds, or rely on explicit events.

## 8. Questions
*Builder: add questions here if spec is unclear.*
