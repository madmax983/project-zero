# Chronal Displacement (Spec 796)

## 1. Overview
Time is not a constant; sometimes you borrow from the future, but you always have to pay it back. You can overcharge your colony's production by initiating a "Chronal Shift," drastically speeding up local Layer 1 time relative to Layer 2/3. However, this incurs a "Chronal Debt." Eventually, the area will suffer a "Chronal Stutter," grinding to a halt for an equal amount of time while the rest of the universe catches up.

This creates tension: the immense, immediate benefit of borrowing time vs. the terrifying vulnerability of paying it back when you least expect it.

## 2. Dependencies
- `src/simulation.rs` (Simulation time delta)
- `src/layer1/production.rs` (Production speed logic)
- `src/layer1/physics.rs` (Chronal states)

## 3. RED Phase: Tests First

```rust
// tests/layer1/test_chronal_displacement.rs

use bevy::prelude::*;
use scale::simulation::SimulationTime;
use scale::layer1::physics::{ChronalShiftEvent, ChronalDebt, ChronalStutter};
use scale::layer1::production::ProductionSpeed;

#[test]
fn test_chronal_shift_increases_speed_and_accrues_debt() {
    let mut app = App::new();
    // ... setup ...

    let factory = app.world_mut().spawn(ProductionSpeed { multiplier: 1.0 }).id();
    app.world_mut().insert_resource(ChronalDebt(0));

    // Act
    app.world_mut().resource_mut::<Events<ChronalShiftEvent>>().send(ChronalShiftEvent { duration: 10 });
    app.update();

    // Assert: Speed increased and debt accrued
    let speed = app.world().get::<ProductionSpeed>(factory).unwrap();
    assert!(speed.multiplier > 1.0);
    assert_eq!(app.world().resource::<ChronalDebt>().0, 10);
}

#[test]
fn test_chronal_debt_triggers_stutter() {
    let mut app = App::new();
    // ... setup ...

    let factory = app.world_mut().spawn(ProductionSpeed { multiplier: 1.0 }).id();
    app.world_mut().insert_resource(ChronalDebt(100)); // Large debt

    // Act: Advance time enough to trigger stutter randomly
    for _ in 0..100 {
        app.update();
    }

    // Assert: A Stutter is active, slowing things down
    let stutter = app.world().get_resource::<ChronalStutter>();
    assert!(stutter.is_some());
    let speed = app.world().get::<ProductionSpeed>(factory).unwrap();
    assert!(speed.multiplier < 1.0);
}

#[test]
fn test_stutter_pays_off_debt() {
    let mut app = App::new();
    // ... setup ...

    app.world_mut().insert_resource(ChronalDebt(10));
    app.world_mut().insert_resource(ChronalStutter { remaining: 10 });

    app.update();

    // Assert: Debt is reduced
    let debt = app.world().resource::<ChronalDebt>();
    assert!(debt.0 < 10);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/chronal_displacement.rs

use bevy::prelude::*;
use crate::layer1::production::ProductionSpeed;

#[derive(Event)]
pub struct ChronalShiftEvent {
    pub duration: u32,
}

#[derive(Resource, Default)]
pub struct ChronalDebt(pub u32);

#[derive(Resource)]
pub struct ChronalStutter {
    pub remaining: u32,
}

pub fn handle_chronal_shift(
    mut events: EventReader<ChronalShiftEvent>,
    mut speeds: Query<&mut ProductionSpeed>,
    mut debt: ResMut<ChronalDebt>,
) {
    for event in events.read() {
        debt.0 += event.duration;
        for mut speed in speeds.iter_mut() {
            speed.multiplier = 3.0; // Huge boost
        }
    }
}

pub fn check_debt_stutter(
    mut commands: Commands,
    debt: Res<ChronalDebt>,
    stutter: Option<Res<ChronalStutter>>,
) {
    // Only trigger if not already stuttering, with chance based on debt
    if stutter.is_none() && debt.0 > 0 {
        let trigger_chance = (debt.0 as f32) / 100.0;
        // Simple mock random for now
        if rand::random::<f32>() < trigger_chance {
            commands.insert_resource(ChronalStutter { remaining: debt.0 });
        }
    }
}

pub fn apply_stutter(
    mut commands: Commands,
    mut stutter: Option<ResMut<ChronalStutter>>,
    mut debt: ResMut<ChronalDebt>,
    mut speeds: Query<&mut ProductionSpeed>,
) {
    if let Some(mut stutter_res) = stutter {
        if stutter_res.remaining > 0 {
            stutter_res.remaining -= 1;
            if debt.0 > 0 {
                debt.0 -= 1;
            }
            for mut speed in speeds.iter_mut() {
                speed.multiplier = 0.0; // Total freeze
            }
        } else {
            commands.remove_resource::<ChronalStutter>();
            for mut speed in speeds.iter_mut() {
                speed.multiplier = 1.0; // Back to normal
            }
        }
    }
}

pub struct ChronalDisplacementPlugin;
impl Plugin for ChronalDisplacementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChronalDebt>()
           .add_event::<ChronalShiftEvent>()
           .add_systems(Update, (
               handle_chronal_shift,
               check_debt_stutter,
               apply_stutter,
           ));
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**:
    - Tie the `ProductionSpeed` modifier directly into the global Simulation Time delta (e.g., `Time<Virtual>`) instead of manually overriding `ProductionSpeed` components.
    - Make sure `ChronalShiftEvent` has a localized radius (a building or sector) instead of boosting the whole map.
- **Code Smells**: Hardcoded 3.0 speed boost and 0.0 freeze values. These should be tunable.
- **Performance**: N/A, very simple state tracking.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Overcharging increases production multiplier and accrues `ChronalDebt`
- [ ] Stutter randomly triggers when debt is high, freezing production
- [ ] Stutter pays off the debt over time

## 7. Technical Guidance
- **Integration Points**: Register `ChronalDisplacementPlugin` in `simulation.rs`. Tie the shift/stutter modifiers into Bevy's time scaling if possible so that movement and metabolism are also affected, not just production.
- **Gotchas**: Don't accidentally stutter the entire game's UI layer. Only Layer 1 entities should be frozen or sped up.

## 8. Questions
*Builder: add questions here if spec is unclear.*
