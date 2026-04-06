# 814: Radio Nostalgia

## 1. Overview
The stars are a time machine. The news from home arrives 50 years late.
Your Comms Console picks up broadcasts from the Homeworld, delayed by light-years. "New" music or political news shifts Colony Ethics or Moods. The news might be about a war that ended before you were born.

The colony receives a "Victory" broadcast from the Empire. Morale soars. Two years later, a refugee ship arrives saying the Empire actually fell 20 years ago. The "Victory" was propaganda. Do you censor the news (Control) or let it play (Morale/Chaos)?

## 2. Dependencies
- `010-chronicle-system.md` for historical recording of the broadcasts.
- `031-pop-morale.md` for mood shifts.
- `378-comms-relay.md` or similar for the Comms Console building/functionality.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_radio_nostalgia_broadcast_received() {
    // Arrange: Setup world with a functional Comms Console
    let mut app = setup_test_app();
    let colony_entity = spawn_colony(&mut app.world);
    let console_entity = spawn_comms_console(&mut app.world, colony_entity);

    // Act: Advance simulation time to trigger a delayed broadcast arrival
    let initial_morale = get_colony_morale(&app.world, colony_entity);
    trigger_delayed_broadcast(&mut app.world, colony_entity, BroadcastType::Victory);
    app.update();

    // Assert: Verify a chronicle event is logged and morale increases
    let events = get_chronicle_events(&app.world);
    assert!(events.iter().any(|e| matches!(e, ChronicleEvent::BroadcastReceived { broadcast_type: BroadcastType::Victory })));
    let new_morale = get_colony_morale(&app.world, colony_entity);
    assert!(new_morale > initial_morale, "Victory broadcast should increase morale");
}

#[test]
fn test_radio_nostalgia_truth_revealed() {
    // Arrange: Setup world that received a Victory broadcast previously
    let mut app = setup_test_app();
    let colony_entity = spawn_colony(&mut app.world);
    trigger_delayed_broadcast(&mut app.world, colony_entity, BroadcastType::Victory);
    app.update();

    // Act: A refugee ship arrives revealing the Victory was propaganda
    let initial_morale = get_colony_morale(&app.world, colony_entity);
    trigger_refugee_arrival_with_truth(&mut app.world, colony_entity);
    app.update();

    // Assert: Morale plummets due to the truth
    let new_morale = get_colony_morale(&app.world, colony_entity);
    assert!(new_morale < initial_morale, "Truth revealing propaganda should decrease morale significantly");
}

#[test]
fn test_radio_nostalgia_censorship() {
    // Arrange: Setup world with censorship policy active
    let mut app = setup_test_app();
    let colony_entity = spawn_colony(&mut app.world);
    set_colony_policy(&mut app.world, colony_entity, Policy::CensorBroadcasts);

    // Act: A negative broadcast arrives
    let initial_morale = get_colony_morale(&app.world, colony_entity);
    trigger_delayed_broadcast(&mut app.world, colony_entity, BroadcastType::Defeat);
    app.update();

    // Assert: Morale is unaffected (or drops slightly due to censorship awareness)
    let new_morale = get_colony_morale(&app.world, colony_entity);
    assert_eq!(new_morale, initial_morale, "Censored broadcasts should not immediately drop morale");
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Core components and events for radio broadcasts
#[derive(Component)]
pub struct CommsConsole;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BroadcastType {
    Victory,
    Defeat,
    Propaganda,
}

#[derive(Event)]
pub struct BroadcastReceivedEvent {
    pub colony: Entity,
    pub broadcast_type: BroadcastType,
}

pub fn handle_broadcasts_system(
    mut events: EventReader<BroadcastReceivedEvent>,
    mut morale_query: Query<&mut ColonyMorale>,
) {
    for event in events.read() {
        if let Ok(mut morale) = morale_query.get_mut(event.colony) {
            match event.broadcast_type {
                BroadcastType::Victory => morale.value += 10.0,
                BroadcastType::Defeat => morale.value -= 15.0,
                _ => {}
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration with Chronicle:** Ensure every major broadcast is properly recorded in the Chronicle system to maintain the colony's historical context.
- **Ethics Shift:** Rather than just a flat morale buff/debuff, the broadcasts should slowly shift the colony's civic ethics over time.
- **Censorship Tension:** The decision to censor should generate its own unique resource tension (Admin cost) and a slight persistent "Distrust" penalty to morale, balancing the safety of ignorance against the cost of control.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Delayed broadcasts alter morale based on their type, and truth revelations cause severe morale drops if the original broadcast was a lie.

## 7. Technical Guidance
- **Component Structuring:** Consider placing the active broadcasts in a resource or component attached to the `CommsConsole` to easily iterate through active effects.
- **Event Handling:** Use Bevy's event system to trigger the `BroadcastReceivedEvent` when the delayed timer expires. Ensure the timers are affected by the `SimulationTime`.
- **Chronicle Integration:** Use the existing `ChronicleSystem` to log the arrival of these broadcasts.

## 8. Questions
*Builder: add questions here if spec is unclear.*
