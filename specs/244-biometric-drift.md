# 244: Biometric Drift

## Overview

"The machine remembers who you *were*, not who you *are*."

Over time, a Pop's physical characteristics change due to aging, scarring (combat/accidents), or cybernetic augmentation. Security systems (Doors, Terminals) that rely on **Biometric Locks** (Spec 142) slowly lose confidence in the Pop's identity.

This introduces **Biometric Drift**, a value tracking the divergence between a Pop's current biology and their stored profile.
- **Drift > 50%**: Access slows down (retries, "Please try again").
- **Drift > 80%**: Access Denied. The Pop cannot enter their own home or workplace.
- **Recalibration**: A new Admin/Security action to update the profile and reset drift to 0%.

This adds a maintenance loop to security: high security is safe but high-maintenance.

## Dependencies

- `004` — Pop Entity (Identity)
- `142` — Biometric Lockouts (The system being affected)
- `009` — Job System (For the Recalibration task)
- `034` — Pop Health (Damage/Scars trigger drift)

## RED Phase: Tests First

Write these tests in `src/layer1/security/biometric_drift_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::security::{BiometricProfile, SecurityTerminal, AccessResult, check_access_system, drift_accumulation_system};
    use crate::layer1::pop::{Pop, Age, Scars};
    use crate::layer1::map::GridPosition;
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_drift_accumulation_over_time() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 1000 });

        let pop = world.spawn((
            Pop,
            BiometricProfile {
                last_update_tick: 0,
                drift: 0.0,
            },
            Age { ticks: 5000 },
        )).id();

        // Run drift system
        let mut schedule = Schedule::default();
        schedule.add_systems(drift_accumulation_system);
        schedule.run(&mut world);

        let profile = world.get::<BiometricProfile>(pop).unwrap();
        // 1000 ticks should cause some drift
        assert!(profile.drift > 0.0);
    }

    #[test]
    fn test_scars_increase_drift() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            BiometricProfile { drift: 0.0, ..Default::default() },
            Scars { count: 0 },
        )).id();

        // Add a scar (simulate combat event)
        // This likely happens in a damage system, but here we manually add it and check if drift responds
        // Or we trigger a "TraumaEvent".
        // Let's assume the drift system checks for changes in Scar count vs stored count.
        // For MVP, just manually increasing drift via a helper function `apply_trauma_drift`.

        crate::layer1::security::apply_trauma_drift(&mut world, pop, 0.2);

        let profile = world.get::<BiometricProfile>(pop).unwrap();
        assert_eq!(profile.drift, 0.2);
    }

    #[test]
    fn test_access_denied_high_drift() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            BiometricProfile { drift: 0.9, ..Default::default() }, // 90% Drift
        )).id();

        let terminal = world.spawn(SecurityTerminal {
            required_clearance: 1,
            strictness: 1.0, // Strict
        }).id();

        // Check Access
        let result = crate::layer1::security::check_access(&world, pop, terminal);
        assert_eq!(result, AccessResult::DeniedDrift);
    }

    #[test]
    fn test_access_slow_medium_drift() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            BiometricProfile { drift: 0.6, ..Default::default() }, // 60% Drift
        )).id();

        let terminal = world.spawn(SecurityTerminal {
            required_clearance: 1,
            strictness: 1.0,
        }).id();

        let result = crate::layer1::security::check_access(&world, pop, terminal);
        assert_eq!(result, AccessResult::Delayed(2.0)); // 2 seconds delay
    }

    #[test]
    fn test_recalibration_resets_drift() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            BiometricProfile { drift: 0.9, last_update_tick: 0 },
        )).id();

        world.insert_resource(SimulationTime { tick: 2000 });

        // Perform recalibration action
        crate::layer1::security::recalibrate_profile(&mut world, pop);

        let profile = world.get::<BiometricProfile>(pop).unwrap();
        assert_eq!(profile.drift, 0.0);
        assert_eq!(profile.last_update_tick, 2000);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components (`src/layer1/security.rs`)

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Default)]
pub struct BiometricProfile {
    pub last_update_tick: u64,
    pub drift: f32, // 0.0 to 1.0
    // Tracked stats to calculate delta
    pub recorded_scars: u32,
}

#[derive(Component, Debug, Clone)]
pub struct SecurityTerminal {
    pub required_clearance: u8,
    pub strictness: f32, // Multiplier for drift tolerance
}

#[derive(Debug, PartialEq)]
pub enum AccessResult {
    Granted,
    Delayed(f32), // Seconds
    DeniedDrift,
    DeniedClearance,
}
```

### 2. Systems

```rust
use crate::shared::time::SimulationTime;
use crate::layer1::pop::{Age, Scars};

pub fn drift_accumulation_system(
    mut query: Query<(&mut BiometricProfile, Option<&Age>, Option<&Scars>)>,
    time: Res<SimulationTime>,
) {
    for (mut profile, age, scars) in query.iter_mut() {
        // Time-based drift (Age)
        // e.g. 0.0001 per tick
        let time_delta = time.tick.saturating_sub(profile.last_update_tick);
        let time_drift = (time_delta as f32) * 0.00001;

        // Trauma-based drift
        let current_scars = scars.map(|s| s.count).unwrap_or(0);
        let scar_drift = if current_scars > profile.recorded_scars {
            (current_scars - profile.recorded_scars) as f32 * 0.1
        } else {
            0.0
        };

        // Total
        profile.drift = (time_drift + scar_drift).min(1.0);
    }
}

pub fn check_access(world: &World, pop: Entity, terminal: Entity) -> AccessResult {
    let profile = world.get::<BiometricProfile>(pop).unwrap(); // Handle error gracefully in real code
    let term = world.get::<SecurityTerminal>(terminal).unwrap();

    // Check drift vs strictness
    // Thresholds: 0.5 (Delay), 0.8 (Deny)
    // Adjusted by strictness (Higher strictness = lower threshold)
    let effective_drift = profile.drift * term.strictness;

    if effective_drift > 0.8 {
        return AccessResult::DeniedDrift;
    } else if effective_drift > 0.5 {
        return AccessResult::Delayed(effective_drift * 5.0); // e.g. 3-4 seconds
    }

    AccessResult::Granted
}

pub fn recalibrate_profile(world: &mut World, pop: Entity) {
    let current_tick = world.resource::<SimulationTime>().tick;
    // Get current scars to sync
    let current_scars = world.get::<Scars>(pop).map(|s| s.count).unwrap_or(0);

    if let Some(mut profile) = world.get_mut::<BiometricProfile>(pop) {
        profile.drift = 0.0;
        profile.last_update_tick = current_tick;
        profile.recorded_scars = current_scars;
    }
}
```

### 3. Helpers

`apply_trauma_drift` used in tests is effectively just modifying the component, or handled automatically by the system if `Scars` component is updated. For manual test:

```rust
pub fn apply_trauma_drift(world: &mut World, pop: Entity, amount: f32) {
    if let Some(mut profile) = world.get_mut::<BiometricProfile>(pop) {
        profile.drift = (profile.drift + amount).min(1.0);
    }
}
```

## REFACTOR Phase: Quality & Design

- **UI Feedback**: When access is delayed/denied, show a floating icon ("Scanning... Failed") over the Pop.
- **Auto-Recalibrate**: High-tier "AI Security" terminals might auto-recalibrate on successful (but delayed) entry, removing the manual work at the cost of Power.
- **Cybernetics**: `CyberneticAugmentation` (Spec 151) should massive increase drift immediately.

## Acceptance Criteria

- [ ] `BiometricProfile` tracks drift.
- [ ] Time and Trauma increase drift.
- [ ] `SecurityTerminal` logic rejects high drift.
- [ ] `recalibrate_profile` resets drift.
- [ ] Tests pass.

## Questions

*Builder: Does drift affect non-security interactions?*
*Architect: Yes, it can slowly alter their social identity, meaning other Pops might stop recognizing them over time.*
*Architect: Not initially. For MVP, it strictly applies to locks and access terminals.*
*Architect: No, for MVP Biometric Drift only affects interactions that require security clearance (e.g., doors, locked chests, restricted terminals).*
*Architect: No, friends still recognize you. This is purely machine vision failure.*
