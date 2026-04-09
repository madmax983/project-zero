# The Orbital Breadcrumbs

## 1. Overview
Fleeing pirate fleets or damaged merchant vessels in the system layer (Layer 2) occasionally jettison encrypted data drives or cargo pods into the colony's orbit. These pods slowly decay their orbit and eventually fall to the planet's surface (Layer 1). Collecting these fallen "Breadcrumbs" yields fragments of valuable tactical data, treasure maps, or localized warnings, creating a dynamic bridge between galactic events and surface gameplay.

## 2. Dependencies
- Cross-layer event system bridging Layer 2 and Layer 1.
- Existing orbital or space-layer entity structure.
- Layer 1 terrain generation or entity placement logic to handle falling objects.
- `Time` resource for decay handling.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_jettison_event_spawns_orbital_breadcrumb() {
    // Arrange
    let mut app = App::new();
    app.add_event::<JettisonPodEvent>();
    app.add_systems(Update, handle_jettison_event_system);

    // Act
    app.world_mut().send_event(JettisonPodEvent {
        pod_data_type: PodDataType::TreasureMapFragment,
        orbit_decay_days: 10.0,
    });
    app.update();

    // Assert: An OrbitalBreadcrumb entity should exist with the given data
    let query = app.world_mut().query::<&OrbitalBreadcrumb>().iter(&app.world()).next();
    assert!(query.is_some());
    assert_eq!(query.unwrap().decay_remaining, 10.0);
}

#[test]
fn test_orbital_breadcrumb_decays_and_falls_to_surface() {
    // Arrange
    let mut app = App::new();
    app.insert_resource(Time::default());
    app.add_event::<PodFellToSurfaceEvent>();
    app.add_systems(Update, decay_orbital_breadcrumb_system);

    let pod = app.world_mut().spawn(OrbitalBreadcrumb {
        pod_data_type: PodDataType::Warning,
        decay_remaining: 1.0,
    }).id();

    // Act: Advance time by more than 1 day
    app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(101));
    app.update();

    // Assert: Breadcrumb despawned from orbit, surface event fired
    assert!(app.world().get::<OrbitalBreadcrumb>(pod).is_none());
    let events = app.world().resource::<Events<PodFellToSurfaceEvent>>();
    let mut reader = events.get_reader();
    let ev = reader.read(events).next().expect("Expected PodFellToSurfaceEvent to be emitted");
    assert_eq!(ev.pod_data_type, PodDataType::Warning);
}

#[test]
fn test_surface_pod_collection_yields_data() {
    // Arrange
    let mut app = App::new();
    app.add_event::<PodCollectedEvent>();
    app.add_event::<DataFragmentAcquiredEvent>();
    app.add_systems(Update, collect_surface_pod_system);

    let pod = app.world_mut().spawn((
        SurfacePod { pod_data_type: PodDataType::TreasureMapFragment },
    )).id();

    // Act
    app.world_mut().send_event(PodCollectedEvent { entity: pod });
    app.update();

    // Assert: Surface pod is gone, data fragment acquired
    assert!(app.world().get::<SurfacePod>(pod).is_none());

    let events = app.world().resource::<Events<DataFragmentAcquiredEvent>>();
    let mut reader = events.get_reader();
    assert!(reader.read(events).next().is_some());
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PodDataType {
    TreasureMapFragment,
    Warning,
    EncryptedDrive,
}

#[derive(Event)]
pub struct JettisonPodEvent {
    pub pod_data_type: PodDataType,
    pub orbit_decay_days: f32,
}

#[derive(Event)]
pub struct PodFellToSurfaceEvent {
    pub pod_data_type: PodDataType,
}

#[derive(Event)]
pub struct PodCollectedEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct DataFragmentAcquiredEvent {
    pub fragment_type: PodDataType,
}

#[derive(Component)]
pub struct OrbitalBreadcrumb {
    pub pod_data_type: PodDataType,
    pub decay_remaining: f32,
}

#[derive(Component)]
pub struct SurfacePod {
    pub pod_data_type: PodDataType,
}

pub fn handle_jettison_event_system(
    mut commands: Commands,
    mut events: EventReader<JettisonPodEvent>,
) {
    for event in events.read() {
        commands.spawn(OrbitalBreadcrumb {
            pod_data_type: event.pod_data_type,
            decay_remaining: event.orbit_decay_days,
        });
    }
}

pub fn decay_orbital_breadcrumb_system(
    mut commands: Commands,
    time: Res<Time>,
    mut pods: Query<(Entity, &mut OrbitalBreadcrumb)>,
    mut fall_events: EventWriter<PodFellToSurfaceEvent>,
) {
    // Arbitrary scale: 1 day = 100 seconds
    let day_delta = time.delta_secs() / 100.0;

    for (entity, mut pod) in pods.iter_mut() {
        pod.decay_remaining -= day_delta;
        if pod.decay_remaining <= 0.0 {
            fall_events.send(PodFellToSurfaceEvent {
                pod_data_type: pod.pod_data_type,
            });
            commands.entity(entity).despawn();
        }
    }
}

pub fn collect_surface_pod_system(
    mut commands: Commands,
    mut collect_events: EventReader<PodCollectedEvent>,
    pods: Query<&SurfacePod>,
    mut acquire_events: EventWriter<DataFragmentAcquiredEvent>,
) {
    for ev in collect_events.read() {
        if let Ok(pod) = pods.get(ev.entity) {
            acquire_events.send(DataFragmentAcquiredEvent {
                fragment_type: pod.pod_data_type,
            });
            commands.entity(ev.entity).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spawn Coordinates**: `PodFellToSurfaceEvent` currently lacks coordinates. A listener system needs to take this event and actually spawn a `SurfacePod` onto the `TerrainGrid` in Layer 1.
- **Collection Action**: `PodCollectedEvent` represents the mechanic abstractly, but an integration into `evaluate_actions_system` is needed so haulers or explorers prioritize moving to the `SurfacePod` coordinates to interact with it.
- **Data Accumulation**: `DataFragmentAcquiredEvent` should tie into a central resource that tracks the 0/5 fragments needed to generate a real, high-value map or warning.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Breadcrumbs correctly transition from Orbital components to Surface events to Acquired data fragments.

## 7. Technical Guidance
- Integrate the drop locations smartly. Pods shouldn't land inside mountains or solid walls unless the collision logic destroys them upon impact.
- Consider giving pods a physical visual effect when they enter the atmosphere, signaling their upcoming presence to the player.

## 8. Questions
*Builder: How should we represent the piecing together of the fragments? A dedicated UI window, or just popups via the narrative chronicle when the required amount is reached?*
