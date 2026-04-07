# 842: Data Physicality

## 1. Overview
Knowledge has physical weight and vulnerability in SCALE. Research progress and Map Data are not abstract concepts bound to the player's UI; they are physically stored in "Server Banks" or "File Cabinets". If these physical storage structures are destroyed (by fire, earthquake, or attack), the associated unlocked technology or map visibility is immediately lost (Fog of War returns, building schematics vanish).

## 2. Dependencies
- Building components (`Structure`, `Health`).
- Technology/Research system (`UnlockedTechs`, `TechTree`).
- Map visibility (`FogOfWar`).
- Destruction events (`EntityDestroyedEvent`).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_destroying_server_bank_removes_tech() {
        let mut app = App::new();
        app.add_systems(Update, data_physicality_destruction_system);
        app.init_resource::<UnlockedTechs>();

        // Arrange: Add tech and link to a specific building
        app.world_mut().resource_mut::<UnlockedTechs>().techs.push("AdvancedMasonry".to_string());

        let server_entity = app.world_mut().spawn((
            DataStorage { stored_tech: Some("AdvancedMasonry".to_string()) },
            Health { current: 0, max: 100 }, // Destroyed
        )).id();

        // Act
        app.update();

        // Assert: Tech should be gone
        let techs = app.world().resource::<UnlockedTechs>();
        assert!(!techs.techs.contains(&"AdvancedMasonry".to_string()), "Tech should be lost when storage is destroyed");
    }

    #[test]
    fn test_destroying_file_cabinet_restores_fog_of_war() {
        let mut app = App::new();
        app.add_systems(Update, data_physicality_destruction_system);
        app.init_resource::<FogOfWar>();

        // Arrange: Uncover a map sector
        app.world_mut().resource_mut::<FogOfWar>().revealed_sectors.push(SectorId(5));

        let cabinet_entity = app.world_mut().spawn((
            DataStorage { stored_map_sector: Some(SectorId(5)) },
            Health { current: 0, max: 50 }, // Destroyed
        )).id();

        // Act
        app.update();

        // Assert: Sector is fogged again
        let fow = app.world().resource::<FogOfWar>();
        assert!(!fow.revealed_sectors.contains(&SectorId(5)), "Sector should be hidden when map data is destroyed");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct UnlockedTechs {
    pub techs: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SectorId(pub u32);

#[derive(Resource, Default)]
pub struct FogOfWar {
    pub revealed_sectors: Vec<SectorId>,
}

#[derive(Component)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

#[derive(Component)]
pub struct DataStorage {
    pub stored_tech: Option<String>,
    pub stored_map_sector: Option<SectorId>,
}

pub fn data_physicality_destruction_system(
    mut commands: Commands,
    query: Query<(Entity, &Health, &DataStorage)>,
    mut techs: ResMut<UnlockedTechs>,
    mut fow: ResMut<FogOfWar>,
) {
    for (entity, health, storage) in query.iter() {
        if health.current == 0 {
            if let Some(tech) = &storage.stored_tech {
                techs.techs.retain(|t| t != tech);
            }
            if let Some(sector) = &storage.stored_map_sector {
                fow.revealed_sectors.retain(|s| s != sector);
            }

            // Clean up the destroyed building entity
            commands.entity(entity).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Driven:** Relying on `Health == 0` polling is an anti-pattern. Instead, listen for an `EntityDestroyedEvent` or `BuildingDestroyedEvent` and trigger the data loss there.
- **Data Redundancy:** A single tech might be backed up across multiple servers. Instead of an `Option<String>`, `UnlockedTechs` should probably track the *number of copies* existing in the colony, losing the tech only when the count hits 0.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code.
- [ ] Tech is removed from `UnlockedTechs` when its housing storage is destroyed.
- [ ] Map sectors revert to hidden when their specific Map Data storage is destroyed.

## 7. Technical Guidance
- **Dependencies:** The storage building must explicitly register what it holds.
- **Redundancy:** Think about how players copy data to backup servers. This will likely need a `DataTransfer` task or job in layer 1.

## 8. Questions
*Builder: add questions here if spec is unclear.*
