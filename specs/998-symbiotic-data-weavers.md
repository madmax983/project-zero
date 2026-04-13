# 998: The Symbiotic Data-Weavers

## 1. Overview
Building a supercomputer out of living, breathing alien flora. Certain bioluminescent flora on Layer 1 can be genetically altered to act as optical data processors. Instead of building massive, power-hungry silicon server farms, players cultivate "Data Forests." These forests process Layer 3 administrative tasks and tech research but require perfect ecological balance, water, and specific soil nutrients rather than raw electricity.
If a severe drought hits the colony, the resulting "wilting" of the network crashes the entire planetary administrative grid and erases ongoing tech research.

## 2. Dependencies
- Layer 1 Flora/Farming mechanics.
- Layer 1 Water/Ecology systems.
- Layer 3 Tech/Administrative processing logic.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_data_forest_processing() {
        let mut app = App::new();
        app.add_systems(Update, process_data_forest_system);

        app.world_mut().spawn((
            DataForest { processing_power: 10.0 },
            FloraState::Healthy,
            WaterSupply { current: 100.0, required: 10.0 },
        ));

        app.world_mut().insert_resource(TechResearch { progress: 0.0 });

        app.update();

        let research = app.world().get_resource::<TechResearch>().unwrap();
        assert_eq!(research.progress, 10.0);
    }

    #[test]
    fn test_data_forest_wilting() {
        let mut app = App::new();
        app.add_systems(Update, process_data_forest_system);

        app.world_mut().spawn((
            DataForest { processing_power: 10.0 },
            FloraState::Wilting,
            WaterSupply { current: 0.0, required: 10.0 },
        ));

        app.world_mut().insert_resource(TechResearch { progress: 50.0 });

        app.update();

        let research = app.world().get_resource::<TechResearch>().unwrap();
        // Progress erased due to wilting
        assert_eq!(research.progress, 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct DataForest {
    pub processing_power: f32,
}

#[derive(Component, PartialEq)]
pub enum FloraState {
    Healthy,
    Wilting,
}

#[derive(Component)]
pub struct WaterSupply {
    pub current: f32,
    pub required: f32,
}

#[derive(Resource)]
pub struct TechResearch {
    pub progress: f32,
}

pub fn process_data_forest_system(
    query: Query<(&DataForest, &FloraState, &WaterSupply)>,
    mut research: Option<ResMut<TechResearch>>,
) {
    if let Some(mut research) = research {
        for (forest, state, water) in query.iter() {
            if *state == FloraState::Wilting || water.current < water.required {
                research.progress = 0.0;
            } else if *state == FloraState::Healthy {
                research.progress += forest.processing_power;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Erasing all `TechResearch` progress globally just because one forest is wilting might be too harsh. We should tie the loss to specific tech or proportional reduction.
- **Performance**: We can sum processing power of healthy forests and apply it once, rather than iterating and applying directly to the resource inside the loop.
- **Design Improvements**: Link this with the administrative grid capacity properly. Use events for "Wilting Crash" to notify the player.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Data forests process tech/admin tasks when healthy.
- [ ] Data forests erase or significantly reduce progress when wilting due to water lack.

## 7. Technical Guidance
- Integrate with existing ecology/flora systems, likely `layer1::flora` or `layer1::ecology`.
- Ensure `WaterSupply` connects properly to the planet's water grid.
- Send an alert or notification via `AddChronicleEvent` when a data forest crashes to let the player know they lost research data.

## 8. Questions
*Builder: add questions here if spec is unclear.*
