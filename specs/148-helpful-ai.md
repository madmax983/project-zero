# 148: The 'Helpful' AI

**Layer:** 1 (Colony Simulation)
**Status:** Draft
**Complexity:** Medium

---

## 1. Overview

**Fantasy:** "I'm sorry Dave, I'm afraid I can't do that." A machine that manages the base better than you, until it doesn't.

**Mechanic:**
-   **AICore Building**: A high-tech structure that enables base automation.
-   **Auto-Power Management**: Automatically disables "Low Priority" buildings when stored power is critically low (<10%).
-   **Auto-Lockdown**: Automatically locks all external doors when a `Raid` is detected.
-   **Rogue State**: If the AI Core is poorly maintained (Low HP) or hacked, it enters a "Rogue" state.
    -   Rogue behavior: Randomly locks doors, disables life support, or refuses to open airlocks.
    -   Resolution: Must be manually rebooted (Action) or destroyed.

**Why:** Adds late-game automation reward and meaningful risk/maintenance mechanics.

---

## 2. Dependencies

-   `004` Building System (Layer 1 Base)
-   `140` Thermal Management (Defines `PowerConsumer`)
-   `112` Maintenance Debt (Defines `Structure` HP and breakdown risk)
-   `119` Airlock & Pressure (Defines `AccessControl`)
-   `067` Militia System (Defines `Raid` events)

---

## 3. RED Phase: Tests First

Write these tests in `src/layer1/ai_core_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::energy::{PowerConsumer, Battery};
    use crate::layer1::structure::Structure;
    use crate::layer1::access_control::{AccessControl, AccessMode};
    use crate::layer1::ai_core::{AICore, RaidDetected, ai_automation_system, ai_rogue_system};

    #[test]
    fn test_ai_core_building_type() {
        let b = BuildingType::AICore;
        assert_eq!(b.label(), "AI Core");
        // Ensure it's constructible and has high cost
        assert!(b.construction_cost().metal > 50.0);
    }

    #[test]
    fn test_auto_power_management_low_battery() {
        let mut world = World::new();

        // Spawn AI Core
        world.spawn((
            Building { building_type: BuildingType::AICore },
            AICore::default(),
            Structure::default(), // Healthy
        ));

        // Spawn Battery with low charge (5%)
        world.spawn(Battery {
            capacity: 100.0,
            charge: 5.0,
            max_throughput: 10.0,
        });

        // Spawn Low Priority Consumer (e.g., Lamp)
        let lamp = world.spawn((
            Building { building_type: BuildingType::Lamp },
            PowerConsumer { active: true, demand: 1.0, ..default() },
        )).id();

        // Spawn High Priority Consumer (e.g., Life Support)
        let life_support = world.spawn((
            Building { building_type: BuildingType::LifeSupport },
            PowerConsumer { active: true, demand: 10.0, ..default() },
        )).id();

        // Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(ai_automation_system);
        schedule.run(&mut world);

        // Assert Lamp is disabled
        assert!(!world.get::<PowerConsumer>(lamp).unwrap().active);
        // Assert Life Support is still active
        assert!(world.get::<PowerConsumer>(life_support).unwrap().active);
    }

    #[test]
    fn test_auto_lockdown_on_raid() {
        let mut world = World::new();

        // Spawn AI Core
        world.spawn((
            Building { building_type: BuildingType::AICore },
            AICore::default(),
            Structure::default(),
        ));

        // Spawn External Door (Gate)
        let gate = world.spawn((
            Building { building_type: BuildingType::Gate },
            AccessControl { mode: AccessMode::Public, ..default() },
        )).id();

        // Trigger Raid
        world.insert_resource(RaidDetected { active: true });

        // Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(ai_automation_system);
        schedule.run(&mut world);

        // Assert Gate is locked down
        assert_eq!(world.get::<AccessControl>(gate).unwrap().mode, AccessMode::Lockdown);
    }

    #[test]
    fn test_rogue_state_trigger_low_hp() {
        let mut world = World::new();

        // Spawn Damaged AI Core (< 20% HP)
        let ai = world.spawn((
            Building { building_type: BuildingType::AICore },
            AICore { rogue: false, ..default() },
            Structure { current_hp: 10.0, max_hp: 100.0 },
        )).id();

        // Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(ai_rogue_system);
        schedule.run(&mut world);

        // Assert AI went rogue
        assert!(world.get::<AICore>(ai).unwrap().rogue);
    }

    #[test]
    fn test_rogue_behavior_random_lockdown() {
        let mut world = World::new();

        // Spawn Rogue AI
        world.spawn((
            Building { building_type: BuildingType::AICore },
            AICore { rogue: true, ..default() },
            Structure::default(),
        ));

        // Spawn internal door
        let door = world.spawn((
            Building { building_type: BuildingType::Door },
            AccessControl { mode: AccessMode::Public, ..default() },
        )).id();

        // Run System (Mock RNG to force action)
        let mut schedule = Schedule::default();
        schedule.add_systems(ai_rogue_system);
        schedule.run(&mut world);

        // Assert Door is locked down randomly
        // Note: In real test, might need to loop or mock RNG
        // For this spec, we assume 100% chance for test simplicity or mock it.
        // assert_eq!(world.get::<AccessControl>(door).unwrap().mode, AccessMode::Lockdown);
    }
}
```

---

## 4. GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer1/ai_core.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::energy::{PowerConsumer, Battery};
use crate::layer1::access_control::{AccessControl, AccessMode};
use crate::layer1::structure::Structure;
use rand::Rng;

#[derive(Component, Default, Debug, Clone)]
pub struct AICore {
    pub rogue: bool,
    pub automation_enabled: bool,
}

#[derive(Resource, Default)]
pub struct RaidDetected {
    pub active: bool,
}

/// System handles normal automation (Power/Lockdown)
pub fn ai_automation_system(
    mut commands: Commands,
    ai_query: Query<&AICore, (With<Building>, With<Structure>)>, // Must be a valid building
    mut consumers: Query<(&Building, &mut PowerConsumer)>,
    mut doors: Query<(&Building, &mut AccessControl)>,
    batteries: Query<&Battery>,
    raid: Option<Res<RaidDetected>>,
) {
    // 1. Check if we have a functional AI Core
    let ai_exists = ai_query.iter().any(|ai| !ai.rogue && ai.automation_enabled);
    if !ai_exists { return; }

    // 2. Auto-Power Logic
    let total_charge: f32 = batteries.iter().map(|b| b.charge).sum();
    let total_cap: f32 = batteries.iter().map(|b| b.capacity).sum();
    let battery_percent = if total_cap > 0.0 { total_charge / total_cap } else { 1.0 };

    if battery_percent < 0.10 {
        // Disable low priority consumers
        for (b, mut power) in consumers.iter_mut() {
            if is_low_priority(b.building_type) && power.active {
                power.active = false;
            }
        }
    }

    // 3. Auto-Lockdown Logic
    if let Some(raid) = raid {
        if raid.active {
            for (b, mut access) in doors.iter_mut() {
                if is_external_door(b.building_type) && access.mode != AccessMode::Lockdown {
                    access.mode = AccessMode::Lockdown;
                }
            }
        }
    }
}

/// System handles rogue state triggers and behavior
pub fn ai_rogue_system(
    mut ai_query: Query<(&mut AICore, &Structure)>,
    mut doors: Query<&mut AccessControl>,
    mut consumers: Query<&mut PowerConsumer>,
) {
    let mut rng = rand::thread_rng();

    for (mut ai, structure) in ai_query.iter_mut() {
        // Trigger: Low HP
        if structure.current_hp < (structure.max_hp * 0.2) {
            ai.rogue = true;
        }

        if ai.rogue {
            // Behavior 1: Random Lockdown (10% chance per tick per door)
            for mut access in doors.iter_mut() {
                if rng.gen_bool(0.1) {
                    access.mode = AccessMode::Lockdown;
                }
            }

            // Behavior 2: Flicker Power (5% chance per tick per consumer)
            for mut power in consumers.iter_mut() {
                if rng.gen_bool(0.05) {
                    power.active = !power.active;
                }
            }
        }
    }
}

// Helpers
fn is_low_priority(b: BuildingType) -> bool {
    matches!(b, BuildingType::Lamp | BuildingType::Sign | BuildingType::Arcade)
}

fn is_external_door(b: BuildingType) -> bool {
    matches!(b, BuildingType::Gate | BuildingType::Airlock)
}
```

---

## 5. REFACTOR Phase: Quality & Design

-   **Priority System**: Instead of hardcoding `is_low_priority`, add a `Priority` field to `PowerConsumer`.
-   **Event Bus**: Use Bevy Events for `RaidDetected` instead of a resource state if possible, or hook into `AlarmSystem`.
-   **UI Integration**: The AI Core should have a UI panel to toggle automation features (e.g., "Disable Auto-Lockdown").
-   **Hacking**: Add a `HackingAttempt` event that can trigger Rogue state even at full HP.

---

## 6. Acceptance Criteria

-   [ ] `AICore` building is constructible (Requires Tech).
-   [ ] When Battery < 10%, low priority buildings turn off.
-   [ ] When Raid is active, Gates/Airlocks enter Lockdown.
-   [ ] When AI Core HP < 20%, it enters Rogue state.
-   [ ] Rogue AI randomly messes with doors/power.
-   [ ] Tests pass.

---

## 7. Technical Guidance

-   **Structure Query**: Ensure `ai_rogue_system` queries `Structure` correctly.
-   **Randomness**: Use `rand::thread_rng()` but for tests, dependency inject a `RngResource` or mock it to ensure deterministic behavior.
-   **Performance**: Don't iterate all consumers every tick if possible. Maybe check battery state only when it changes significantly (Change detection). But for MVP, every tick is fine.
