# Spec 608: Cultural Artifacts

## 1. Overview
**Layer:** 1
**Fantasy:** A colony that remembers its history through art. The statue in the square isn't just decoration; it's a memory of the famine.
**Mechanic:** Crafters create "Art" (Statues, Tapestries, Songs) tagged with recent major Colony Memories. These items radiate auras: "Victory" art buffs Courage, "Tragedy" art buffs Caution but lowers Mood.
**Emergence:** A colony filled with monuments to a past massacre becomes grim, vigilant, and unshakeable. A colony with only "Feast" art becomes happy but soft.
**Tension:** Erase the painful history (happiness) or preserve it (resilience)?

## 2. Dependencies
- `src/layer1/items.rs` or `src/layer1/building.rs` (Art as constructible entities)
- `src/layer1/memory.rs` or `src/layer1/chronicle.rs` (Colony-wide memory tracking)
- `src/layer1/morale.rs` and `src/layer1/traits.rs` (Applying auras for Mood, Courage, Caution)
- `src/layer1/spatial.rs` or `src/layer1/map.rs` (Aura radius checks)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod cultural_artifacts_tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_art_creation_tags_with_memory() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, create_art_system);

        let memory = ColonyMemory { theme: MemoryTheme::Tragedy, impact: 10 };
        app.insert_resource(RecentMemories(vec![memory.clone()]));

        let crafter_entity = app.world_mut().spawn((
            Pop,
            CraftingJob { target: ArtType::Statue }
        )).id();

        // Act
        app.update();

        // Assert
        let art_query = app.world().query::<&CulturalArtifact>().iter(&app.world()).collect::<Vec<_>>();
        assert_eq!(art_query.len(), 1, "An art piece should be created.");
        assert_eq!(art_query[0].memory.theme, MemoryTheme::Tragedy, "Art should be tagged with the recent memory.");
    }

    #[test]
    fn test_tragedy_art_radiates_caution_and_lowers_mood() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_art_auras_system);

        let art_entity = app.world_mut().spawn((
            CulturalArtifact { memory: ColonyMemory { theme: MemoryTheme::Tragedy, impact: 10 } },
            GridPosition { x: 5, y: 5 },
        )).id();

        let pop_entity = app.world_mut().spawn((
            Pop,
            GridPosition { x: 5, y: 6 }, // Inside radius
            Mood { value: 50.0 },
            Caution { value: 50.0 },
        )).id();

        // Act
        app.update();

        // Assert
        let mood = app.world().get::<Mood>(pop_entity).unwrap();
        let caution = app.world().get::<Caution>(pop_entity).unwrap();

        assert!(mood.value < 50.0, "Tragedy art should lower mood.");
        assert!(caution.value > 50.0, "Tragedy art should increase caution.");
    }

    #[test]
    fn test_victory_art_radiates_courage_and_buffs_mood() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_art_auras_system);

        let art_entity = app.world_mut().spawn((
            CulturalArtifact { memory: ColonyMemory { theme: MemoryTheme::Victory, impact: 10 } },
            GridPosition { x: 5, y: 5 },
        )).id();

        let pop_entity = app.world_mut().spawn((
            Pop,
            GridPosition { x: 5, y: 6 }, // Inside radius
            Mood { value: 50.0 },
            Courage { value: 50.0 },
        )).id();

        // Act
        app.update();

        // Assert
        let mood = app.world().get::<Mood>(pop_entity).unwrap();
        let courage = app.world().get::<Courage>(pop_entity).unwrap();

        assert!(mood.value > 50.0, "Victory art should increase mood.");
        assert!(courage.value > 50.0, "Victory art should increase courage.");
    }

    #[test]
    fn test_art_aura_respects_radius() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_art_auras_system);

        let art_entity = app.world_mut().spawn((
            CulturalArtifact { memory: ColonyMemory { theme: MemoryTheme::Victory, impact: 10 } },
            GridPosition { x: 5, y: 5 }, // Aura radius 5
        )).id();

        let far_pop_entity = app.world_mut().spawn((
            Pop,
            GridPosition { x: 15, y: 15 }, // Outside radius
            Mood { value: 50.0 },
            Courage { value: 50.0 },
        )).id();

        // Act
        app.update();

        // Assert
        let mood = app.world().get::<Mood>(far_pop_entity).unwrap();
        let courage = app.world().get::<Courage>(far_pop_entity).unwrap();

        assert_eq!(mood.value, 50.0, "Pop outside radius should not be affected.");
        assert_eq!(courage.value, 50.0, "Pop outside radius should not be affected.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Mood { pub value: f32 }

#[derive(Component)]
pub struct Courage { pub value: f32 }

#[derive(Component)]
pub struct Caution { pub value: f32 }

#[derive(Component, Clone)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, PartialEq, Debug)]
pub enum MemoryTheme {
    Victory,
    Tragedy,
    Feast,
}

#[derive(Clone, Debug)]
pub struct ColonyMemory {
    pub theme: MemoryTheme,
    pub impact: i32,
}

#[derive(Resource)]
pub struct RecentMemories(pub Vec<ColonyMemory>);

pub enum ArtType {
    Statue,
    Tapestry,
}

#[derive(Component)]
pub struct CraftingJob {
    pub target: ArtType,
}

#[derive(Component)]
pub struct CulturalArtifact {
    pub memory: ColonyMemory,
}

pub fn create_art_system(
    mut commands: Commands,
    mut query: Query<(Entity, &CraftingJob), With<Pop>>,
    memories: Option<Res<RecentMemories>>,
) {
    if let Some(recent_memories) = memories {
        if let Some(latest_memory) = recent_memories.0.last() {
            for (entity, _) in query.iter_mut() {
                // Complete job and spawn artifact
                commands.entity(entity).remove::<CraftingJob>();
                commands.spawn(CulturalArtifact {
                    memory: latest_memory.clone(),
                });
            }
        }
    }
}

pub fn apply_art_auras_system(
    art_query: Query<(&CulturalArtifact, &GridPosition)>,
    mut pop_query: Query<(&mut Mood, Option<&mut Courage>, Option<&mut Caution>, &GridPosition), With<Pop>>,
) {
    let aura_radius_sq = 25; // radius 5

    for (artifact, art_pos) in art_query.iter() {
        for (mut mood, mut courage_opt, mut caution_opt, pop_pos) in pop_query.iter_mut() {
            let dx = art_pos.x - pop_pos.x;
            let dy = art_pos.y - pop_pos.y;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq <= aura_radius_sq {
                match artifact.memory.theme {
                    MemoryTheme::Tragedy => {
                        mood.value -= 1.0;
                        if let Some(mut caution) = caution_opt {
                            caution.value += 1.0;
                        }
                    }
                    MemoryTheme::Victory => {
                        mood.value += 1.0;
                        if let Some(mut courage) = courage_opt {
                            courage.value += 1.0;
                        }
                    }
                    MemoryTheme::Feast => {
                        mood.value += 2.0;
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactor Opportunity:** Instead of applying flat stat modifications every tick inside `apply_art_auras_system`, use a temporary ECS component (`AuraBuff { source: Entity, duration: f32 }`) to track modifiers. This prevents mood from instantly dropping to 0 or 100 within a few frames.
- **Integration:** Link `ColonyMemory` extraction to the actual `Chronicle` system (`src/layer1/chronicle.rs`) so that events like `RaidDefeatedEvent` automatically become `Victory` memories available for crafters.
- **Optimization:** Use spatial partitioning (e.g., querying rooms or a grid chunk) to avoid an O(N*M) loop checking every pop against every artifact.
- **Lore Connection:** Integrate with the `Lore Master`'s lexical generation to give generated art specific procedural names (e.g., "The Weeping Obsidian of Year 4").

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Art correctly inherits the `MemoryTheme` from recent colony events.
- [ ] Art emits local auras modifying Pop behavior/moods appropriately.

## 7. Technical Guidance
- Implement within `src/layer1/art.rs` or `src/layer1/culture.rs`.
- Ensure traits like `Courage` and `Caution` actually influence utility AI decisions (e.g., a highly cautious pop will flee earlier during a raid, while a courageous one might fight back).
- Provide a UI tooltip for the artifact explaining its aura and the specific history memory it represents.

## 8. Questions
*Builder: add questions here if spec is unclear.*
