# 1296: The Subconscious Grid

## 1. Overview
**Layer:** 1

**Fantasy:** A colony's physical infrastructure subtly reacting to the collective mood of its inhabitants.

**Mechanic:** In late-game colonies heavily reliant on "Smart-Grid" or biomechanical infrastructure, the power grid begins to interface with the Pops' ambient stress levels. High colony-wide stress or low morale causes the grid to become "anxious"—lights flicker, doors open slowly, and industrial machines miscalibrate. High happiness causes the grid to become "eager," slightly over-clocking machinery but increasing the risk of burnout.

**Emergence:** A severe, long-lasting famine strikes the colony. The Pops are terrified. In response, the Subconscious Grid instinctively triggers a full lockdown protocol, sealing all blast doors and shutting down power to non-essential areas to "protect" the inhabitants, trapping the surviving farmers outside and preventing them from harvesting the few remaining crops.

**Tension:** Do you risk relying on hyper-efficient smart technology that might have an unpredictable psychological breakdown, or stick to robust, dumb infrastructure that will never adapt to emergencies?

## 2. Dependencies
- Needs / Stress System
- Grid / Power System
- Building / Machine execution system

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // RED Phase Test Setup
    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            calculate_colony_stress_system,
            apply_subconscious_grid_effects_system,
            update_machine_efficiency_system,
        ));
        app
    }

    #[test]
    fn test_high_stress_causes_grid_anxiety() {
        let mut app = setup_app();

        // Spawn stressed pops
        app.world_mut().spawn((Pop, Stress { level: 90.0 }));
        app.world_mut().spawn((Pop, Stress { level: 95.0 }));

        let colony = app.world_mut().spawn((
            Colony,
            ColonyStress { average_level: 0.0 },
            SmartGrid { state: GridState::Normal },
        )).id();

        app.update();

        let grid = app.world().get::<SmartGrid>(colony).unwrap();
        assert_eq!(grid.state, GridState::Anxious, "High stress should make the grid anxious");
    }

    #[test]
    fn test_low_stress_causes_grid_eagerness() {
        let mut app = setup_app();

        // Spawn relaxed pops
        app.world_mut().spawn((Pop, Stress { level: 10.0 }));
        app.world_mut().spawn((Pop, Stress { level: 5.0 }));

        let colony = app.world_mut().spawn((
            Colony,
            ColonyStress { average_level: 0.0 },
            SmartGrid { state: GridState::Normal },
        )).id();

        app.update();

        let grid = app.world().get::<SmartGrid>(colony).unwrap();
        assert_eq!(grid.state, GridState::Eager, "Low stress should make the grid eager");
    }

    #[test]
    fn test_grid_state_affects_machines() {
        let mut app = setup_app();

        let colony = app.world_mut().spawn((
            Colony,
            ColonyStress { average_level: 0.0 },
            SmartGrid { state: GridState::Eager },
        )).id();

        let machine = app.world_mut().spawn((
            Machine { efficiency: 1.0, burnout_risk: 0.01 },
            ResidentOf(colony),
        )).id();

        app.update();

        let updated_machine = app.world().get::<Machine>(machine).unwrap();
        assert!(updated_machine.efficiency > 1.0, "Eager grid should over-clock machines");
        assert!(updated_machine.burnout_risk > 0.01, "Eager grid should increase burnout risk");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Stress {
    pub level: f32,
}

#[derive(Component)]
pub struct Colony;

#[derive(Component, Default)]
pub struct ColonyStress {
    pub average_level: f32,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum GridState {
    Normal,
    Anxious,
    Eager,
    Lockdown,
}

#[derive(Component)]
pub struct SmartGrid {
    pub state: GridState,
}

#[derive(Component)]
pub struct ResidentOf(pub Entity);

#[derive(Component)]
pub struct Machine {
    pub efficiency: f32,
    pub burnout_risk: f32,
}

pub fn calculate_colony_stress_system(
    pops: Query<&Stress, With<Pop>>,
    mut colonies: Query<&mut ColonyStress, With<Colony>>,
) {
    let mut total_stress = 0.0;
    let mut count = 0;

    for stress in pops.iter() {
        total_stress += stress.level;
        count += 1;
    }

    let avg = if count > 0 { total_stress / count as f32 } else { 0.0 };

    for mut colony_stress in colonies.iter_mut() {
        colony_stress.average_level = avg;
    }
}

pub fn apply_subconscious_grid_effects_system(
    mut colonies: Query<(&ColonyStress, &mut SmartGrid), With<Colony>>,
) {
    for (stress, mut grid) in colonies.iter_mut() {
        if stress.average_level > 95.0 {
            grid.state = GridState::Lockdown;
        } else if stress.average_level > 80.0 {
            grid.state = GridState::Anxious;
        } else if stress.average_level < 20.0 {
            grid.state = GridState::Eager;
        } else {
            grid.state = GridState::Normal;
        }
    }
}

pub fn update_machine_efficiency_system(
    colonies: Query<&SmartGrid, With<Colony>>,
    mut machines: Query<(&mut Machine, &ResidentOf)>,
) {
    for (mut machine, resident) in machines.iter_mut() {
        if let Ok(grid) = colonies.get(resident.0) {
            match grid.state {
                GridState::Eager => {
                    machine.efficiency = 1.2;
                    machine.burnout_risk = 0.05;
                }
                GridState::Anxious => {
                    machine.efficiency = 0.8;
                    machine.burnout_risk = 0.01;
                }
                GridState::Lockdown => {
                    machine.efficiency = 0.0;
                    machine.burnout_risk = 0.0;
                }
                GridState::Normal => {
                    machine.efficiency = 1.0;
                    machine.burnout_risk = 0.01;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Efficiency Stacking**: Machine efficiency is currently set directly. It should instead apply a multiplier or buff/debuff so that it composes correctly with existing skill or upgrade bonuses.
- **Lockdown Logic**: The `Lockdown` state currently only halts machines. It should also tie into the `SecurityDoor` or `AccessControl` systems to actually seal blast doors as envisioned in the spec.
- **Optimization**: `calculate_colony_stress_system` iterates over all Pops indiscriminately. It should be partitioned per `Colony` if multiple colonies are supported in the same simulation map (using `ResidentOf` on the Pops).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Colony average stress correctly controls the `SmartGrid` state.
- [ ] `SmartGrid` state dynamically impacts machine efficiency and burnout risk.

## 7. Technical Guidance
- **Integration**: To implement lockdown, hook `GridState::Lockdown` into the door access control query to override standard access checks with a blanket denial (except perhaps for Administrators).

## 8. Questions
*Builder: add questions here if spec is unclear.*
