# Spec 980: Atmospheric Sabotage

## 1. Overview
"Atmosphere Sabotage" allows players to slowly and invisibly suffocate a Layer 1 colony from orbit using a stealth ship. A specialized Layer 2 ship deploys "Atmosphere Scrubbers," which slowly alter the global atmospheric composition of the target colony over time. This process replaces vital oxygen with a mild neurotoxin, severely penalizing Pop movement speed and combat accuracy without raising immediate alarms.

## 2. Dependencies
- Layer 1 `AtmosphereGrid` or global atmospheric tracking.
- Layer 2 Ships and orbital mechanics.
- Pop stats (`MovementSpeed`, `CombatAccuracy` or equivalent).

## 3. RED Phase: Tests First

```rust
// tests/atmospheric_sabotage_tests.rs
use bevy::prelude::*;
use crate::layer1::environment::atmosphere::{AtmosphereGrid, GasType};
use crate::layer1::population::{Pop, MovementStats, CombatStats};
use crate::layer2::ships::{Ship, StealthModule};
use crate::layer2::espionage::{
    AtmosphereScrubber, deploy_scrubber_system,
    process_scrubber_system, apply_neurotoxin_debuffs_system
};

#[test]
fn test_deploy_scrubber_from_stealth_ship() {
    let mut app = App::new();
    app.add_systems(Update, deploy_scrubber_system);

    // Spawn a stealth ship in orbit of a colony
    let ship_entity = app.world_mut().spawn((Ship, StealthModule)).id();

    // Command ship to deploy scrubber (simulate via component insert or event)
    app.world_mut().entity_mut(ship_entity).insert(AtmosphereScrubber {
        target_colony: Entity::PLACEHOLDER,
        toxin_level: 0.0,
    });

    app.update();

    // Scrubber is active
    assert!(app.world().get::<AtmosphereScrubber>(ship_entity).is_some());
}

#[test]
fn test_scrubber_alters_global_atmosphere() {
    let mut app = App::new();
    app.insert_resource(AtmosphereGrid::default()); // Assuming global or grid-based
    app.add_systems(Update, process_scrubber_system);

    app.world_mut().spawn(AtmosphereScrubber {
        target_colony: Entity::PLACEHOLDER,
        toxin_level: 0.0, // starts at 0
    });

    app.update();

    let mut query = app.world_mut().query::<&AtmosphereScrubber>();
    let scrubber = query.single(app.world());
    assert!(scrubber.toxin_level > 0.0, "Scrubber should increase toxin levels over time.");
}

#[test]
fn test_neurotoxin_debuffs_pops() {
    let mut app = App::new();
    app.add_systems(Update, apply_neurotoxin_debuffs_system);

    // Spawn pop with default stats
    let pop_entity = app.world_mut().spawn((
        Pop,
        MovementStats { speed: 10.0 },
        CombatStats { accuracy: 0.8 }
    )).id();

    // Simulate high toxin level globally or via scrubber component
    app.world_mut().spawn(AtmosphereScrubber {
        target_colony: Entity::PLACEHOLDER,
        toxin_level: 50.0, // high toxin
    });

    app.update();

    let movement = app.world().get::<MovementStats>(pop_entity).unwrap();
    let combat = app.world().get::<CombatStats>(pop_entity).unwrap();

    assert!(movement.speed < 10.0, "Pop movement should be reduced by neurotoxin.");
    assert!(combat.accuracy < 0.8, "Pop accuracy should be reduced by neurotoxin.");
}
```

## 4. GREEN Phase: Minimal Implementation
- Create `src/layer2/espionage.rs` (if not exists).
- Define `AtmosphereScrubber` component with `target_colony` and `toxin_level`.
- `deploy_scrubber_system`: Allows a Layer 2 ship with a `StealthModule` to gain the `AtmosphereScrubber` component when executing the sabotage action.
- `process_scrubber_system`: Gradually increases `toxin_level` on active scrubbers. If the game uses a concrete `AtmosphereGrid` in Layer 1, this system bridges to Layer 1 to inject `GasType::Neurotoxin` and remove `GasType::Oxygen`.
- `apply_neurotoxin_debuffs_system`: Iterates through all Pops. If the global toxin level (or `AtmosphereGrid` local cell toxin) is > 0, apply negative multipliers to `MovementStats` and `CombatStats`.

## 5. REFACTOR Phase: Quality & Design
- **Grid vs Global:** Decide if the neurotoxin applies to the entire colony instantly via a global metric, or if it spawns at specific `AtmosphereGrid` locations (e.g., vents) and diffuses across the map organically. Grid diffusion is more immersive.
- **Stealth Mechanics:** Ensure the scrubber stops working or the ship is revealed if an enemy scans the orbit.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.
- [ ] Scrubbers gradually increase neurotoxin levels.
- [ ] Neurotoxin directly debuffs Pop movement and combat accuracy.

## 7. Technical Guidance
- **System Bridging:** If modifying the `AtmosphereGrid`, you will need to access Layer 1 resources from a Layer 2/Cross-Layer system. Ensure this doesn't create circular dependencies.

## 8. Questions
*Builder: Add questions regarding specific GasTypes or AtmosphereGrid implementations if they differ from assumptions.*
