# 1309: The Asteroid Hermits

## 1. Overview
**Layer:** 2

**Fantasy:** Pops rejecting the complex society of core worlds to live a rugged, isolated life on barren rocks.

**Mechanic:** Pops with high dissatisfaction and low social needs might hijack or purchase small craft to settle uncolonized asteroids. These "Hermit Outposts" generate no taxes but occasionally discover rare minerals or anomalous artifacts due to their deep-space isolation.

**Emergence:** A thriving, hyper-bureaucratic core world might see a mass exodus of its most skilled but antisocial miners, creating a network of independent, hard-to-tax asteroid settlements that eventually demand sovereignty.

**Tension:** Do you force the hermits back into the fold to reclaim their labor, or leave them alone and trade for the bizarre artifacts they sometimes dig up?

## 2. Dependencies
- Layer 1 Needs/Dissatisfaction systems.
- Layer 2 Orbit/Node structures (Asteroids).
- Layer 1 to Layer 2 abstraction (converting Pops to Hermit Outposts).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // RED Phase Test Setup
    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            evaluate_hermit_exodus_system,
            process_hermit_discoveries_system,
        ));
        app
    }

    #[test]
    fn test_dissatisfied_pop_becomes_hermit() {
        let mut app = setup_app();

        // Spawn an uncolonized asteroid
        let asteroid = app.world_mut().spawn((
            Asteroid,
            Uncolonized,
        )).id();

        // Spawn a highly dissatisfied pop with low social need
        let pop = app.world_mut().spawn((
            Pop,
            Dissatisfaction { level: 95.0 }, // High dissatisfaction
            SocialNeed { value: 10.0 },      // Low social need
        )).id();

        app.update();

        // Assert: Pop is removed/converted and Asteroid becomes HermitOutpost
        assert!(app.world().get::<Pop>(pop).is_none(), "Pop should leave the colony");

        let outpost = app.world().get::<HermitOutpost>(asteroid);
        assert!(outpost.is_some(), "Asteroid should now be a Hermit Outpost");
        assert!(app.world().get::<Uncolonized>(asteroid).is_none(), "Asteroid should no longer be Uncolonized");
    }

    #[test]
    fn test_satisfied_pop_does_not_become_hermit() {
        let mut app = setup_app();

        let asteroid = app.world_mut().spawn((
            Asteroid,
            Uncolonized,
        )).id();

        let pop = app.world_mut().spawn((
            Pop,
            Dissatisfaction { level: 20.0 }, // Low dissatisfaction
            SocialNeed { value: 10.0 },
        )).id();

        app.update();

        assert!(app.world().get::<Pop>(pop).is_some(), "Satisfied Pop should stay");
        assert!(app.world().get::<HermitOutpost>(asteroid).is_none(), "Asteroid should remain uncolonized");
    }

    #[test]
    fn test_hermit_outpost_discovers_artifacts() {
        let mut app = setup_app();
        app.add_event::<DiscoveryEvent>();

        // Spawn a Hermit Outpost with high discovery progress
        app.world_mut().spawn((
            HermitOutpost,
            DiscoveryProgress { progress: 100.0 }, // Ready to discover
        ));

        app.update();

        // Assert: DiscoveryEvent should be emitted
        let events = app.world().resource::<Events<DiscoveryEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).next().is_some(), "Hermit Outpost should generate a DiscoveryEvent");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Dissatisfaction {
    pub level: f32,
}

#[derive(Component)]
pub struct SocialNeed {
    pub value: f32,
}

#[derive(Component)]
pub struct Asteroid;

#[derive(Component)]
pub struct Uncolonized;

#[derive(Component)]
pub struct HermitOutpost;

#[derive(Component, Default)]
pub struct DiscoveryProgress {
    pub progress: f32,
}

#[derive(Event)]
pub struct DiscoveryEvent {
    pub outpost: Entity,
    pub item_type: String, // E.g., "Rare Mineral", "Anomalous Artifact"
}

pub fn evaluate_hermit_exodus_system(
    mut commands: Commands,
    pops: Query<(Entity, &Dissatisfaction, &SocialNeed), With<Pop>>,
    asteroids: Query<Entity, (With<Asteroid>, With<Uncolonized>)>,
) {
    let mut available_asteroids: Vec<Entity> = asteroids.iter().collect();

    for (pop_entity, dissatisfaction, social_need) in pops.iter() {
        if dissatisfaction.level > 90.0 && social_need.value < 20.0 {
            if let Some(asteroid_entity) = available_asteroids.pop() {
                // Remove the pop
                commands.entity(pop_entity).despawn();

                // Convert the asteroid
                commands.entity(asteroid_entity).remove::<Uncolonized>();
                commands.entity(asteroid_entity).insert((
                    HermitOutpost,
                    DiscoveryProgress::default(),
                ));
            }
        }
    }
}

pub fn process_hermit_discoveries_system(
    mut outposts: Query<(Entity, &mut DiscoveryProgress), With<HermitOutpost>>,
    mut event_writer: EventWriter<DiscoveryEvent>,
) {
    for (entity, mut discovery) in outposts.iter_mut() {
        discovery.progress += 1.0; // Fixed increment for minimal implementation

        if discovery.progress >= 100.0 {
            discovery.progress = 0.0;
            event_writer.send(DiscoveryEvent {
                outpost: entity,
                item_type: "Anomalous Artifact".to_string(),
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **RNG & Probability**: `evaluate_hermit_exodus_system` currently forces an exodus deterministically. It should use RNG so that only a *chance* of exodus occurs each tick when conditions are met.
- **Ship/Travel Mechanic**: Pops currently teleport to the asteroid and despawn. The system should ideally spawn a small ship entity that travels to the node before converting it.
- **Discovery Variation**: `process_hermit_discoveries_system` generates a static artifact. This should be wired to a loot table or random generator (perhaps integrating with existing Inventory or Trade systems).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Dissatisfied Pops with low social needs correctly leave to form Hermit Outposts if uncolonized asteroids exist.
- [ ] Hermit Outposts periodically generate DiscoveryEvents.

## 7. Technical Guidance
- **Layer Crossing**: Pay attention to how Pop deletion is handled in Layer 1 so that housing, jobs, and social relations are properly cleaned up when the Pop becomes a hermit.
- **Chronicle Integration**: Emitting an `AddChronicleEvent` when a Hermit Outpost is formed would be a great way to log the "Mass Exodus" event in the lore.

## 8. Questions
*Builder: add questions here if spec is unclear.*
