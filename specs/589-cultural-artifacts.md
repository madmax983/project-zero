# 589: Cultural Artifacts

## 1. Overview
**Layer:** 1
**Fantasy:** A colony that remembers its history through art. The statue in the square isn't just decoration; it's a memory of the famine.
**Mechanic:** Crafters create "Art" (Statues, Tapestries, Songs) tagged with recent major Colony Memories. These items radiate auras: "Victory" art buffs Courage, "Tragedy" art buffs Caution but lowers Mood.

## 2. Dependencies
- Layer 1 Memory System (`ColonyMemory`)
- Layer 1 Crafting System
- Layer 1 Aura/Buff System

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_art_creation_inherits_memory() {
        let mut app = App::new();
        app.insert_resource(RecentMemories(vec![MemoryType::Tragedy_Famine]));

        let crafter = app.world_mut().spawn((Pop::default(), Crafter, CraftingTarget::Art)).id();

        app.add_systems(Update, create_art_system);
        app.update();

        // Find the spawned art
        let mut art_query = app.world_mut().query::<&CulturalArtifact>();
        let artifact = art_query.iter(app.world()).next().unwrap();

        assert_eq!(artifact.theme, MemoryType::Tragedy_Famine);
    }

    #[test]
    fn test_tragedy_art_aura_effects() {
        let mut app = App::new();
        let art = app.world_mut().spawn((
            CulturalArtifact { theme: MemoryType::Tragedy_Famine },
            AuraEmitters { radius: 5.0 }
        )).id();

        let pop = app.world_mut().spawn((
            Pop::default(),
            Mood { current: 50.0 },
            Caution { level: 10.0 }
        )).id();

        app.add_systems(Update, apply_artifact_auras_system);
        app.update();

        let mood = app.world().get::<Mood>(pop).unwrap();
        let caution = app.world().get::<Caution>(pop).unwrap();

        assert!(mood.current < 50.0, "Tragedy art should lower mood");
        assert!(caution.level > 10.0, "Tragedy art should increase caution");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(PartialEq, Clone, Copy)]
pub enum MemoryType { Tragedy_Famine, Victory_Raid, Neutral }

#[derive(Resource)]
pub struct RecentMemories(pub Vec<MemoryType>);

#[derive(Component)]
pub struct CulturalArtifact { pub theme: MemoryType }

#[derive(Component)]
pub struct AuraEmitters { pub radius: f32 }

pub fn create_art_system(
    mut commands: Commands,
    query: Query<Entity, With<Crafter>>, // Simplified trigger
    memories: Res<RecentMemories>,
) {
    for _ in query.iter() {
        let theme = memories.0.last().unwrap_or(&MemoryType::Neutral).clone();
        commands.spawn((
            CulturalArtifact { theme },
            AuraEmitters { radius: 10.0 }
        ));
    }
}

pub fn apply_artifact_auras_system(
    artifacts: Query<&CulturalArtifact>,
    mut pops: Query<(&mut Mood, &mut Caution)>,
) {
    for artifact in artifacts.iter() {
        for (mut mood, mut caution) in pops.iter_mut() {
            if artifact.theme == MemoryType::Tragedy_Famine {
                mood.current -= 5.0;
                caution.level += 5.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** The aura system needs spatial checks. Right now, `apply_artifact_auras_system` applies to all Pops regardless of `radius` or distance. Need to integrate `Transform` and calculate distance.
- **API Improvements:** `RecentMemories` should provide a weighted random choice rather than just grabbing the `.last()` memory to allow diverse art styles.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Art items successfully capture a `MemoryType` upon creation.
- [ ] Pop entities within the spatial radius of an artifact receive the correct buffs/debuffs.

## 7. Technical Guidance
- Integrate with `src/layer1/memory.rs`. When a major event fires (like a colonist death or repelling a raid), it should push to `RecentMemories`.
- Ensure auras don't stack infinitely. Use an `ActiveAuras` component on Pops to track sources.

## 8. Questions
*Builder: add questions here if spec is unclear.*
