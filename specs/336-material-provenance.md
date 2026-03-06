# 336 - Material Provenance

## 1. Overview
The **Material Provenance** feature ensures that constructed buildings inherit properties from the raw materials used to build them. Instead of a "Wall" simply being a "Wall", a wall made of "Alien Bone" has different base statistics (Flammability, Insulation, Beauty, and potential hidden traits) than a wall made of "Plastisteel". This adds a layer of depth to construction decisions, forcing players to balance material abundance against material safety and utility.

## 2. Dependencies
- `004` Basic Building
- `018` Mining Resources
- `020` Construction Costs

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_building_inherits_material_properties() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_material_provenance_system);

        let material_entity = app.world_mut().spawn((
            ResourceItem { item_type: ResourceType::Wood },
            MaterialProperties {
                flammability: 0.8,
                insulation: 0.2,
                beauty: 0.1,
            }
        )).id();

        let building_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::Wall },
            ConstructedFrom { materials: vec![material_entity] },
        )).id();

        // Act
        app.update();

        // Assert
        let inherited_props = app.world().get::<InheritedMaterialProperties>(building_entity).unwrap();
        assert_eq!(inherited_props.flammability, 0.8);
        assert_eq!(inherited_props.insulation, 0.2);
        assert_eq!(inherited_props.beauty, 0.1);
    }

    #[test]
    fn test_building_averages_multiple_materials() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_material_provenance_system);

        let material1 = app.world_mut().spawn((
            MaterialProperties { flammability: 1.0, insulation: 0.0, beauty: 0.0 }
        )).id();

        let material2 = app.world_mut().spawn((
            MaterialProperties { flammability: 0.0, insulation: 1.0, beauty: 1.0 }
        )).id();

        let building_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::Wall },
            ConstructedFrom { materials: vec![material1, material2] },
        )).id();

        // Act
        app.update();

        // Assert
        let inherited_props = app.world().get::<InheritedMaterialProperties>(building_entity).unwrap();
        assert_eq!(inherited_props.flammability, 0.5);
        assert_eq!(inherited_props.insulation, 0.5);
        assert_eq!(inherited_props.beauty, 0.5);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct MaterialProperties {
    pub flammability: f32,
    pub insulation: f32,
    pub beauty: f32,
}

#[derive(Component)]
pub struct ConstructedFrom {
    pub materials: Vec<Entity>,
}

#[derive(Component)]
pub struct InheritedMaterialProperties {
    pub flammability: f32,
    pub insulation: f32,
    pub beauty: f32,
}

pub fn apply_material_provenance_system(
    mut commands: Commands,
    buildings: Query<(Entity, &ConstructedFrom), Without<InheritedMaterialProperties>>,
    materials: Query<&MaterialProperties>,
) {
    for (entity, constructed_from) in buildings.iter() {
        let mut total_flammability = 0.0;
        let mut total_insulation = 0.0;
        let mut total_beauty = 0.0;
        let mut count = 0.0;

        for &mat_entity in &constructed_from.materials {
            if let Ok(props) = materials.get(mat_entity) {
                total_flammability += props.flammability;
                total_insulation += props.insulation;
                total_beauty += props.beauty;
                count += 1.0;
            }
        }

        if count > 0.0 {
            commands.entity(entity).insert(InheritedMaterialProperties {
                flammability: total_flammability / count,
                insulation: total_insulation / count,
                beauty: total_beauty / count,
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**:
  - Instead of averaging all properties equally, consider a weighted average based on the quantity/mass of each material used.
  - Implement a caching mechanism for material properties if querying them constantly becomes a bottleneck.
- **Code Smells**:
  - The base implementation averages properties. Ensure it handles cases where a building is made of 99% stone and 1% wood correctly (weighted average).
- **Performance**:
  - This system only runs once per building (via `Without<InheritedMaterialProperties>`), so performance impact should be minimal.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Buildings correctly inherit and calculate properties based on their constituent materials.

## 7. Technical Guidance
- Integrate with the existing `ConstructionCost` and Great Works logic where applicable.
- Make sure to add `InheritedMaterialProperties` to the building's entity as soon as construction finishes.
- Consider what happens if a building is repaired with a *different* material later—this spec averages initial construction, but a robust implementation might recalculate on repair.

## 8. Questions
*Builder: add questions here if spec is unclear.*
