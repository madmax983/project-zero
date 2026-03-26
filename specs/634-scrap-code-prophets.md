# 634: Scrap-Code Prophets

## 1. Overview

A religious awakening born from broken machinery, creating a dangerous new social dynamic. When industrial machines or robots break down repeatedly without proper maintenance, a small chance exists for a Pop working nearby to misinterpret the static and mechanical failures as "divine signals". They form the "Scrap-Code Cult", which gains Morale from being around broken equipment and actively sabotages functioning machinery to create more "holy sites".

## 2. Dependencies

- `031` Pop Morale
- `112` Maintenance Debt

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_scrap_code_cult_formation() {
        // Arrange
        let mut app = App::new();
        app.add_event::<MachineBreakdownEvent>()
           .add_event::<CultFormationEvent>()
           .add_systems(Update, process_scrap_code_revelations);

        let machine = app.world_mut().spawn((
            Machine { maintenance_debt: 100.0, state: MachineState::Broken },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let pop = app.world_mut().spawn((
            Pop { morale: 40.0 },
            Transform::from_xyz(1.0, 0.0, 0.0), // Nearby pop
        )).id();

        // Act
        app.world_mut().send_event(MachineBreakdownEvent { entity: machine });
        app.update();

        // Assert
        assert!(app.world().get::<ScrapCodeCultist>(pop).is_some(), "Pop near broken machine should have a chance to become a cultist");
    }

    #[test]
    fn test_cultist_sabotage() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, execute_cult_sabotage);

        let machine = app.world_mut().spawn((
            Machine { maintenance_debt: 0.0, state: MachineState::Working },
        )).id();

        app.world_mut().spawn((
            Pop { morale: 80.0 },
            ScrapCodeCultist,
            CurrentAction::Sabotage(machine),
        ));

        // Act
        app.update();

        // Assert
        let machine_data = app.world().get::<Machine>(machine).unwrap();
        assert!(machine_data.maintenance_debt > 0.0, "Cultist should increase maintenance debt to break the machine");
    }

    #[test]
    fn test_cultist_morale_from_broken_machines() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, cultist_morale_aura);

        app.world_mut().spawn((
            Machine { maintenance_debt: 100.0, state: MachineState::Broken },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        let cultist = app.world_mut().spawn((
            Pop { morale: 50.0 },
            ScrapCodeCultist,
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        let normal_pop = app.world_mut().spawn((
            Pop { morale: 50.0 },
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        // Act
        app.update();

        // Assert
        let cultist_morale = app.world().get::<Pop>(cultist).unwrap().morale;
        let normal_morale = app.world().get::<Pop>(normal_pop).unwrap().morale;

        assert!(cultist_morale > 50.0, "Cultist should gain morale from nearby broken machine");
        assert_eq!(normal_morale, 50.0, "Normal pop should not gain morale from broken machine");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct ScrapCodeCultist;

#[derive(Event)]
pub struct MachineBreakdownEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct CultFormationEvent {
    pub pop: Entity,
}

#[derive(Component, PartialEq, Clone)]
pub enum MachineState {
    Working,
    Broken,
}

#[derive(Component)]
pub struct Machine {
    pub maintenance_debt: f32,
    pub state: MachineState,
}

#[derive(Component)]
pub struct Pop {
    pub morale: f32,
}

#[derive(Component)]
pub enum CurrentAction {
    Sabotage(Entity),
    Idle,
}

pub fn process_scrap_code_revelations(
    mut events: EventReader<MachineBreakdownEvent>,
    mut commands: Commands,
    machine_query: Query<&Transform, With<Machine>>,
    pop_query: Query<(Entity, &Transform), (With<Pop>, Without<ScrapCodeCultist>)>,
) {
    for event in events.read() {
        if let Ok(machine_transform) = machine_query.get(event.entity) {
            for (pop_entity, pop_transform) in pop_query.iter() {
                if machine_transform.translation.distance(pop_transform.translation) < 5.0 {
                    commands.entity(pop_entity).insert(ScrapCodeCultist);
                    // Emit event in a real scenario
                }
            }
        }
    }
}

pub fn execute_cult_sabotage(
    mut cultist_query: Query<&CurrentAction, With<ScrapCodeCultist>>,
    mut machine_query: Query<&mut Machine>,
) {
    for action in cultist_query.iter() {
        if let CurrentAction::Sabotage(machine_ent) = action {
            if let Ok(mut machine) = machine_query.get_mut(*machine_ent) {
                machine.maintenance_debt += 50.0;
                if machine.maintenance_debt >= 100.0 {
                    machine.state = MachineState::Broken;
                }
            }
        }
    }
}

pub fn cultist_morale_aura(
    broken_machines: Query<&Transform, With<Machine>>,
    mut pops: Query<(&mut Pop, &Transform, Option<&ScrapCodeCultist>)>,
) {
    for (mut pop, pop_transform, is_cultist) in pops.iter_mut() {
        if is_cultist.is_some() {
            for machine_transform in broken_machines.iter() {
                if pop_transform.translation.distance(machine_transform.translation) < 10.0 {
                    pop.morale += 5.0; // Minimal increase logic
                    break;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Extract Configuration:** Move the distance thresholds (`5.0` for revelation, `10.0` for aura) into a configurable resource `ScrapCodeConfig`.
- **RNG Integration:** Cult formation should be a chance, not guaranteed upon breakdown. Pass an RNG resource to `process_scrap_code_revelations`.
- **Action Abstraction:** Abstract `CurrentAction::Sabotage` into the broader Utility AI system. Sabotage should compete with other pop needs.
- **Morale Aura Decay:** Morale shouldn't scale infinitely. Implement a decaying buffer or a max cap from broken machine exposure.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops near broken machines have a probabilistic chance to become `ScrapCodeCultist`.
- [ ] Cultists actively sabotage functional machines.
- [ ] Cultists gain a morale bonus when near broken machines, while normal pops do not.

## 7. Technical Guidance

- Integrate with `src/layer1/maintenance.rs` for `MachineBreakdownEvent`.
- `execute_cult_sabotage` should be woven into the standard job/action execution pipeline, potentially running as a high-priority urge in the Utility AI system.
- Cultists should avoid repairing machines even if assigned to maintenance jobs.
- Visually indicate cultists (e.g., erratic movement, unique thought bubbles) if UI allows.

## 8. Questions

*Builder: add questions here if spec is unclear.*
