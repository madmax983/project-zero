# 785: The Symbiotic Infrastructure

## 1. Overview
**Layer:** 1
**Fantasy:** Your buildings aren't just structures; they're living organisms that need care.
**Mechanic:** You can grow "Symbiotic Structures" instead of building them. They cost food instead of minerals and slowly repair themselves. However, they also have "Needs" like Pops (e.g., specific temperatures, light levels, or even social interaction from caretaker Pops).
**Emergence:** During a solar eclipse, your bioluminescent, symbiotic power grid "goes to sleep," plunging the colony into darkness and causing a cascade failure of your life support systems because you didn't provide enough artificial light to keep the grid awake.
**Tension:** The self-sustaining resilience and low mineral cost of living buildings vs. the constant, complex maintenance of their biological needs.

## 2. Dependencies
- Layer 1 Buildings and Structure System (`Structure`, `BuildingType`)
- Layer 1 Resources (`ColonyResources`, especially Food)
- Layer 1 Environment (Temperature, Light Levels)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::structure::Structure;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            process_symbiotic_regeneration_system,
            process_symbiotic_needs_system
        ));
        app
    }

    #[test]
    fn test_symbiotic_structure_regenerates_health() {
        let mut app = setup_app();

        // Spawn a damaged symbiotic structure
        let entity = app.world_mut().spawn((
            Structure { current_hp: 50.0, max_hp: 100.0 },
            SymbioticStructure { regeneration_rate: 5.0, is_dormant: false },
        )).id();

        app.update();

        // Structure HP should increase
        let structure = app.world().get::<Structure>(entity).unwrap();
        assert_eq!(structure.current_hp, 55.0, "Symbiotic structure should regenerate HP when not dormant");
    }

    #[test]
    fn test_symbiotic_structure_goes_dormant_if_needs_unmet() {
        let mut app = setup_app();

        // Spawn structure with specific light need
        let entity = app.world_mut().spawn((
            Structure { current_hp: 50.0, max_hp: 100.0 },
            SymbioticStructure { regeneration_rate: 5.0, is_dormant: false },
            SymbioticNeeds { required_light: 100.0, required_temp: 20.0 },
            EnvironmentStatus { current_light: 0.0, current_temp: 20.0 }, // Light is 0 (eclipse scenario)
        )).id();

        app.update();

        // Structure should become dormant
        let symbiotic = app.world().get::<SymbioticStructure>(entity).unwrap();
        assert!(symbiotic.is_dormant, "Structure should enter dormancy when light needs are unmet");

        // Dormant structure should NOT regenerate
        app.update();
        let structure = app.world().get::<Structure>(entity).unwrap();
        assert_eq!(structure.current_hp, 50.0, "Dormant structure should not regenerate health");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::structure::Structure;

#[derive(Component)]
pub struct SymbioticStructure {
    pub regeneration_rate: f32,
    pub is_dormant: bool,
}

#[derive(Component)]
pub struct SymbioticNeeds {
    pub required_light: f32,
    pub required_temp: f32,
}

// Dummy environment component for testing
#[derive(Component)]
pub struct EnvironmentStatus {
    pub current_light: f32,
    pub current_temp: f32,
}

pub fn process_symbiotic_needs_system(
    mut query: Query<(&mut SymbioticStructure, &SymbioticNeeds, &EnvironmentStatus)>,
) {
    for (mut symbiotic, needs, env) in query.iter_mut() {
        // Simple check: if environment light falls below required, go dormant
        if env.current_light < needs.required_light || env.current_temp < needs.required_temp {
            symbiotic.is_dormant = true;
        } else {
            symbiotic.is_dormant = false;
        }
    }
}

pub fn process_symbiotic_regeneration_system(
    mut query: Query<(&mut Structure, &SymbioticStructure)>,
) {
    for (mut structure, symbiotic) in query.iter_mut() {
        if !symbiotic.is_dormant {
            structure.current_hp += symbiotic.regeneration_rate;
            if structure.current_hp > structure.max_hp {
                structure.current_hp = structure.max_hp;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Environment Integration:** The `EnvironmentStatus` component is a placeholder. The real implementation must query the global/tile environment data (e.g., `GridPosition` mapped to environmental temperature and light grids).
- **Dormancy Effects:** Dormancy shouldn't just halt regeneration. It needs to disable the core functionality of the building. For instance, if it's a symbiotic power plant, entering dormancy should sever its power generation output.
- **Consumption:** Symbiotic structures should consume Food/Biomass passively per tick. Add a system that deducts from `ColonyResources`. If food runs out, they should take starvation damage, decaying rather than regenerating.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `symbiotic_infrastructure`.
- [ ] Symbiotic structures regenerate their own HP over time up to max.
- [ ] Symbiotic structures become dormant when environmental needs (light/temp) are unmet.
- [ ] Regeneration halts when the structure is dormant.

## 7. Technical Guidance
- **System Ordering:** Ensure `process_symbiotic_needs_system` runs *before* `process_symbiotic_regeneration_system` and any building functionality systems so the dormancy state is accurately reflected.
- **Building Types:** Consider adding a `Symbiotic` tag to the `BuildingType` enum to allow UI filtering (e.g., separating "Construct Building" vs "Grow Symbiote" in the player menu).
- **Events:** Emit a `SymbioticDormancyEvent` when a structure falls asleep so the Chronicle system or UI can notify the player of the impending cascade failure.

## 8. Questions
*Builder: add questions here if spec is unclear.*
