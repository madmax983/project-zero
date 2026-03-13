# 368 The Star-Eater's Wake

## 1. Overview
A massive, dark-matter entity (The Star-Eater) passes through the star system (Layer 2). It doesn't attack, but its wake siphons energy from the sun, plunging Layer 1 into a sudden, deep freeze ("The Long Dark"). It also leaves behind "Dark Motes", an incredibly volatile but energy-dense resource. This creates an immediate survival crisis balanced by a huge resource windfall.

## 2. Dependencies
- `094-system-view-architecture.md` (for Layer 2 entities)
- `140-thermal-management.md` (for Temperature on Layer 1)
- `018-mining-resources.md` (for Dark Motes collection)

## 3. RED Phase: Tests First

```rust
// tests/integration/star_eater_test.rs

use crate::layer2::system::StarEaterEvent;
use crate::layer1::weather::TemperatureGrid;
use crate::layer1::resources::{ColonyResources, ResourceType};

#[test]
fn test_star_eater_triggers_global_freeze() {
    let mut app = setup_test_app();
    let initial_temp = app.world().resource::<TemperatureGrid>().average_temp();

    // Star-Eater arrives
    app.world_mut().send_event(StarEaterEvent::Arrival);
    app.update();

    // Temperature should plummet
    let new_temp = app.world().resource::<TemperatureGrid>().average_temp();
    assert!(new_temp < initial_temp - 50.0);
}

#[test]
fn test_star_eater_spawns_dark_motes() {
    let mut app = setup_test_app();

    // The Star-Eater passes and leaves motes behind
    app.world_mut().send_event(StarEaterEvent::Departure);
    app.update();

    // Check map for motes
    let motes = app.world().query::<&DarkMote>().iter(app.world()).count();
    assert!(motes > 0);
}

#[test]
fn test_burning_dark_motes_heats_grid() {
    let mut app = setup_test_app();
    app.world_mut().resource_mut::<ColonyResources>().add(ResourceType::DarkMote, 10);
    let initial_temp = app.world().resource::<TemperatureGrid>().average_temp();

    // Burn 1 mote
    app.world_mut().send_event(BurnMoteEvent { amount: 1 });
    app.update();

    let resources = app.world().resource::<ColonyResources>();
    assert_eq!(resources.get_amount(ResourceType::DarkMote), 9);

    let new_temp = app.world().resource::<TemperatureGrid>().average_temp();
    assert!(new_temp > initial_temp + 20.0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/tech/star_eater.rs
use bevy::prelude::*;
use crate::layer1::weather::TemperatureGrid;
use crate::layer1::resources::{ColonyResources, ResourceType};

#[derive(Event)]
pub enum StarEaterEvent {
    Arrival,
    Departure,
}

#[derive(Event)]
pub struct BurnMoteEvent {
    pub amount: u32,
}

#[derive(Component)]
pub struct DarkMote;

pub fn handle_star_eater_events(
    mut events: EventReader<StarEaterEvent>,
    mut temperature: ResMut<TemperatureGrid>,
    mut commands: Commands,
) {
    for event in events.read() {
        match event {
            StarEaterEvent::Arrival => {
                // Drop global temp significantly
                for t in temperature.cells.iter_mut() {
                    *t -= 60.0;
                }
            }
            StarEaterEvent::Departure => {
                // Spawn motes around map (simplified here)
                commands.spawn(DarkMote);
                commands.spawn(DarkMote);
                commands.spawn(DarkMote);
            }
        }
    }
}

pub fn burn_mote_system(
    mut burn_events: EventReader<BurnMoteEvent>,
    mut resources: ResMut<ColonyResources>,
    mut temperature: ResMut<TemperatureGrid>,
) {
    for ev in burn_events.read() {
        if resources.get_amount(ResourceType::DarkMote) >= ev.amount {
            // Since we can't generic consume safely for deductions without specific methods, assume `consume_dark_mote` exists
            // Or use generic consume if it's available. Assuming `consume` is valid per `AGENTS.md` docs.
            resources.consume(ResourceType::DarkMote, ev.amount);

            // Apply massive heat to grid
            for t in temperature.cells.iter_mut() {
                *t += 25.0 * (ev.amount as f32);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Mote Spawning:** Use a procedural placement algorithm (like noise) rather than raw `spawn` calls.
- **Side-Effects:** Add a mutation chance or lung damage debuff when pops are near burning motes without protection.
- **Duration:** Implement a timer for the Long Dark instead of an instant Departure event.

## 6. Acceptance Criteria
- [ ] Tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.
- [ ] `StarEaterEvent::Arrival` drastically lowers `TemperatureGrid`.
- [ ] `StarEaterEvent::Departure` spawns `DarkMote` entities.
- [ ] Burning Dark Motes increases local temperature.

## 7. Technical Guidance
- `TemperatureGrid` might require a localized "heater" entity rather than a global buff when burning motes.
- Integrate with Layer 2 by triggering the Arrival based on a specific `Entity` moving across the system map.

## 8. Questions
*Builder: Add any questions here.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
