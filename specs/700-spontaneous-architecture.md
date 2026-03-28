# 700 Spontaneous Architecture

## 1. Overview
The colony is a living, breathing entity, not just a perfectly planned grid. Pops who are idle and possess their own resources may spontaneously claim empty tiles adjacent to their housing to construct personal structures, such as sheds, gardens, or small shrines. These organic structures provide localized morale boosts but occupy valuable real estate, blocking future infrastructure projects and creating friction between the colony's central planning and individual liberty.

## 2. Dependencies
- `Pop` component and utility AI for idle state evaluation.
- `TerrainGrid` and `MapState` for tile occupation checking.
- `Morale` system for happiness buffs.
- Private stashes/inventories for Pop resources.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_idle_pop_builds_spontaneous_structure() {
        // Arrange: World with a pop, housing, and an empty adjacent tile
        let mut app = App::new();
        let home_entity = app.world_mut().spawn(Housing { ..default() }).id();
        let pop = app.world_mut().spawn((
            Pop { ..default() },
            IdleState,
            AssignedHousing(home_entity),
            Inventory { resources: vec![Resource::Scrap, Resource::Scrap] }
        )).id();

        let empty_tile = GridPosition { x: 5, y: 5 };
        app.world_mut().insert_resource(TerrainGrid::new_empty()); // Setup grid

        // Act: Run the spontaneous architecture system
        app.add_systems(Update, evaluate_spontaneous_architecture);
        app.update();

        // Assert: The pop consumed resources and built a structure
        let inventory = app.world().get::<Inventory>(pop).unwrap();
        assert!(inventory.resources.is_empty(), "Pop should consume resources to build");

        let mut structures = app.world().query::<(&SpontaneousStructure, &GridPosition)>();
        let structure_exists = structures.iter(app.world()).any(|(_, pos)| *pos == empty_tile);
        assert!(structure_exists, "A spontaneous structure should be built on the adjacent tile");
    }

    #[test]
    fn test_spontaneous_structure_boosts_local_morale() {
        // Arrange
        let mut app = App::new();
        app.world_mut().spawn((
            SpontaneousStructure { structure_type: StructureType::Garden },
            GridPosition { x: 5, y: 5 }
        ));
        let pop = app.world_mut().spawn((
            Pop { ..default() },
            Morale { value: 50.0 },
            GridPosition { x: 5, y: 6 } // Adjacent to structure
        )).id();

        // Act
        app.add_systems(Update, apply_spontaneous_morale_aura);
        app.update();

        // Assert
        let morale = app.world().get::<Morale>(pop).unwrap();
        assert!(morale.value > 50.0, "Pop morale should be boosted by the nearby spontaneous structure");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct SpontaneousStructure {
    pub structure_type: StructureType,
}

pub enum StructureType {
    Garden,
    Shed,
    Shrine,
}

pub fn evaluate_spontaneous_architecture(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut Inventory, &AssignedHousing, &IdleState)>,
    grid: Res<TerrainGrid>,
) {
    for (entity, mut inventory, housing, _) in pops.iter_mut() {
        if inventory.resources.len() >= 2 {
            // Find empty adjacent tile to housing (mock logic for minimal pass)
            if let Some(pos) = find_empty_adjacent(&grid, housing.0) {
                inventory.resources.clear();
                commands.spawn((
                    SpontaneousStructure { structure_type: StructureType::Garden },
                    pos,
                ));
            }
        }
    }
}

pub fn apply_spontaneous_morale_aura(
    structures: Query<(&SpontaneousStructure, &GridPosition)>,
    mut pops: Query<(&mut Morale, &GridPosition)>,
) {
    for (_, s_pos) in structures.iter() {
        for (mut morale, p_pos) in pops.iter_mut() {
            if s_pos.distance(p_pos) <= 1 {
                morale.value += 5.0; // Minimal morale boost
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**: Extract the aura calculation into a generic `apply_aura` utility if one doesn't exist, to support other localized buffs.
- **Code Smells**: Hardcoded resource costs (2 Scrap) should be moved to a `SpontaneousArchitectureConfig` resource.
- **Performance Considerations**: The `apply_spontaneous_morale_aura` uses an O(N*M) spatial check. Consider using a spatial hash grid or bounding volume hierarchy for larger colonies.
- **API Improvements**: Use Bevy events (`SpontaneousBuildEvent`) so that the UI and Chronicle system can react and generate lore about Pops building their own little slices of paradise.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/architecture/spontaneous.rs`.
- [ ] Pops successfully deduct from their private inventory when building.
- [ ] A new entity with `SpontaneousStructure` is spawned on an empty grid tile.

## 7. Technical Guidance
- **Code Structure Suggestions**: Create a new module `src/layer1/architecture/spontaneous.rs`. Integrate the evaluation into the Utility AI as an Action or a separate observation system for idle Pops.
- **Integration Points**: Register the building action so it can emit an `AddChronicleEvent` or rumor.
- **Gotchas**: Ensure the building only occurs on tiles that the `TerrainGrid` marks as `Buildable` and `Empty`. Do not allow Pops to build over essential pathing choke points.

## 8. Questions
*Builder: add questions here if spec is unclear.*
