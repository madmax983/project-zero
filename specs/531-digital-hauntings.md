# 531 Digital Hauntings

## 1. Overview
**Layer:** Cross-layer (Layer 1 -> Narrative)
**Fantasy:** The ghost in the machine. Your technology remembers the people who died using it.
**Mechanic:** When a Pop dies violently near an active console or high-tech structure, a "Data Fragment" of their final moments is permanently written into the machine's code. These "Haunted" machines function perfectly but occasionally broadcast the dead Pop's last thoughts or scream through the colony's comms network, causing massive Morale drops and Paranoia.
**Emergence:** Your primary Fusion Reactor is haunted by the chief engineer who burned to death fixing it during a pirate raid. Every time power usage peaks, the reactor screams their final words over the intercom. The colony is terrified, but you can't dismantle your only power source, so you just endure the constant, localized panic.
**Tension:** The immense cost of replacing critical infrastructure vs. the steady, psychological drain of keeping "cursed" technology running.

## 2. Dependencies
- `034 Pop Health and Damage` (Violent death events)
- `046 Notifications System` (To broadcast the haunting)
- `031 Pop Morale` (Stress/Paranoia impacts)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_violent_death_haunts_nearby_machine() {
    // Arrange
    let mut app = setup_world();
    let machine_pos = GridPosition { x: 5, y: 5 };

    let machine_id = app.world_mut().spawn((
        Building,
        BuildingType::FusionReactor,
        MachineSpirit::default(), // Indicates a complex machine
        machine_pos,
    )).id();

    let pop_id = app.world_mut().spawn((
        Pop,
        PopName("Engineer Bob".to_string()),
        machine_pos, // Died right next to it
    )).id();

    // Act: Send a violent death event
    app.world_mut().send_event(PopDeathEvent {
        entity: pop_id,
        cause: DeathCause::Violent(DamageType::Fire),
        name: "Engineer Bob".to_string(),
        position: machine_pos,
    });

    app.update();

    // Assert: The machine now has the Haunted component
    let haunted = app.world().get::<HauntedMachine>(machine_id);
    assert!(haunted.is_some(), "Machine should become haunted after a violent death nearby");
    assert_eq!(haunted.unwrap().ghost_name, "Engineer Bob");
}

#[test]
fn test_haunted_machine_broadcasts_stress() {
    // Arrange
    let mut app = setup_world();
    let machine_pos = GridPosition { x: 5, y: 5 };

    app.world_mut().spawn((
        Building,
        BuildingType::FusionReactor,
        HauntedMachine {
            ghost_name: "Engineer Bob".to_string(),
            broadcast_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        },
        machine_pos,
    ));

    let pop_id = app.world_mut().spawn((
        Pop,
        StressTracker { current: 0.0, max: 100.0 },
        GridPosition { x: 6, y: 5 }, // Nearby
    )).id();

    // Act: Advance time to trigger broadcast
    let mut time = app.world_mut().resource_mut::<Time>();
    time.advance_by(Duration::from_secs_f32(1.1));
    app.update();

    // Assert: Nearby pop gained stress
    let stress = app.world().get::<StressTracker>(pop_id).unwrap();
    assert!(stress.current > 0.0, "Nearby pop should gain stress when haunted machine broadcasts");

    // Assert: Notification generated
    let notifications = app.world().resource::<Notifications>();
    assert!(notifications.iter().any(|n| n.title.contains("Ghost Broadcast")));
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// In src/layer1/narrative/hauntings.rs

#[derive(Component)]
pub struct HauntedMachine {
    pub ghost_name: String,
    pub broadcast_timer: Timer,
}

pub fn haunting_creation_system(
    mut death_events: EventReader<PopDeathEvent>,
    mut commands: Commands,
    machine_query: Query<(Entity, &GridPosition), With<MachineSpirit>>,
) {
    for event in death_events.read() {
        if matches!(event.cause, DeathCause::Violent(_)) {
            // Find nearby machines (distance <= 1)
            for (machine_entity, m_pos) in machine_query.iter() {
                if event.position.distance(m_pos) <= 1.0 {
                    commands.entity(machine_entity).insert(HauntedMachine {
                        ghost_name: event.name.clone(),
                        broadcast_timer: Timer::from_seconds(100.0, TimerMode::Repeating), // Default broadcast interval
                    });
                    break; // Only haunt one machine
                }
            }
        }
    }
}

pub fn haunted_broadcast_system(
    time: Res<Time>,
    mut machine_query: Query<(&mut HauntedMachine, &GridPosition)>,
    mut pop_query: Query<(&mut StressTracker, &GridPosition), With<Pop>>,
    mut notifications: ResMut<Notifications>,
) {
    for (mut haunted, m_pos) in machine_query.iter_mut() {
        if haunted.broadcast_timer.tick(time.delta()).just_finished() {
            // Send notification
            notifications.push(Notification::new(
                "Ghost Broadcast",
                &format!("A scream matching {}'s voice echoed from the machinery.", haunted.ghost_name),
            ));

            // Apply stress to nearby pops (radius 5)
            for (mut stress, p_pos) in pop_query.iter_mut() {
                if m_pos.distance(p_pos) <= 5.0 {
                    stress.current = (stress.current + 10.0).min(stress.max);
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration:** Hook into `042 Energy System` so that the broadcast timer ticks faster when the machine is under high load or power spikes.
- **Lore:** Add a Chronicle Event when a machine first becomes haunted.
- **UI:** The inspector should clearly show that a building is `[HAUNTED by Name]`, perhaps with a glitchy visual effect on the UI panel.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/narrative/hauntings.rs`.
- [ ] Violent deaths near machines with `MachineSpirit` apply the `HauntedMachine` component.
- [ ] Haunted machines periodically increase the stress of nearby pops and generate notifications.

## 7. Technical Guidance
- Distance checks should use Chebyshev or Manhattan distance (`GridPosition::distance`) to align with grid logic.
- Ensure the `HauntedMachine` component is serialized so hauntings persist across save games.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
