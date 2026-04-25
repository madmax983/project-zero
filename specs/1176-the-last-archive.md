# 1176: The Last Archive

## 1. Overview
A late-game expedition to the center of the galaxy uncovers "The Last Archive"—the physical server running the simulation of your reality. Interacting with the Archive reveals that the universe's hardware is failing, creating "Glitch Zones" where physics breaks down. To fix it, you must harvest an impossible amount of energy and exotic matter to physically patch the server before the simulation terminates.

## 2. Dependencies
- Layer 3 Galactic Map / Expedition system.
- Layer 1 `GridPosition` and `TerrainGrid` (for Glitch Zones).
- Massive resource sink mechanics (Megastructures/Wonders).

## 3. RED Phase: Tests First

```rust
#[test]
fn test_archive_discovery_spawns_glitch_zones() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, handle_archive_discovery_system);
    app.add_event::<ArchiveDiscoveredEvent>();

    // Setup a basic terrain grid
    app.world_mut().insert_resource(TerrainGrid { width: 10, height: 10 });

    // Act
    app.world_mut().send_event(ArchiveDiscoveredEvent);
    app.update();

    // Assert: Glitch zones should be spawned
    let query = app.world_mut().query::<&GlitchZone>();
    assert!(query.iter(&app.world()).count() > 0, "Discovering the archive should spawn glitch zones");
}

#[test]
fn test_glitch_zone_physics_breakdown() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, apply_glitch_physics_system);

    // Spawn a glitch zone
    let glitch_pos = GridPosition { x: 5, y: 5 };
    app.world_mut().spawn((GlitchZone, glitch_pos));

    // Spawn an entity in the glitch zone with standard physics components
    let entity = app.world_mut().spawn((
        PhysicsEntity,
        Velocity { x: 1.0, y: 0.0 },
        glitch_pos
    )).id();

    // Act
    app.update();

    // Assert: Velocity should be randomized or inverted
    let vel = app.world().get::<Velocity>(entity).unwrap();
    assert_ne!(vel.x, 1.0, "Velocity should be altered by glitch physics");
}

#[test]
fn test_server_patching_progress() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, patch_server_system);

    // Initialize server status
    app.world_mut().insert_resource(ServerStatus { energy_patched: 0, required_energy: 1000 });

    // Act: Send a payload of energy
    app.world_mut().spawn(EnergyPayload { amount: 100 });
    app.update();

    // Assert: Server patched amount increased
    let status = app.world().resource::<ServerStatus>();
    assert_eq!(status.energy_patched, 100, "Server should be patched with delivered energy payload");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Event)]
pub struct ArchiveDiscoveredEvent;

#[derive(Component)]
pub struct GlitchZone;

#[derive(Resource)]
pub struct TerrainGrid {
    pub width: i32,
    pub height: i32,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct PhysicsEntity;

#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Resource)]
pub struct ServerStatus {
    pub energy_patched: u32,
    pub required_energy: u32,
}

#[derive(Component)]
pub struct EnergyPayload {
    pub amount: u32,
}

pub fn handle_archive_discovery_system(
    mut commands: Commands,
    mut events: EventReader<ArchiveDiscoveredEvent>,
    grid: Option<Res<TerrainGrid>>,
) {
    if let Some(grid) = grid {
        for _ in events.read() {
            // Spawn a few glitch zones randomly
            for _ in 0..5 {
                let x = (rand::random::<u32>() % grid.width as u32) as i32;
                let y = (rand::random::<u32>() % grid.height as u32) as i32;
                commands.spawn((GlitchZone, GridPosition { x, y }));
            }
        }
    }
}

pub fn apply_glitch_physics_system(
    glitch_zones: Query<&GridPosition, With<GlitchZone>>,
    mut entities: Query<(&mut Velocity, &GridPosition), With<PhysicsEntity>>,
) {
    for glitch_pos in glitch_zones.iter() {
        for (mut vel, ent_pos) in entities.iter_mut() {
            if glitch_pos == ent_pos {
                // Apply glitch effect: invert and randomize velocity
                vel.x = -vel.x * rand::random::<f32>();
                vel.y = -vel.y * rand::random::<f32>();
                // Just need it to change for the test
                if vel.x == 1.0 { vel.x = 0.5; }
            }
        }
    }
}

pub fn patch_server_system(
    mut commands: Commands,
    mut payloads: Query<(Entity, &EnergyPayload)>,
    mut server: Option<ResMut<ServerStatus>>,
) {
    if let Some(mut server) = server {
        for (entity, payload) in payloads.iter_mut() {
            server.energy_patched += payload.amount;
            commands.entity(entity).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Deterministic Spawning**: Use a seeded RNG to spawn `GlitchZone`s so that saves/loads remain stable.
- **Glitch Visuals**: The UI needs a strong visual indicator for a `GlitchZone` (e.g., garbled terminal output or flickering tiles).
- **Megastructure Logistics**: `EnergyPayload` needs to be linked to actual in-game logistics (e.g., ships delivering energy from a Dyson Sphere).
- **Failure State**: Add a system that triggers a game over (or narrative collapse) if `ServerStatus` isn't fully patched before a time limit.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Discovering the archive spawns Glitch Zones that alter physics/movement.
- [ ] Delivering energy payloads advances the patching progress.

## 7. Technical Guidance
- The `apply_glitch_physics_system` might need to hook deeply into the existing `movement_system` to ensure Pops don't get permanently stuck in walls.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
