# 1017: Legacy Code

## 1. Overview
Over time, the colony's "Operating System" accumulates "Bloat". Computing tasks (Research, Turret Targeting, Auto-Doors) gain latency. "Reformatting" clears Bloat but requires a total system shutdown (0 power/control) for a duration. This creates tension between operating with high latency (e.g., defense grid lag causing turrets to fire too late) vs the extreme vulnerability of a complete reboot.

## 2. Dependencies
- Layer 1 `Colony` or `ComputerCore` entity.
- Layer 1 `Power` system (for shutdowns).
- Layer 1 `Defense` (Turret aiming/firing).
- Layer 1 `Research` system.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::colony::ComputerCore;
    use crate::layer1::defense::Turret;
    use crate::simulation::SimulationTime;

    #[test]
    fn test_bloat_accumulation_increases_turret_latency() {
        let mut app = App::new();
        app.insert_resource(SimulationTime { ticks: 0 });
        app.add_systems(Update, (accumulate_bloat_system, apply_latency_system).chain());

        // Setup Core
        app.world_mut().spawn((
            ComputerCore,
            SystemBloat { amount: 0.0, accumulation_rate: 1.0 },
        ));

        // Setup Turret
        let turret = app.world_mut().spawn((
            Turret,
            ActionLatency { delay_ticks: 0 },
        )).id();

        // Simulate 100 ticks passing
        app.world_mut().resource_mut::<SimulationTime>().ticks = 100;
        app.update();

        let latency = app.world().get::<ActionLatency>(turret).unwrap();
        assert!(latency.delay_ticks > 0, "Turret latency should increase as System Bloat accumulates.");
    }

    #[test]
    fn test_reformatting_clears_bloat_but_disables_core() {
        let mut app = App::new();
        app.add_event::<ReformatCommand>();
        app.add_systems(Update, process_reformat_system);

        let core = app.world_mut().spawn((
            ComputerCore,
            SystemBloat { amount: 100.0, accumulation_rate: 1.0 },
            CoreStatus::Online,
        )).id();

        app.world_mut().resource_mut::<Events<ReformatCommand>>().send(ReformatCommand {
            core_entity: core,
        });

        app.update();

        let bloat = app.world().get::<SystemBloat>(core).unwrap();
        let status = app.world().get::<CoreStatus>(core).unwrap();

        assert_eq!(bloat.amount, 0.0, "Reformatting should clear all bloat.");
        assert_eq!(*status, CoreStatus::OfflineRebooting, "Reformatting must take the core offline.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/legacy_code.rs
use bevy::prelude::*;
use crate::layer1::colony::ComputerCore;
use crate::layer1::defense::Turret;

#[derive(Component)]
pub struct SystemBloat {
    pub amount: f32,
    pub accumulation_rate: f32,
}

#[derive(Component)]
pub struct ActionLatency {
    pub delay_ticks: u32,
}

#[derive(Component, PartialEq, Debug)]
pub enum CoreStatus {
    Online,
    OfflineRebooting,
}

#[derive(Event)]
pub struct ReformatCommand {
    pub core_entity: Entity,
}

pub fn accumulate_bloat_system(
    mut query: Query<(&mut SystemBloat, &CoreStatus)>,
) {
    for (mut bloat, status) in query.iter_mut() {
        if *status == CoreStatus::Online {
            bloat.amount += bloat.accumulation_rate; // Arbitrary MVP increase per tick
        }
    }
}

pub fn apply_latency_system(
    core_query: Query<&SystemBloat, With<ComputerCore>>,
    mut target_query: Query<&mut ActionLatency, With<Turret>>,
) {
    // Assuming one main core for MVP
    if let Ok(bloat) = core_query.get_single() {
        let added_delay = (bloat.amount / 10.0) as u32; // Every 10 bloat = 1 tick delay

        for mut latency in target_query.iter_mut() {
            latency.delay_ticks = added_delay;
        }
    }
}

pub fn process_reformat_system(
    mut events: EventReader<ReformatCommand>,
    mut query: Query<(&mut SystemBloat, &mut CoreStatus)>,
) {
    for event in events.read() {
        if let Ok((mut bloat, mut status)) = query.get_mut(event.core_entity) {
            bloat.amount = 0.0;
            *status = CoreStatus::OfflineRebooting;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Action Execution Queue:** Turrets and other automated systems actually need to *respect* the `ActionLatency`. Currently, we just set a variable. You'll need a command queue where actions are pushed, wait for `delay_ticks`, and then pop to execute.
- **Reboot Timer:** `CoreStatus::OfflineRebooting` is permanent in the MVP. We need a timer system to bring it back `Online` after a set duration (e.g., 60 seconds).
- **Power Grid Impact:** When the core goes `OfflineRebooting`, it should ideally sever the main power grid connection, plunging the colony into darkness and turning off life support.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_bloat_accumulation_increases_turret_latency` passes.
- [ ] Test `test_reformatting_clears_bloat_but_disables_core` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- If `SimulationTime` drives updates, `accumulate_bloat_system` should probably check delta-time or tick changes rather than incrementing blindly every frame.
- The single-core assumption (`core_query.get_single()`) works for MVP but might fail if the player can build multiple server racks.

## 8. Questions
*Builder: add questions here if spec is unclear.*
