# 410-The Inspector

## 1. Overview
**Layer:** Cross-layer (1 -> 3)
**Fantasy:** Hosting a dignitary who can make or break your reputation.
**Mechanic:** A high-ranking NPC (Inspector, Ambassador, Imperial Tax Auditor) arrives via shuttle. They pathfind to "High Value" or "High Traffic" areas. Their mood is determined by what they see (Decor, Food Quality, Squalor, Corpse Piles). Their exit report buffs/debuffs Layer 3 relations or funding.
**Emergence:** The Ambassador walks through a slum to get to the palace, sees a pile of vomit, and declares the colony "Uncivilized," cutting off trade.
**Tension:** Restrict their movement (insulting) or risk them seeing the ugly truth (risky)?

## 2. Dependencies
- Pathfinding system (must allow the NPC to navigate to specific zones).
- Zone designation system (High Value / High Traffic areas).
- Mood/Needs system (tracking the Inspector's state).
- Faction/Relations system (Layer 3 effects).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_inspector_spawning() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<SpawnInspectorEvent>();

        // Act
        app.world_mut().send_event(SpawnInspectorEvent {
            faction: FactionId::Empire,
        });
        app.update();

        // Assert
        let inspectors: Vec<_> = app.world_mut().query::<&Inspector>().iter(&app.world()).collect();
        assert_eq!(inspectors.len(), 1, "An inspector should spawn from the event");
    }

    #[test]
    fn test_inspector_mood_influenced_by_surroundings() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let inspector = app.world_mut().spawn((
            Inspector { base_mood: 50.0 },
            Transform::from_translation(Vec3::new(10., 0., 0.))
        )).id();

        // Spawn a corpse nearby (Squalor)
        app.world_mut().spawn((Corpse, Transform::from_translation(Vec3::new(11., 0., 0.))));

        // Act
        // Run observation system
        app.add_systems(Update, inspector_observation_system);
        app.update();

        // Assert
        let inspector_comp = app.world().get::<Inspector>(inspector).unwrap();
        assert!(inspector_comp.base_mood < 50.0, "Inspector mood should decrease when near a corpse");
    }

    #[test]
    fn test_inspector_exit_report() {
         // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let inspector = app.world_mut().spawn((
            Inspector { base_mood: 80.0 }, // High mood
            FactionRelation(FactionId::Empire, 50.0), // Current relation
        )).id();

        // Act
        // Trigger departure
        app.world_mut().send_event(InspectorDepartEvent { entity: inspector });
        app.add_systems(Update, process_inspector_departure);
        app.update();

        // Assert
        // Check if relation improved
        let relations = app.world().get::<FactionRelation>(inspector).unwrap(); // (Assuming global relation resource or component on faction entity in reality)
        assert!(relations.1 > 50.0, "High mood inspector should improve relations on departure");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass

#[derive(Component)]
pub struct Inspector {
    pub base_mood: f32,
}

#[derive(Component)]
pub struct Corpse;

pub struct SpawnInspectorEvent {
    pub faction: u32, // Simplified ID
}

pub struct InspectorDepartEvent {
    pub entity: Entity,
}

#[derive(Resource, Default)]
pub struct FactionRelations {
    pub empire: f32,
}

pub fn spawn_inspector_system(
    mut commands: Commands,
    mut events: EventReader<SpawnInspectorEvent>,
) {
    for _event in events.read() {
        commands.spawn(Inspector { base_mood: 50.0 });
    }
}

pub fn inspector_observation_system(
    mut query: Query<(&mut Inspector, &Transform)>,
    corpse_query: Query<&Transform, With<Corpse>>,
) {
    for (mut inspector, transform) in query.iter_mut() {
        for corpse_transform in corpse_query.iter() {
            if transform.translation.distance(corpse_transform.translation) < 5.0 {
                inspector.base_mood -= 10.0;
            }
        }
    }
}

pub fn process_inspector_departure(
    mut events: EventReader<InspectorDepartEvent>,
    query: Query<&Inspector>,
    mut relations: ResMut<FactionRelations>,
) {
    for event in events.read() {
        if let Ok(inspector) = query.get(event.entity) {
            if inspector.base_mood > 70.0 {
                relations.empire += 10.0;
            } else if inspector.base_mood < 30.0 {
                relations.empire -= 10.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:**
  - `inspector_observation_system` should iterate over a generic `Beauty`/`Squalor` component attached to tiles/entities rather than hardcoded `Corpse`.
  - The pathfinding AI for the inspector should target `HighValueZone` markers.
  - Implement a proper `Report` struct that details *why* the mood changed (e.g., "Saw 3 corpses, ate a fine meal").
- **Code Smells:**
  - Distance checking in an $O(N \times M)$ loop. Use a spatial grid or `kd-tree` for production.
- **Performance Considerations:**
  - Optimizing the spatial queries for observation to avoid scanning the entire map.
- **API Improvements:**
  - `InspectorDepartEvent` should include the finalized report data rather than just the entity, to decouple the departure logic from the actual entity (which might despawn immediately).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Inspector spawns upon receiving an event.
- [ ] Inspector's mood reacts to negative/positive environmental factors (Squalor/Beauty).
- [ ] Inspector's departure updates global faction relations based on final mood.

## 7. Technical Guidance
- **Code Structure:** Add to `src/layer1/visitors.rs` or a new `inspector.rs` module.
- **Integration Points:**
  - Tie the spawn event to ship arrivals at the trade pad/spaceport.
  - Integrate observation logic with the existing `Beauty`/`Squalor` map.
  - Link the final report to Layer 3 diplomacy systems.
- **Gotchas:** Make sure the inspector has a guaranteed exit path; if they get trapped, their mood should plummet rapidly.

## 8. Questions
*Builder: add questions here if spec is unclear.*
