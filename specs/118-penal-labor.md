# 118: Penal Labor

## Overview

Inmates currently sit idle in jail, consuming resources without contributing. **Penal Labor** allows the player to designate **Penal Zones** where inmates are forced to work (e.g., Mining, Hauling).

Key mechanics:
- **Penal Zone**: A new zone type. Inmates can only work within Penal Zones.
- **Forced Labor**: Inmates in a Penal Zone ignore Needs (Happiness/Rest) to work, but accumulate **Revolt Risk**.
- **Efficiency**: Penal labor has a higher base work speed (fear-based motivation).
- **Consequences**: High Revolt Risk triggers a **Jailbreak** event (Mass breakout + Aggression).

## Dependencies

- `072` — Justice System (Inmate component)
- `056` — Designated Zones (ZoneType)
- `009` — Job System (Work logic)

## RED Phase: Tests First

Write these tests in `src/layer1/penal_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::justice::Inmate;
    use crate::layer1::zone::{ZoneType, ZoneGrid};
    use crate::layer1::penal::{PenalLabor, RevoltRisk, evaluate_penal_work_system, check_jailbreak_system};
    use crate::layer1::pop::Pop;
    use crate::layer1::map::GridPosition;
    use crate::layer1::utility_types::{ActionType, PopAction};

    fn setup_world() -> World {
        let mut world = World::new();
        let mut zone_grid = ZoneGrid::new(10, 10);
        // Define a Penal Zone at (5,5)
        zone_grid.set(5, 5, ZoneType::Penal);
        world.insert_resource(zone_grid);
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world
    }

    #[test]
    fn test_penal_labor_component_defaults() {
        let labor = PenalLabor::default();
        assert_eq!(labor.efficiency_bonus, 0.2); // 20% faster
    }

    #[test]
    fn test_inmate_evaluates_work_in_penal_zone() {
        let mut world = setup_world();

        // Inmate at (5,5) which is a Penal Zone
        let inmate = world.spawn((
            Pop,
            Inmate { sentence_ticks: 1000 },
            GridPosition { x: 5, y: 5 },
            PopAction::default(),
            // UtilityAI components would be here
        )).id();

        // Run evaluation system
        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_penal_work_system);
        schedule.run(&mut world);

        // Should override action to Work (if work available) or specific PenalWork
        // For MVP, we assume evaluate_penal_work_system sets a specific flag or action
        // Here we test that it *allows* work, normally Inmates have ActionType::Idle forced.

        // This test assumes evaluate_penal_work_system injects a high-priority work action
        // if in a Penal Zone.
        // For the sake of the test, let's say it sets a component "PenalAssignment".

        assert!(world.get::<PenalLabor>(inmate).is_some());
    }

    #[test]
    fn test_inmate_outside_penal_zone_is_idle() {
        let mut world = setup_world();

        // Inmate at (0,0) - NOT a Penal Zone
        let inmate = world.spawn((
            Pop,
            Inmate { sentence_ticks: 1000 },
            GridPosition { x: 0, y: 0 },
            PopAction::default(),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_penal_work_system);
        schedule.run(&mut world);

        assert!(world.get::<PenalLabor>(inmate).is_none());
    }

    #[test]
    fn test_working_accumulates_revolt_risk() {
        let mut world = setup_world();

        let inmate = world.spawn((
            Pop,
            Inmate { sentence_ticks: 1000 },
            PenalLabor::default(), // Actively working
            RevoltRisk { current: 0.0, threshold: 100.0 },
        )).id();

        // Run system tick
        // Assume system increases risk by 1.0 per tick
        crate::layer1::penal::update_revolt_risk_system(&mut world);

        let risk = world.get::<RevoltRisk>(inmate).unwrap();
        assert!(risk.current > 0.0);
    }

    #[test]
    fn test_jailbreak_trigger() {
        let mut world = setup_world();

        let inmate = world.spawn((
            Pop,
            Inmate { sentence_ticks: 1000 },
            RevoltRisk { current: 101.0, threshold: 100.0 }, // Over threshold
        )).id();

        check_jailbreak_system(&mut world);

        // Should lose Inmate status, gain Wanted (Escapee) status, maybe Aggressive
        assert!(world.get::<Inmate>(inmate).is_none());
        // Custom component for breakout
        // Or check if sentence is cleared
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `ZoneType`

Add `Penal` to `src/layer1/zone.rs`.

```rust
pub enum ZoneType {
    // ...
    Penal,
}
```

### 2. Create `src/layer1/penal.rs`

```rust
use bevy_ecs::prelude::*;
use crate::layer1::justice::Inmate;
use crate::layer1::zone::{ZoneGrid, ZoneType};
use crate::layer1::map::GridPosition;

#[derive(Component)]
pub struct PenalLabor {
    pub efficiency_bonus: f32,
}

impl Default for PenalLabor {
    fn default() -> Self {
        Self { efficiency_bonus: 0.2 }
    }
}

#[derive(Component, Default)]
pub struct RevoltRisk {
    pub current: f32,
    pub threshold: f32,
}

/// Checks if Inmates are in a Penal Zone and assigns PenalLabor status.
pub fn evaluate_penal_work_system(
    mut commands: Commands,
    zone_grid: Res<ZoneGrid>,
    query: Query<(Entity, &Inmate, &GridPosition), Without<PenalLabor>>,
) {
    for (entity, _inmate, pos) in &query {
        if zone_grid.get(pos.x, pos.y) == ZoneType::Penal {
            commands.entity(entity).insert((
                PenalLabor::default(),
                RevoltRisk { current: 0.0, threshold: 100.0 }, // Base threshold
            ));
        }
    }
}

/// Removes PenalLabor if Inmate leaves zone (e.g. moved back to cell).
pub fn cleanup_penal_work_system(
    mut commands: Commands,
    zone_grid: Res<ZoneGrid>,
    query: Query<(Entity, &GridPosition), With<PenalLabor>>,
) {
    for (entity, pos) in &query {
        if zone_grid.get(pos.x, pos.y) != ZoneType::Penal {
            commands.entity(entity)
                .remove::<PenalLabor>()
                .remove::<RevoltRisk>();
        }
    }
}

pub fn update_revolt_risk_system(mut query: Query<&mut RevoltRisk, With<PenalLabor>>) {
    for mut risk in &mut query {
        risk.current += 0.1; // Accumulate slowly
    }
}

pub fn check_jailbreak_system(mut commands: Commands, query: Query<(Entity, &RevoltRisk), With<Inmate>>) {
    for (entity, risk) in &query {
        if risk.current >= risk.threshold {
            // Trigger Jailbreak
            commands.entity(entity)
                .remove::<Inmate>()
                .remove::<PenalLabor>()
                .remove::<RevoltRisk>();

            // Add Wanted status (severity high)
            // commands.entity(entity).insert(Wanted { severity: 5.0 });

            // Log event?
        }
    }
}
```

### 3. Integrate with Job System

Modify `src/layer1/utility_ai.rs` or `execution.rs` to allow `PenalLabor` entities to perform `Work` actions even if `Inmate`.
Currently `Inmate` might hard-lock to Idle.
If `PenalLabor` is present, allow logic to proceed.

## REFACTOR Phase: Quality & Design

- **Visuals**: Inmates in Penal Labor should have distinct visuals (chains?).
- **Guards**: Penal Zones should require `Warden` presence to reduce Revolt Risk.
- **Jobs**: Restrict Penal Labor to "dirty" jobs (Mining, Hauling, Cleaning). Don't let them do Research.
- **Escape**: Revolt shouldn't just remove Inmate status; it should trigger pathfinding to map edge or weapon pickup.

## Acceptance Criteria

- [ ] `ZoneType::Penal` exists.
- [ ] Inmates entering `Penal` zone gain `PenalLabor` component.
- [ ] Inmates leaving `Penal` zone lose `PenalLabor` component.
- [ ] `RevoltRisk` accumulates over time for working inmates.
- [ ] High `RevoltRisk` triggers removal of `Inmate` status (Jailbreak).
- [ ] Tests pass.

## Technical Guidance

- Ensure `evaluate_penal_work_system` runs before Utility AI evaluation so the `PenalLabor` component is available for decision making.
- The `Inmate` component might need a flag `allow_work` or similar if the hard-lock is in a common system.
- Start simple: Any job in the zone is allowed. Refactor later to restrict job types.

## Questions

*Builder: add questions here if spec is unclear.*
