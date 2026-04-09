# 914: The Silent Strike

## 1. Overview
Workers weaponize their presence without officially rebelling. Highly dissatisfied but non-violent Pops will perform a "Silent Strike" where they show up to their workstations but produce exactly zero output. They still consume food and occupy space, making it harder to replace them with scabs.

## 2. Dependencies
- `layer1::pops` (Pop, Morale, Job components)
- `layer1::economy` (Production output calculations)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pops::{Pop, Morale, Job};
    use crate::layer1::economy::ProductionModifier;

    #[test]
    fn test_silent_strike_activation() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_silent_strike_system);

        let entity = app.world_mut().spawn((
            Pop,
            Morale { value: 10.0, threshold: 20.0 }, // Low morale
            Job { id: "miner".to_string() },
        )).id();

        app.update();

        // Should gain the SilentStrike component
        assert!(app.world().get::<SilentStrike>(entity).is_some());
    }

    #[test]
    fn test_silent_strike_production_halt() {
        let mut app = App::new();
        app.add_systems(Update, apply_silent_strike_production_modifier_system);

        let entity = app.world_mut().spawn((
            Pop,
            SilentStrike,
            ProductionModifier { multiplier: 1.0 },
        )).id();

        app.update();

        // Production multiplier should be forced to 0.0
        let modifier = app.world().get::<ProductionModifier>(entity).unwrap();
        assert_eq!(modifier.multiplier, 0.0);
    }

    #[test]
    fn test_silent_strike_resolution() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_silent_strike_system);

        let entity = app.world_mut().spawn((
            Pop,
            SilentStrike,
            Morale { value: 50.0, threshold: 20.0 }, // Morale restored
            Job { id: "miner".to_string() },
        )).id();

        app.update();

        // Should lose the SilentStrike component
        assert!(app.world().get::<SilentStrike>(entity).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pops::{Pop, Morale};
use crate::layer1::economy::ProductionModifier;

#[derive(Component)]
pub struct SilentStrike;

pub fn evaluate_silent_strike_system(
    mut commands: Commands,
    query: Query<(Entity, &Morale), With<Pop>>,
) {
    for (entity, morale) in query.iter() {
        if morale.value < morale.threshold {
            commands.entity(entity).insert(SilentStrike);
        } else {
            commands.entity(entity).remove::<SilentStrike>();
        }
    }
}

pub fn apply_silent_strike_production_modifier_system(
    mut query: Query<&mut ProductionModifier, With<SilentStrike>>,
) {
    for mut modifier in query.iter_mut() {
        modifier.multiplier = 0.0;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Configurable Thresholds:** The morale threshold for a silent strike should probably be a configurable resource rather than hardcoded or tied directly to a generic morale threshold.
- **Differentiate from standard low morale:** We might want a `Dissatisfaction` or `RebellionType` enum to determine *how* they act out (violent vs silent).
- **Consumption:** Ensure that Pops on Silent Strike still consume their daily upkeep (food/water), which is the core tension of the mechanic.

## 6. Acceptance Criteria
- [ ] TDD Tests written and passing.
- [ ] Test coverage >= 85%.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Pops with low morale gain `SilentStrike` and produce 0 output while still consuming resources.

## 7. Technical Guidance
- Implement this in `src/layer1/pops/strike.rs`.
- Ensure it hooks into the existing production pipeline so the multiplier actually affects building output.

## 8. Questions
*Builder: add questions here if spec is unclear.*
