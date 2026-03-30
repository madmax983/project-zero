# 749 - The Phantom Armada

## 1. Overview
Introduce `SensorBlip` components and `SensorGhostEvent`. Long-range sensors occasionally detect massive enemy fleets on Layer 2. These might be "sensor ghosts" caused by stellar radiation, or actual fleets. Players must decide whether to panic-build defenses or ignore the warning. Spending resources launches a `ProbeVerificationEvent` to determine reality.

## 2. Dependencies
- `094` System View Architecture
- `099` Fleet Movement
- `672` Sensor Ambiguity

## 3. RED Phase: Tests First
```rust
#[test]
fn test_sensor_ghost_event_creates_phantom_blip() {
    let mut app = setup_test_app();

    app.world_mut().send_event(SensorGhostEvent {
        sector: Sector { x: 10, y: 10 },
        is_real: false,
    });

    app.update();

    // Check that a massive unidentified fleet blip was created at sector 10,10
    let mut query = app.world_mut().query::<(&SensorBlip, &Sector)>();
    let (blip, pos) = query.single(app.world());
    assert_eq!(pos.x, 10);
    assert!(blip.is_massive_threat);
}

#[test]
fn test_probe_verification_reveals_ghost() {
    let mut app = setup_test_app();

    let sector = Sector { x: 10, y: 10 };
    let blip = app.world_mut().spawn((
        SensorBlip { is_massive_threat: true, is_real: false },
        sector,
    )).id();

    // Send a probe
    app.world_mut().send_event(ProbeVerificationEvent { target: sector });

    app.update();

    // The blip should be removed or marked as a ghost (despawned for simplicity)
    assert!(app.world().get_entity(blip).is_err());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Event)]
pub struct SensorGhostEvent {
    pub sector: Sector,
    pub is_real: bool,
}

#[derive(Event)]
pub struct ProbeVerificationEvent {
    pub target: Sector,
}

#[derive(Component)]
pub struct SensorBlip {
    pub is_massive_threat: bool,
    pub is_real: bool,
}

pub fn spawn_sensor_ghost_system(
    mut commands: Commands,
    mut events: EventReader<SensorGhostEvent>,
) {
    for event in events.read() {
        commands.spawn((
            SensorBlip {
                is_massive_threat: true,
                is_real: event.is_real,
            },
            event.sector,
        ));
    }
}

pub fn verify_probe_system(
    mut commands: Commands,
    mut events: EventReader<ProbeVerificationEvent>,
    blips: Query<(Entity, &SensorBlip, &Sector)>,
) {
    for event in events.read() {
        for (entity, blip, sector) in blips.iter() {
            if event.target == *sector && !blip.is_real {
                commands.entity(entity).despawn();
            } else if event.target == *sector && blip.is_real {
                // Reveal actual fleet logic here (e.g. replace blip with Fleet)
                commands.entity(entity).remove::<SensorBlip>();
                commands.entity(entity).insert(Fleet { faction: Hostile });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Currently the probe verification happens instantly. Add a `ProbeFlightTimer` so verification takes game-time.
- The `Fleet` component injection should properly spawn a full fleet composition rather than just injecting a component on the old blip entity.
- If players ignore a real fleet, it should eventually arrive.

## 6. Acceptance Criteria
- [ ] Random `SensorGhostEvent`s spawn large blips on the Layer 2 map.
- [ ] `ProbeVerificationEvent` resolves the blip (despawns if fake, spawns hostile fleet if real).
- [ ] Tests pass and `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Coverage ≥85% for the new phantom armada systems.

## 7. Technical Guidance
- Integrate with the existing `Sensor Ambiguity` (Spec 672) logic if possible, treating the Phantom Armada as an extreme end-game version of an unidentified contact.
- When creating the actual fleet on verification, use the `FleetSpawner` or equivalent logic.

## 8. Questions
- *Builder: How long should a probe take to arrive? Make it configurable so Designer can balance the tension.*
  - *Architect:* Yes, please make the probe flight time configurable via a constant or resource, so the Designer can balance the tension. A default of a few in-game days is a good starting point.
