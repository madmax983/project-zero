# 690 - Salvage-Cult Engineers

## 1. Overview
**Layer:** 1
**Fantasy:** Mechanics who begin worshipping broken machinery instead of fixing it, spreading a techno-religious memetic hazard.
**Mechanic:** If a complex machine (like a power reactor or atmosphere processor) remains in a state of disrepair for a long time, the Pops assigned to fix it have a chance to form a "Salvage Cult." They stop trying to repair the machine and instead start decorating it, protecting it, and preaching about the "purity of the broken state." This belief acts as a memetic virus, spreading to other engineers.

## 2. Dependencies
- `Pop` component (Layer 1)
- `Machine` or `Workstation` component with durability/repair states
- `RepairTask` assignment logic
- `Cultist` or `Belief` memetic spreading system

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_engineer_becomes_cultist_after_long_repair() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, evaluate_salvage_cult_formation);

        // A machine that's been broken for 100 cycles
        let machine = app.world_mut().spawn(Machine {
            health: 10.0,
            cycles_broken: 100, // Very long
            max_health: 100.0,
        }).id();

        // An engineer assigned to fix it
        let engineer = app.world_mut().spawn((
            Pop,
            RepairTask { target: machine },
        )).id();

        // Act
        app.update();

        // Assert
        let cultist = app.world().get::<SalvageCultist>(engineer);
        assert!(cultist.is_some(), "Engineer should become a Salvage Cultist after prolonged exposure to a broken machine");
    }

    #[test]
    fn test_cultist_stops_repairing() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_repair_tasks);

        let machine = app.world_mut().spawn(Machine {
            health: 10.0,
            cycles_broken: 100,
            max_health: 100.0,
        }).id();

        let cultist_engineer = app.world_mut().spawn((
            Pop,
            RepairTask { target: machine },
            SalvageCultist, // Already infected
        )).id();

        // Act
        app.update();

        // Assert
        let m = app.world().get::<Machine>(machine).unwrap();
        assert_eq!(m.health, 10.0, "A Salvage Cultist should NOT perform repairs on the machine");
    }

    #[test]
    fn test_cultist_spreads_belief_to_nearby_engineers() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, spread_cult_belief);

        let machine = app.world_mut().spawn((Machine {
            health: 10.0,
            cycles_broken: 100,
            max_health: 100.0,
        }, Transform::from_xyz(0.0, 0.0, 0.0))).id();

        let cultist = app.world_mut().spawn((
            Pop,
            RepairTask { target: machine },
            SalvageCultist,
            Transform::from_xyz(1.0, 0.0, 0.0), // Close to machine
        )).id();

        let innocent_engineer = app.world_mut().spawn((
            Pop,
            RepairTask { target: machine }, // Also assigned to machine
            Transform::from_xyz(2.0, 0.0, 0.0), // Close to cultist
        )).id();

        // Act
        app.update();

        // Assert
        let infected = app.world().get::<SalvageCultist>(innocent_engineer);
        assert!(infected.is_some(), "Cult belief should spread to nearby engineers working on the same machine");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Machine {
    pub health: f32,
    pub max_health: f32,
    pub cycles_broken: u32,
}

#[derive(Component)]
pub struct RepairTask {
    pub target: Entity,
}

#[derive(Component)]
pub struct SalvageCultist;

pub fn evaluate_salvage_cult_formation(
    mut commands: Commands,
    query_engineers: Query<(Entity, &RepairTask), Without<SalvageCultist>>,
    query_machines: Query<&Machine>,
) {
    for (entity, task) in query_engineers.iter() {
        if let Ok(machine) = query_machines.get(task.target) {
            // If the machine has been broken for over 50 cycles
            if machine.health < machine.max_health * 0.5 && machine.cycles_broken > 50 {
                // There's a high chance to snap and form a cult
                // In RED phase minimal, we just do it 100% of the time
                commands.entity(entity).insert(SalvageCultist);
            }
        }
    }
}

pub fn process_repair_tasks(
    query_engineers: Query<(&RepairTask, Option<&SalvageCultist>)>,
    mut query_machines: Query<&mut Machine>,
) {
    for (task, cultist) in query_engineers.iter() {
        if cultist.is_none() {
            // Normal engineer performs repair
            if let Ok(mut machine) = query_machines.get_mut(task.target) {
                machine.health += 5.0; // Flat repair rate
                machine.health = machine.health.min(machine.max_health);
            }
        }
    }
}

pub fn spread_cult_belief(
    mut commands: Commands,
    query_cultists: Query<(&Transform, &RepairTask), With<SalvageCultist>>,
    query_innocents: Query<(Entity, &Transform, &RepairTask), (With<Pop>, Without<SalvageCultist>)>,
) {
    let spread_radius = 10.0;

    for (cult_transform, cult_task) in query_cultists.iter() {
        for (innocent_entity, innocent_transform, innocent_task) in query_innocents.iter() {
            // If they are near each other and working on the same broken machine
            if cult_task.target == innocent_task.target {
                let distance = cult_transform.translation.distance(innocent_transform.translation);
                if distance < spread_radius {
                    commands.entity(innocent_entity).insert(SalvageCultist);
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **RNG/Probability**: Cult formation and spreading shouldn't be 100% guaranteed. Introduce RNG based on the Pop's `Morale`, `Education` (lower = more susceptible), or `Traits`.
- **Cult Decor**: Cultists should start adding `Shrine` or `Decor` components to the broken machine, visually changing it and actively making it *harder* for non-cultists to repair it (e.g., adding an "Obstructed" component).
- **Player Notification**: Fire a `CultSpottedEvent` when the first cultist appears so the player is aware their engineering team has stopped working.
- **Machine State Tracking**: `cycles_broken` implies a tick/timer logic. Ensure there's a system actually incrementing this value on machines that are damaged.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Engineers assigned to long-broken machines reliably become `SalvageCultist`s.
- [ ] Cultists cease repairing the machine but retain the assignment (blocking others).
- [ ] Cultists reliably spread the `SalvageCultist` component to nearby non-cultist engineers.

## 7. Technical Guidance
- **Memetic Systems Integration**: If the project already has a generic memetic or idea-spreading system (e.g., rumor web), hook into that rather than building a custom distance-based `spread_cult_belief` system.
- **Job Allocation**: Since cultists keep the `RepairTask` but do no work, they effectively gridlock the repair queue. This is intentional. The player will need to manually reassign or fire them.
- **Performance**: The distance check in `spread_cult_belief` should only run periodically (e.g., every few seconds), not every frame.

## 8. Questions
*Builder: add questions here if spec is unclear.*
