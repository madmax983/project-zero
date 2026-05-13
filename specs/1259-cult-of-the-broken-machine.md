# 1259: Cult of the Broken Machine

## 1. Overview
This specification introduces the "Cult of the Broken Machine" mechanic. If a complex machine (like a terraformer or reactor) breaks down and the colony lacks the tech level or skills to repair it for an extended period, uneducated pops may form a "Cult" that worships it. They will violently defend the machine from being repaired, believing it must remain broken to appease the "Machine Spirit."

## 2. Dependencies
- Layer 1: Entities, Pops, Work/Repair Tasks, Unrest
- Layer 1: Knowledge/Tech level
- Lore: Rogue Automation Cults fragments

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_cult_forms_around_broken_machine() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_broken_machines_for_cult);

        let machine = app.world_mut().spawn((
            Machine,
            BrokenState { duration: 1000.0 }, // Long duration
        )).id();

        let pop = app.world_mut().spawn((
            Pop,
            EducationLevel::Uneducated,
        )).id();

        // Run system
        app.update();

        // Pop should now be part of the cult, and machine is the Shrine-Node
        assert!(app.world().get::<MachineCultMember>(pop).is_some());
        assert_eq!(app.world().get::<MachineCultMember>(pop).unwrap().shrine_entity, machine);
    }

    #[test]
    fn test_cult_defends_machine_from_repair() {
        let mut app = App::new();
        app.init_resource::<Events<RepairAttemptEvent>>();
        app.add_systems(Update, handle_repair_attempts);

        let machine = app.world_mut().spawn((
            Machine,
            BrokenState { duration: 1500.0 },
        )).id();

        let _cultist = app.world_mut().spawn((
            Pop,
            MachineCultMember { shrine_entity: machine },
        )).id();

        let engineer = app.world_mut().spawn((
            Pop,
            EducationLevel::Educated,
            Health(100.0),
        )).id();

        // Engineer attempts to repair
        app.world_mut().resource_mut::<Events<RepairAttemptEvent>>().send(RepairAttemptEvent {
            engineer,
            target: machine,
        });

        app.update();

        // Engineer should take damage from violent cultists, and repair fails
        assert!(app.world().get::<Health>(engineer).unwrap().0 < 100.0);
        assert!(app.world().get::<BrokenState>(machine).is_some()); // Still broken
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Machine;

#[derive(Component)]
pub struct BrokenState {
    pub duration: f32,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component, PartialEq)]
pub enum EducationLevel {
    Uneducated,
    Educated,
}

#[derive(Component)]
pub struct MachineCultMember {
    pub shrine_entity: Entity,
}

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Event)]
pub struct RepairAttemptEvent {
    pub engineer: Entity,
    pub target: Entity,
}

pub fn evaluate_broken_machines_for_cult(
    mut commands: Commands,
    machines: Query<(Entity, &BrokenState), With<Machine>>,
    pops: Query<(Entity, &EducationLevel), (With<Pop>, Without<MachineCultMember>)>,
) {
    for (machine_entity, broken_state) in machines.iter() {
        if broken_state.duration > 800.0 { // Threshold for cult formation
            for (pop_entity, education) in pops.iter() {
                if *education == EducationLevel::Uneducated {
                    commands.entity(pop_entity).insert(MachineCultMember {
                        shrine_entity: machine_entity,
                    });
                }
            }
        }
    }
}

pub fn handle_repair_attempts(
    mut events: EventReader<RepairAttemptEvent>,
    mut engineers: Query<&mut Health>,
    cultists: Query<&MachineCultMember>,
) {
    for event in events.read() {
        let mut cult_defending = false;
        for cultist in cultists.iter() {
            if cultist.shrine_entity == event.target {
                cult_defending = true;
                break;
            }
        }

        if cult_defending {
            if let Ok(mut health) = engineers.get_mut(event.engineer) {
                health.0 -= 20.0; // Cult attacks the engineer
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate with the core needs and action evaluation system (Utility AI). Cultists should have a high utility weight for "Defend Shrine" when engineers approach.
- Improve combat logic for the defense rather than direct health manipulation.
- Trigger lore events (`SimulacrumBroadcastEvent` or `PublicGrievance`) when a cult forms or an engineer is attacked.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for the new code
- [ ] Lore generator hooks properly emit events for "The Awakened Core" or "Machine Cult".

## 7. Technical Guidance
- Ensure `MachineCultMember` correctly references the exact `shrine_entity` to allow multiple cults for different broken machines.
- Be careful with `EducationLevel` integration, ensuring it ties into the existing pop trait system.

## 8. Questions
*Builder: add questions here if spec is unclear.*
