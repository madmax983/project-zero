# The Ansible Network

## 1. Overview
**Layer:** Cross-layer (Layer 1 -> 2/3)
**Fantasy:** Information is the most valuable resource.
**Mechanic:** Without an Ansible, Layer 2/3 information (enemy fleet movements, trade prices) is delayed by distance (light lag). Building an Ansible gives real-time data but consumes massive power.
**Emergence:** You see an invasion fleet arriving "now", but the data is 2 weeks old. They are already in orbit.
**Tension:** Power the guns or the phone?

## 2. Dependencies
- Energy/Power Grid (`PowerConsumer`, `PowerProducer`)
- Signal/Event routing (System Map, Fleets, Trade)
- Layer 2 Fleet movement

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_sensor_delay_without_ansible() {
    let mut app = App::new();
    // Setup Layer 2 Fleet far away
    let enemy_fleet = app.world_mut().spawn((
        Fleet { allegiance: Allegiance::Hostile },
        Position { x: 1000.0, y: 1000.0 }, // Far away
    )).id();

    // Simulate sensor check without an active Ansible
    app.update();

    // Read the player's knowledge of the system
    let intel = app.world().resource::<SystemIntel>();
    let fleet_data = intel.get_fleet(enemy_fleet);

    assert!(fleet_data.is_some());
    // Information is delayed by distance
    assert!(fleet_data.unwrap().age > 0);
}

#[test]
fn test_ansible_provides_real_time_intel_if_powered() {
    let mut app = App::new();
    // Setup Layer 1 Ansible
    let ansible = app.world_mut().spawn((
        AnsibleDevice,
        PowerConsumer { needed: 500, received: 500 }, // Fully powered
    )).id();

    let enemy_fleet = app.world_mut().spawn((
        Fleet { allegiance: Allegiance::Hostile },
        Position { x: 1000.0, y: 1000.0 },
    )).id();

    app.update();

    let intel = app.world().resource::<SystemIntel>();
    let fleet_data = intel.get_fleet(enemy_fleet);

    assert!(fleet_data.is_some());
    // Information is real-time because Ansible is active
    assert_eq!(fleet_data.unwrap().age, 0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct AnsibleDevice;

#[derive(Resource)]
pub struct SystemIntel {
    pub fleets: HashMap<Entity, IntelData>,
}

pub struct IntelData {
    pub position: Position,
    pub age: u32,
}

pub fn update_intel_system(
    mut intel: ResMut<SystemIntel>,
    fleets: Query<(Entity, &Position, &Fleet)>,
    ansibles: Query<&PowerConsumer, With<AnsibleDevice>>,
    time: Res<Time>,
) {
    let has_active_ansible = ansibles.iter().any(|p| p.received >= p.needed);

    for (entity, pos, _) in fleets.iter() {
        let age = if has_active_ansible {
            0 // Instant
        } else {
            // Delay based on distance
            let distance = (pos.x.powi(2) + pos.y.powi(2)).sqrt();
            (distance / 100.0) as u32
        };

        intel.fleets.insert(entity, IntelData {
            position: pos.clone(),
            age,
        });
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate "true position" vs "known position" explicitly on Layer 2 objects.
- Ensure the UI correctly displays ghosts/historical paths when the Ansible is offline.
- Add an `AnsibleStatusEvent` for UI notifications when the grid drops the connection.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] `SystemIntel` correctly calculates data lag when no Ansible is powered.
- [ ] `SystemIntel` updates instantly when an Ansible is fully powered.

## 7. Technical Guidance
- The player should never directly see the "true" Layer 2 state in the UI if the Ansible is offline; they must only see the `SystemIntel` snapshot.
- Ensure `PowerConsumer` prioritization allows the player to manually shut off the Ansible to route power to weapons during a siege.

## 8. Questions
*Builder: add questions here if spec is unclear.*
