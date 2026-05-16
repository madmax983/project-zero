# Zero-G Flora

## 1. Overview
This specification details the implementation of "Zero-G Flora", a unique class of agricultural crops that can only be grown in microgravity environments (Layer 2, orbital stations). These crops yield high-value luxury goods or specialized medicines, but require expensive, dedicated orbital hydroponics bays. They present a high-risk, high-reward alternative to traditional Layer 1 (planetary) farming.

## 2. Dependencies
- Layer 1 Resource System (`layer1::resources`)
- Layer 2 Orbital Structures (`layer2::orbital_structures`)
- Layer 2 Microgravity Mechanics (`layer2::microgravity`)
- Crop System (`layer1::agriculture`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::resources::{Resource, ResourceType};
    use crate::layer1::agriculture::{Crop, GrowthConditions};
    use crate::layer2::orbital_structures::{OrbitalStation, ModuleType};
    use crate::layer2::microgravity::MicrogravityEnvironment;

    #[test]
    fn test_zero_g_flora_growth_requires_microgravity() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut growth_conditions = GrowthConditions::default();
        growth_conditions.requires_microgravity = true;

        let crop = Crop {
            name: "Void-Orchid".to_string(),
            conditions: growth_conditions,
            yield_resource: ResourceType::Luxury(String::from("Void-Orchid Extract")),
            base_growth_time: 100.0,
        };

        // Act & Assert
        // Test planetary growth (no microgravity component)
        let planetary_farm = app.world_mut().spawn(()).id();
        let result = crop.can_grow(&app.world(), planetary_farm);
        assert!(!result, "Zero-G Flora should not grow on a planetary surface.");

        // Test orbital growth (with microgravity component)
        let orbital_farm = app.world_mut().spawn(MicrogravityEnvironment {
            stability: 1.0,
        }).id();
        let result2 = crop.can_grow(&app.world(), orbital_farm);
        assert!(result2, "Zero-G Flora should grow in a microgravity environment.");
    }

    #[test]
    fn test_zero_g_flora_yields_high_value_resource() {
        // Arrange
        let crop = Crop {
            name: "Stellar-Vine".to_string(),
            conditions: GrowthConditions { requires_microgravity: true, ..default() },
            yield_resource: ResourceType::Medicine(String::from("Stellar Panacea")),
            base_growth_time: 200.0,
        };

        // Act
        let resource_yield = crop.harvest();

        // Assert
        assert!(matches!(resource_yield.resource_type, ResourceType::Medicine(_)));
        assert_eq!(resource_yield.amount, 10); // Example high yield
    }

    #[test]
    fn test_micro_meteorite_strike_destroys_crop() {
         // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Setup an orbital hydroponics bay with crops
        let farm = app.world_mut().spawn((
            MicrogravityEnvironment { stability: 1.0 },
            CropYield { amount: 50 },
            OrbitalStation { module_type: ModuleType::Hydroponics }
        )).id();

        // Act
        // Simulate depressurization event (e.g. from meteorite)
        app.world_mut().entity_mut(farm).insert(Depressurized);
        app.update();

        // Assert
        let crop_yield = app.world().get::<CropYield>(farm);
        assert!(crop_yield.is_none() || crop_yield.unwrap().amount == 0, "Crops must be destroyed upon depressurization.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Stub implementations to make the tests pass

#[derive(Component)]
pub struct MicrogravityEnvironment {
    pub stability: f32,
}

#[derive(Component)]
pub struct Depressurized;

#[derive(Component)]
pub struct CropYield {
    pub amount: u32,
}

pub mod resources {
    pub enum ResourceType {
        Food,
        Luxury(String),
        Medicine(String),
    }

    pub struct Resource {
        pub resource_type: ResourceType,
        pub amount: u32,
    }
}

pub mod agriculture {
    use super::*;
    use crate::layer1::resources::{Resource, ResourceType};

    #[derive(Default, Clone)]
    pub struct GrowthConditions {
        pub requires_microgravity: bool,
    }

    pub struct Crop {
        pub name: String,
        pub conditions: GrowthConditions,
        pub yield_resource: ResourceType,
        pub base_growth_time: f32,
    }

    impl Crop {
        pub fn can_grow(&self, world: &World, entity: Entity) -> bool {
            if self.conditions.requires_microgravity {
                world.get::<MicrogravityEnvironment>(entity).is_some()
            } else {
                true
            }
        }

        pub fn harvest(&self) -> Resource {
            // Simplified return
            Resource {
                resource_type: match &self.yield_resource {
                    ResourceType::Luxury(name) => ResourceType::Luxury(name.clone()),
                    ResourceType::Medicine(name) => ResourceType::Medicine(name.clone()),
                    _ => ResourceType::Food,
                },
                amount: 10,
            }
        }
    }

    pub fn handle_depressurization(
        mut commands: Commands,
        query: Query<(Entity, &Depressurized), With<CropYield>>
    ) {
        for (entity, _) in query.iter() {
            commands.entity(entity).remove::<CropYield>();
        }
    }
}

pub mod orbital_structures {
    pub enum ModuleType {
        Hydroponics,
        LivingQuarters,
    }

    #[derive(bevy::prelude::Component)]
    pub struct OrbitalStation {
        pub module_type: ModuleType,
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells:** The hardcoded yield values in `harvest()` should be driven by configuration or data files (e.g., balance files) rather than being hardcoded.
- **Design Improvement:** Integration with the global event system for depressurization events is required. The `Depressurized` component acts as a local state marker, but a global event (`DepressurizationEvent(Entity)`) should trigger this state change to alert other systems (like UI or alert managers).
- **Performance:** Ensure that `handle_depressurization` only runs when necessary (e.g. using `Added<Depressurized>`) instead of every tick.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for the new `agriculture` microgravity logic.
- [ ] Zero-G flora cannot be planted on planetary surfaces.
- [ ] Depressurization correctly destroys the current crop yield in an orbital hydroponics bay.

## 7. Technical Guidance
- Integrate `MicrogravityEnvironment` closely with the existing Layer 2 systems. Do not create a duplicate gravity system if one already exists.
- The `Crop` struct changes might require updating existing Layer 1 planetary crops to explicitly have `requires_microgravity: false`.
- Bevy's `App` test setup needs to correctly register the `handle_depressurization` system to pass the meteorite test. You must add `app.add_systems(Update, handle_depressurization)` in your test's Arrange phase.

## 8. Questions
*Builder: add questions here if spec is unclear.*

* Adding `String` payloads to `ResourceType` via variants `Luxury(String)` and `Medicine(String)` makes it impossible to derive `Copy`. `crate::layer1::economy::resources::ResourceType` is used everywhere in the codebase (e.g. as keys in `HashMap`s) with the assumption that it is `Copy` and `Eq`. It is impossible to implement the spec as written without fundamentally breaking and rewriting the core `ResourceType` system. How should we represent these unique resource names instead? (Maybe `&'static str`, or register them as distinct string-less variants like `ResourceType::Luxury` and use `ItemType::Curio` for strings, or use a separate component for the string identifier?)
