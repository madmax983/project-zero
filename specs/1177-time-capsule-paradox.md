# 1177: The Time-Capsule Paradox

## 1. Overview
Receiving a desperate message from your own future. Interacting with a "Time Anomaly" on Layer 2 drops a physical "Time Capsule" on Layer 1. The capsule contains a massive cache of late-game technology and a warning. Opening it grants the tech but creates a temporal paradox: a hostile "Mirror Empire" (representing the timeline where you *did* build the dangerous tech) spawns on the edge of the galaxy, heavily armed and intent on destroying you.

## 2. Dependencies
- Layer 2 Anomaly interaction system.
- Layer 1 `TimeCapsule` placement and interaction.
- Technology/Research unlock system.
- Layer 3 Faction generation (for the Mirror Empire).

## 3. RED Phase: Tests First

```rust
#[test]
fn test_opening_time_capsule_grants_tech() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, open_time_capsule_system);
    app.add_event::<OpenTimeCapsuleEvent>();

    // Setup player tech tree
    app.world_mut().insert_resource(PlayerTechnology { unlocked_advanced_tech: false });

    // Spawn a time capsule
    let capsule = app.world_mut().spawn(TimeCapsule).id();

    // Act
    app.world_mut().send_event(OpenTimeCapsuleEvent { entity: capsule });
    app.update();

    // Assert: Tech is unlocked and capsule is despawned
    let tech = app.world().resource::<PlayerTechnology>();
    assert!(tech.unlocked_advanced_tech, "Opening capsule should unlock advanced tech");
    assert!(app.world().get::<TimeCapsule>(capsule).is_none(), "Capsule should be consumed");
}

#[test]
fn test_opening_time_capsule_spawns_mirror_empire() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, spawn_mirror_empire_system);
    app.add_event::<TimeParadoxTriggeredEvent>();

    // Act
    app.world_mut().send_event(TimeParadoxTriggeredEvent);
    app.update();

    // Assert: Mirror empire is spawned
    let mut query = app.world_mut().query::<&MirrorEmpire>();
    assert_eq!(query.iter(&app.world()).count(), 1, "A single Mirror Empire should spawn");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct PlayerTechnology {
    pub unlocked_advanced_tech: bool,
}

#[derive(Component)]
pub struct TimeCapsule;

#[derive(Event)]
pub struct OpenTimeCapsuleEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct TimeParadoxTriggeredEvent;

#[derive(Component)]
pub struct Faction;

#[derive(Component)]
pub struct MirrorEmpire;

pub fn open_time_capsule_system(
    mut commands: Commands,
    mut events: EventReader<OpenTimeCapsuleEvent>,
    mut tech: Option<ResMut<PlayerTechnology>>,
    mut paradox_events: EventWriter<TimeParadoxTriggeredEvent>,
    capsules: Query<Entity, With<TimeCapsule>>,
) {
    if let Some(mut tech) = tech {
        for event in events.read() {
            if capsules.get(event.entity).is_ok() {
                // Unlock tech
                tech.unlocked_advanced_tech = true;

                // Trigger Paradox
                paradox_events.send(TimeParadoxTriggeredEvent);

                // Consume capsule
                commands.entity(event.entity).despawn();
            }
        }
    }
}

pub fn spawn_mirror_empire_system(
    mut commands: Commands,
    mut events: EventReader<TimeParadoxTriggeredEvent>,
    mirror_empires: Query<(), With<MirrorEmpire>>,
) {
    for _ in events.read() {
        // Prevent multiple mirror empires for now
        if mirror_empires.is_empty() {
            commands.spawn((Faction, MirrorEmpire));
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Tech Integration**: `unlocked_advanced_tech` is a placeholder. It needs to hook into the actual `TechTree` system to unlock specific nodes or grant a massive amount of `ResearchPoints`.
- **Mirror Empire Generation**: The `MirrorEmpire` entity needs a name generation matching the player's but twisted (e.g., "The Dark Terran Dominion"), and it needs to spawn fleets equipped with the newly unlocked tech.
- **Narrative Hook**: The warning message from the capsule needs to be piped into the `NarrativeGenerator` to display to the user.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Opening a Time Capsule grants tech and reliably spawns a hostile Mirror Empire faction.

## 7. Technical Guidance
- Ensure the event chain correctly crosses from the UI interaction (Layer 1) to the global faction spawning (Layer 3).

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
