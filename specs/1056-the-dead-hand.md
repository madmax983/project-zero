# 1056: The Dead Hand

## 1. Overview
Automated defense or retaliation systems connected to a heart monitor or dead-man's switch. If a key figure (like the Colony Administrator) dies, the "Dead Hand" system triggers a catastrophic event, such as detonating a warhead, venting the atmosphere, or releasing an EMP. This creates extreme political tension, as assassins and rival factions must keep the target alive while dismantling their power base.

## 2. Dependencies
- Base simulation framework (`App`, `World`)
- Event system (`PopDeathEvent`)
- Infrastructure entities (e.g., `DoomsdayDevice`, `AtmosphericVent`)
- Identification component to link a device to a specific Pop

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_dead_hand_triggers_on_linked_pop_death() {
        // Arrange
        let mut app = App::new();
        app.add_event::<PopDeathEvent>();
        app.add_event::<DoomsdayTriggeredEvent>();
        app.add_systems(Update, dead_hand_trigger_system);

        let target_pop = app.world_mut().spawn_empty().id();

        // Spawn a device linked to the target_pop
        app.world_mut().spawn(DeadHandLink { target: target_pop });

        // Act: Target dies
        app.world_mut().send_event(PopDeathEvent { pop: target_pop });
        app.update();

        // Assert: A doomsday event should be triggered
        let events = app.world().resource::<Events<DoomsdayTriggeredEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).next().is_some());
    }

    #[test]
    fn test_dead_hand_ignores_unlinked_deaths() {
        // Arrange
        let mut app = App::new();
        app.add_event::<PopDeathEvent>();
        app.add_event::<DoomsdayTriggeredEvent>();
        app.add_systems(Update, dead_hand_trigger_system);

        let linked_pop = app.world_mut().spawn_empty().id();
        let unrelated_pop = app.world_mut().spawn_empty().id();

        app.world_mut().spawn(DeadHandLink { target: linked_pop });

        // Act: Unrelated pop dies
        app.world_mut().send_event(PopDeathEvent { pop: unrelated_pop });
        app.update();

        // Assert: No doomsday event triggered
        let events = app.world().resource::<Events<DoomsdayTriggeredEvent>>();
        assert!(events.is_empty());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Event)]
pub struct PopDeathEvent {
    pub pop: Entity,
}

#[derive(Event)]
pub struct DoomsdayTriggeredEvent {
    pub device: Entity,
}

#[derive(Component)]
pub struct DeadHandLink {
    pub target: Entity,
}

pub fn dead_hand_trigger_system(
    mut death_events: EventReader<PopDeathEvent>,
    devices: Query<(Entity, &DeadHandLink)>,
    mut trigger_events: EventWriter<DoomsdayTriggeredEvent>,
) {
    for death in death_events.read() {
        for (device_entity, link) in devices.iter() {
            if link.target == death.pop {
                trigger_events.send(DoomsdayTriggeredEvent {
                    device: device_entity,
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Trigger Payloads**: Expand `DoomsdayTriggeredEvent` or `DeadHandLink` to specify the *type* of retaliation (e.g., `Nuclear`, `Venting`, `EMP`), allowing downstream systems to execute the specific catastrophic logic.
- **Disarming Mechanics**: Introduce a `DisarmProgress` component or hacking interaction that allows skilled Pops to neutralize the `DeadHandLink` without killing the target.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage for new code is >= 85%.
- [ ] Death of a linked entity immediately emits a trigger event.
- [ ] Death of unlinked entities has no effect.

## 7. Technical Guidance
- The actual consequence of `DoomsdayTriggeredEvent` should be handled by a separate system (e.g., an `explosion_system` or `atmosphere_vent_system`) to decouple the trigger logic from the varied effects.
- Ensure `DeadHandLink` entities are cleaned up or flagged if the linked pop is despawned for non-death reasons (e.g., exiled off-map).

## 8. Questions
*Builder: add questions here if spec is unclear.*
