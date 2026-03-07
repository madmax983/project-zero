# 413-Cryo-Shock

## 1. Overview
**Layer:** 1
**Fantasy:** The human cost of suspension. You don't just "appear" from a pod ready to build a civilization; you are groggy, cold, and weak.
**Mechanic:** Newly spawned pops (from Cryo-ships) start with severe debuffs (slow movement, confusion, nausea). They require "Triage" or time in a "Recovery Bed" before working effectively.
**Emergence:** A raid hits just as a new ship lands. You desperately wake up the marines, but they are vomiting and stumbling, unable to aim, forcing you to use civilians for defense.
**Tension:** Wake them early for bodies (useless/sick) or wait for slow acclimation (efficiency)?

## 2. Dependencies
- Spawning system (Pops arriving via ship).
- Status Effect/Buff system (applying Movement/Work speed penalties).
- Needs/Health system (Nausea/Confusion).
- Hospital/Recovery beds (infrastructure for triage).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_cryo_shock_applied_on_spawn() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // ... setup world ...

        // Act
        // Spawn a pop from a cryo event
        let pop = app.world_mut().spawn(Pop::from_cryo()).id();
        app.update();

        // Assert
        // Check if the pop has the CryoShock status effect
        assert!(app.world().get::<CryoShock>(pop).is_some(), "Pops spawned from cryo must have CryoShock");
    }

    #[test]
    fn test_cryo_shock_applies_debuffs() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let pop = app.world_mut().spawn((
            Pop,
            MovementSpeed(10.0), // Base speed
            CryoShock { duration: 100.0, severity: 0.5 },
        )).id();

        // Act
        // Run system that calculates effective speed
        app.add_systems(Update, apply_cryo_debuff_system);
        app.update();

        // Assert
        // Effective speed should be significantly reduced
        let speed = app.world().get::<EffectiveMovementSpeed>(pop).unwrap().0;
        assert!(speed < 10.0, "Cryo shock must reduce effective movement speed");
        assert_eq!(speed, 5.0, "Cryo shock severity 0.5 halves speed");
    }

    #[test]
    fn test_cryo_shock_duration_decays() {
         // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let pop = app.world_mut().spawn((
            Pop,
            CryoShock { duration: 10.0, severity: 0.5 },
            Time::default(), // Assuming we need time elapsed
        )).id();

        // Act
        // Advance time and run decay system
        app.world_mut().resource_mut::<Time>().update(); // Simulate tick
        app.add_systems(Update, decay_cryo_shock_system);
        app.update();

        // Assert
        // Check if duration is reduced
        let shock = app.world().get::<CryoShock>(pop).unwrap();
        assert!(shock.duration < 10.0, "Cryo shock duration must decay over time");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass

#[derive(Component)]
pub struct Pop;

impl Pop {
    pub fn from_cryo() -> (Self, CryoShock) {
        (Self, CryoShock { duration: 100.0, severity: 0.5 })
    }
}

#[derive(Component)]
pub struct CryoShock {
    pub duration: f32,
    pub severity: f32,
}

#[derive(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component)]
pub struct EffectiveMovementSpeed(pub f32);

pub fn apply_cryo_debuff_system(
    mut query: Query<(Entity, &MovementSpeed, Option<&CryoShock>), Without<EffectiveMovementSpeed>>,
    mut commands: Commands,
) {
    for (entity, base_speed, shock) in query.iter_mut() {
        let mut eff = base_speed.0;
        if let Some(s) = shock {
            eff *= (1.0 - s.severity);
        }
        commands.entity(entity).insert(EffectiveMovementSpeed(eff));
    }
}

pub fn decay_cryo_shock_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CryoShock)>,
    time: Res<Time>,
) {
    for (entity, mut shock) in query.iter_mut() {
        shock.duration -= time.delta_seconds(); // Simplified for test context
        if shock.duration <= 0.0 {
            commands.entity(entity).remove::<CryoShock>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:**
  - `CryoShock` should be handled generically via a central `StatusEffect` system rather than bespoke logic if possible.
  - Integration with the Triage/Medical bed system: Being in a medical bed should multiply the decay rate (e.g., `shock.duration -= delta * 5.0`).
- **Code Smells:**
  - `EffectiveMovementSpeed` as a static Component inserted via Commands; it should likely be calculated continuously in a unified stats pipeline so changes are reactive.
- **Performance Considerations:**
  - Calculating effective stats should be cached or only updated when components change to avoid overhead every tick.
- **API Improvements:**
  - `Pop::from_cryo()` is clean, but a generic `SpawnPopEvent { origin: SpawnOrigin::Cryo }` is better for data-driven design.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops arriving via Cryo spawn with `CryoShock`.
- [ ] `CryoShock` severely debuffs movement and work speed.
- [ ] `CryoShock` naturally decays over time until the pop fully recovers.

## 7. Technical Guidance
- **Code Structure:** `src/layer1/pops/cryo_shock.rs`.
- **Integration Points:**
  - `layer1/pops.rs` or the spawning logic that handles arrivals.
  - `layer1/stats.rs` or wherever movement and work speed modifiers are applied.
- **Gotchas:** Make sure the visual/UI layer indicates this debuff clearly so players understand why their new arrivals are crawling around instead of hauling rocks.

## 8. Questions
*Builder: add questions here if spec is unclear.*
