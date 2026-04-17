# 1073 - Benevolent Malfunctions

## 1. Overview
Machines and buildings can develop "Positive Glitches" (e.g., bypassing safety limiters for +Speed) coupled with "Quirks" (Noise, Heat, inability to stop). Repairing the machine "fixes" it back to standard (lower) stats. This creates tension between keeping a dangerous but highly productive machine versus a safe, standard one.

## 2. Dependencies
- Building entity and components (`src/layer1/building.rs`)
- Production/WorkEfficiency logic (`src/layer1/production.rs` or similar)
- Maintenance/Repair logic (`src/layer1/maintenance.rs` or similar)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_benevolent_malfunction_increases_output() {
        let mut app = App::new();
        app.add_systems(Update, apply_malfunction_effects);

        let building = app.world_mut().spawn((
            Building,
            WorkEfficiency { value: 1.0 },
            BenevolentMalfunction {
                efficiency_bonus: 0.5,
                quirk: MalfunctionQuirk::ExcessHeat,
            },
        )).id();

        app.update();

        // Efficiency should be increased by the malfunction
        let efficiency = app.world().get::<WorkEfficiency>(building).unwrap();
        assert_eq!(efficiency.value, 1.5);
    }

    #[test]
    fn test_repairing_removes_malfunction() {
        let mut app = App::new();
        app.add_systems(Update, (apply_malfunction_effects, process_repairs).chain());

        let building = app.world_mut().spawn((
            Building,
            WorkEfficiency { value: 1.0 },
            BenevolentMalfunction {
                efficiency_bonus: 0.5,
                quirk: MalfunctionQuirk::LoudNoise,
            },
            RepairJobTarget, // A pop is repairing this
        )).id();

        app.update();

        // The malfunction should be removed after repair
        assert!(app.world().get::<BenevolentMalfunction>(building).is_none());

        // Efficiency returns to normal
        let efficiency = app.world().get::<WorkEfficiency>(building).unwrap();
        assert_eq!(efficiency.value, 1.0);
    }

    #[test]
    fn test_malfunction_quirk_applies_penalty() {
        let mut app = App::new();
        app.add_systems(Update, apply_malfunction_quirks);

        let building = app.world_mut().spawn((
            Building,
            BenevolentMalfunction {
                efficiency_bonus: 0.5,
                quirk: MalfunctionQuirk::ExcessHeat,
            },
            HeatEmitter { amount: 10.0 },
        )).id();

        app.update();

        // The quirk should increase heat emission
        let heat = app.world().get::<HeatEmitter>(building).unwrap();
        assert!(heat.amount > 10.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct BenevolentMalfunction {
    pub efficiency_bonus: f32,
    pub quirk: MalfunctionQuirk,
}

#[derive(Clone, PartialEq, Eq)]
pub enum MalfunctionQuirk {
    ExcessHeat,
    LoudNoise,
    Unstoppable,
}

// Dummy components for testing context
#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct WorkEfficiency {
    pub value: f32,
}

#[derive(Component)]
pub struct HeatEmitter {
    pub amount: f32,
}

#[derive(Component)]
pub struct RepairJobTarget;

pub fn apply_malfunction_effects(
    mut query: Query<(&BenevolentMalfunction, &mut WorkEfficiency), Changed<BenevolentMalfunction>>,
) {
    for (malfunction, mut efficiency) in query.iter_mut() {
        // Apply the bonus once when the component is added or changed.
        // A more robust implementation would use a modifier system.
        efficiency.value += malfunction.efficiency_bonus;
    }
}

pub fn apply_malfunction_quirks(
    mut query: Query<(&BenevolentMalfunction, &mut HeatEmitter)>,
) {
    for (malfunction, mut heat) in query.iter_mut() {
        if malfunction.quirk == MalfunctionQuirk::ExcessHeat {
            // Apply excess heat penalty
            heat.amount += 5.0;
        }
    }
}

pub fn process_repairs(
    mut commands: Commands,
    mut query: Query<(Entity, &mut WorkEfficiency, &BenevolentMalfunction), With<RepairJobTarget>>,
) {
    for (entity, mut efficiency, malfunction) in query.iter_mut() {
        // Revert the efficiency bonus
        efficiency.value -= malfunction.efficiency_bonus;

        // Remove the malfunction and the repair job target
        commands.entity(entity).remove::<BenevolentMalfunction>();
        commands.entity(entity).remove::<RepairJobTarget>();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Modifier System Integration**: Instead of manually adding and subtracting `efficiency_bonus`, hook `BenevolentMalfunction` into the standard stat modifier pipeline so it dynamically recalculates.
- **Quirk Expansion**: Define a data-driven approach for Quirks so they can affect different building components (e.g., `AcousticEmitter`, `PowerConsumption`) without hardcoding them in match statements.
- **Player Choice**: Provide a UI toggle for "Do Not Repair" so the player can explicitly forbid engineers from fixing the benevolent malfunction.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for new code.
- [ ] Buildings with `BenevolentMalfunction` show increased production/efficiency.
- [ ] Standard repair jobs successfully remove the component and revert stats.
- [ ] Quirks (like ExcessHeat) apply their negative effects correctly.

## 7. Technical Guidance
- Ensure `apply_malfunction_effects` doesn't infinitely scale efficiency if called every frame. Use Bevy's change detection (`Added` or `Changed`) or a proper modifier list.
- When generating a malfunction event (e.g., via a random tick or wear-and-tear), ensure it only applies to buildings capable of production.

## 8. Questions
*Builder: add questions here if spec is unclear.*
