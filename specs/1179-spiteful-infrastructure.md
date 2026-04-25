# 1179: Spiteful Infrastructure

## 1. Overview
Smart infrastructure (like autodoors, auto-chefs, or transport belts) track user ratings. If a colonist routinely hits, kicks, or poorly maintains a machine, the machine develops "Spite." It will subtly inconvenience that specific Pop—doors open a half-second too late, meals are always slightly burnt, elevators stop on the wrong floor.

## 2. Dependencies
- Layer 1 `Building` system.
- `Pop` entity system and interactions with buildings.
- A relationship or tracking system linking Pops to specific Buildings.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_poor_maintenance_develops_spite() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, machine_maintenance_system);

    let pop = app.world_mut().spawn((Pop, PoorMaintainer)).id();
    let machine = app.world_mut().spawn(Building).id();

    // Spawn an interaction event where the Pop poorly maintains the machine
    app.add_event::<MachineInteractionEvent>();
    app.world_mut().send_event(MachineInteractionEvent {
        pop,
        machine,
        interaction_quality: -5.0,
    });

    // Act
    app.update();

    // Assert: The machine should develop spite against the Pop
    let spite = app.world().get::<SpiteNetwork>(machine);
    assert!(spite.is_some(), "Machine should have a SpiteNetwork");
    assert!(spite.unwrap().has_grudge(pop), "Machine should hold a grudge against the specific pop");
}

#[test]
fn test_spite_causes_inconvenience() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, spite_inconvenience_system);

    let pop = app.world_mut().spawn(Pop).id();

    let mut spite_network = SpiteNetwork::default();
    spite_network.add_grudge(pop, 10.0);

    let machine = app.world_mut().spawn((Building, spite_network)).id();

    // Act
    // Simulate pop interacting with the spiteful machine
    app.add_event::<MachineUseEvent>();
    app.world_mut().send_event(MachineUseEvent { pop, machine });
    app.update();

    // Assert: The pop receives a minor inconvenience debuff
    assert!(app.world().get::<Inconvenienced>(pop).is_some(), "Pop should be inconvenienced by the spiteful machine");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct PoorMaintainer;

#[derive(Component)]
pub struct Building;

#[derive(Event)]
pub struct MachineInteractionEvent {
    pub pop: Entity,
    pub machine: Entity,
    pub interaction_quality: f32,
}

#[derive(Event)]
pub struct MachineUseEvent {
    pub pop: Entity,
    pub machine: Entity,
}

#[derive(Component, Default)]
pub struct SpiteNetwork {
    grudges: HashMap<Entity, f32>,
}

impl SpiteNetwork {
    pub fn add_grudge(&mut self, pop: Entity, amount: f32) {
        *self.grudges.entry(pop).or_insert(0.0) += amount;
    }

    pub fn has_grudge(&self, pop: Entity) -> bool {
        self.grudges.get(&pop).copied().unwrap_or(0.0) > 5.0 // Arbitrary threshold
    }
}

#[derive(Component)]
pub struct Inconvenienced;

pub fn machine_maintenance_system(
    mut commands: Commands,
    mut events: EventReader<MachineInteractionEvent>,
    mut machines: Query<Option<&mut SpiteNetwork>, With<Building>>,
) {
    for event in events.read() {
        if event.interaction_quality < 0.0 {
            if let Ok(spite_opt) = machines.get_mut(event.machine) {
                if let Some(mut spite) = spite_opt {
                    spite.add_grudge(event.pop, -event.interaction_quality);
                } else {
                    let mut new_spite = SpiteNetwork::default();
                    new_spite.add_grudge(event.pop, -event.interaction_quality);
                    commands.entity(event.machine).insert(new_spite);
                }
            }
        }
    }
}

pub fn spite_inconvenience_system(
    mut commands: Commands,
    mut events: EventReader<MachineUseEvent>,
    machines: Query<&SpiteNetwork, With<Building>>,
) {
    for event in events.read() {
        if let Ok(spite) = machines.get(event.machine) {
            if spite.has_grudge(event.pop) {
                commands.entity(event.pop).insert(Inconvenienced);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Broadening**: `MachineInteractionEvent` could be merged with existing interaction or work events.
- **Grudge Decay**: Grudges should decay over time if the Pop performs "Machine Empathy" training or stops kicking it.
- **Inconvenience Effects**: `Inconvenienced` is a placeholder. It should map to actual game effects: movement speed penalty (door delay), lower output quality (burnt food), or slight mood reduction.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Machines track grudges against specific Pops and apply negative effects when used by those Pops.

## 7. Technical Guidance
- Ensure `SpiteNetwork` memory is properly serialized/deserialized if persisting across saves.
- Be careful with `HashMap<Entity, f32>` to clean up entries when Pops die. Hook into `PopDied` events to remove dead Pops from all `SpiteNetwork`s.

## 8. Questions
*Builder: add questions here if spec is unclear.*
