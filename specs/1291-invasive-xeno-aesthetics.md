# 1291: Invasive Xeno-Aesthetics

## 1. Overview
**Layer:** Cross-layer (1 -> 3)

**Fantasy:** A foreign culture taking over your empire, not through war, but through fashion and art.

**Mechanic:** A neighboring, highly influential empire exports "Cultural Artifacts." When these arrive on your Layer 1 colonies, Pops gain a massive mood boost, but they begin to demand their living quarters and public spaces be redesigned to mimic the foreign aesthetics. If you refuse, they suffer severe "Aesthetic Deprivation."

## 2. Dependencies
- Cross-Layer Trade/Logistics
- Pop Mood System
- Grid / Building System
- Faction/Culture System

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
            process_cultural_artifact_system,
            process_aesthetic_deprivation_system,
        ));
        app
    }

    #[test]
    fn test_cultural_artifact_boosts_mood_and_adds_demand() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            Mood { level: 50.0 },
        )).id();

        let colony = app.world_mut().spawn(Colony).id();

        // Simulate artifact arrival
        app.world_mut().spawn(CulturalArtifact { target_colony: colony, source_culture: 2 });
        app.world_mut().entity_mut(pop).insert(ResidentOf(colony));

        app.update();

        let mood = app.world().get::<Mood>(pop).unwrap();
        assert!(mood.level > 50.0, "Artifact should boost mood");

        assert!(app.world().get::<AestheticDemand>(pop).is_some(), "Pop should demand new aesthetics after exposure");
    }

    #[test]
    fn test_unfulfilled_aesthetic_demand_causes_deprivation() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            Mood { level: 100.0 },
            AestheticDemand { culture_id: 2, timer: 10 },
        )).id();

        // Simulate timer expiration
        let mut demand = app.world_mut().get_mut::<AestheticDemand>(pop).unwrap();
        demand.timer = 0;

        app.update();

        let mood = app.world().get::<Mood>(pop).unwrap();
        assert!(mood.level < 100.0, "Mood should drop due to aesthetic deprivation");
        assert!(app.world().get::<AestheticDeprivation>(pop).is_some(), "Pop should gain deprivation component");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Colony;

#[derive(Component)]
pub struct ResidentOf(pub Entity);

#[derive(Component)]
pub struct Mood {
    pub level: f32,
}

#[derive(Component)]
pub struct CulturalArtifact {
    pub target_colony: Entity,
    pub source_culture: u32,
}

#[derive(Component)]
pub struct AestheticDemand {
    pub culture_id: u32,
    pub timer: u32,
}

#[derive(Component)]
pub struct AestheticDeprivation;

pub fn process_cultural_artifact_system(
    mut commands: Commands,
    artifacts: Query<(Entity, &CulturalArtifact)>,
    mut pops: Query<(Entity, &ResidentOf, &mut Mood)>,
) {
    for (artifact_entity, artifact) in artifacts.iter() {
        for (pop_entity, resident_of, mut mood) in pops.iter_mut() {
            if resident_of.0 == artifact.target_colony {
                // Massive mood boost
                mood.level += 30.0;

                // Add the demand
                commands.entity(pop_entity).insert(AestheticDemand {
                    culture_id: artifact.source_culture,
                    timer: 500 // Arbitrary time to fulfill
                });
            }
        }
        // Consume the artifact
        commands.entity(artifact_entity).despawn();
    }
}

pub fn process_aesthetic_deprivation_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut AestheticDemand, &mut Mood)>,
) {
    for (entity, mut demand, mut mood) in pops.iter_mut() {
        if demand.timer > 0 {
            demand.timer -= 1;
        } else {
            // Demand expired unfulfilled
            mood.level -= 40.0; // Severe drop
            commands.entity(entity).insert(AestheticDeprivation);
            commands.entity(entity).remove::<AestheticDemand>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Demand Fulfillment**: Add a mechanic where upgrading a `Housing` building to `ForeignStyleHousing` removes the `AestheticDemand` from its residents.
- **Artifact Spreading**: Instead of despawning the artifact instantly and hitting everyone, it could exist in a `Plaza` or `Market` and slowly infect nearby Pops with the demand via proximity.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- **Culture IDs**: `source_culture` should eventually link to the Layer 3 Empire/Culture data structure to correctly identify the rival state.

## 8. Questions
*Builder: add questions here if spec is unclear.*
