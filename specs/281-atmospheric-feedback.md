# 281: Atmospheric Feedback

## 1. Overview
The planet reacts to your industry. You are not just building on the map; you are changing the map. Heavy industry generates `Smog`. Smog reduces solar power and happiness but increases "Industrial Gloom". Trees absorb Smog. Pollution levels affect Layer 2 planet stats (habitability).

## 2. Dependencies
- `063` Atmospheric Simulation
- `042` Energy System
- `019` Forestry System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(GlobalAtmosphere { smog_level: 0.0 });
        app.add_systems(Update, (produce_smog_system, absorb_smog_system, apply_smog_effects_system));
        app
    }

    #[test]
    fn test_heavy_industry_increases_smog() {
        let mut app = setup_app();

        app.world_mut().spawn(HeavyIndustry { smog_output: 5.0 });

        app.update();

        let atmos = app.world().get_resource::<GlobalAtmosphere>().unwrap();
        assert_eq!(atmos.smog_level, 5.0);
    }

    #[test]
    fn test_trees_absorb_smog() {
        let mut app = setup_app();
        app.world_mut().resource_mut::<GlobalAtmosphere>().smog_level = 10.0;

        app.world_mut().spawn(Tree { smog_absorption: 2.0 });

        app.update();

        let atmos = app.world().get_resource::<GlobalAtmosphere>().unwrap();
        assert_eq!(atmos.smog_level, 8.0);
    }

    #[test]
    fn test_smog_reduces_solar_efficiency() {
        let mut app = setup_app();
        app.world_mut().resource_mut::<GlobalAtmosphere>().smog_level = 50.0;

        let solar_panel = app.world_mut().spawn(SolarPanel { base_output: 100.0, current_output: 100.0 }).id();

        app.update();

        let panel = app.world().get::<SolarPanel>(solar_panel).unwrap();
        assert!(panel.current_output < 100.0); // Output reduced by smog
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct GlobalAtmosphere {
    pub smog_level: f32,
}

#[derive(Component)]
pub struct HeavyIndustry {
    pub smog_output: f32,
}

#[derive(Component)]
pub struct Tree {
    pub smog_absorption: f32,
}

#[derive(Component)]
pub struct SolarPanel {
    pub base_output: f32,
    pub current_output: f32,
}

pub fn produce_smog_system(
    mut atmos: ResMut<GlobalAtmosphere>,
    industry: Query<&HeavyIndustry>,
) {
    for ind in industry.iter() {
        atmos.smog_level += ind.smog_output;
    }
}

pub fn absorb_smog_system(
    mut atmos: ResMut<GlobalAtmosphere>,
    trees: Query<&Tree>,
) {
    for tree in trees.iter() {
        atmos.smog_level = (atmos.smog_level - tree.smog_absorption).max(0.0);
    }
}

pub fn apply_smog_effects_system(
    atmos: Res<GlobalAtmosphere>,
    mut panels: Query<&mut SolarPanel>,
) {
    // 1% reduction per 10 smog, max 50% reduction
    let penalty = (atmos.smog_level / 1000.0).clamp(0.0, 0.5);

    for mut panel in panels.iter_mut() {
        panel.current_output = panel.base_output * (1.0 - penalty);
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- Move `smog_level` tracking to a spatial grid if you want localized smog instead of a single global resource. A grid allows wind to blow smog away from industrial centers into residential ones.
- Add `SmogEmitter` component to existing buildings (Refineries, Generators) rather than a generic `HeavyIndustry` tag.
- Ensure the `Tree` absorption scales with tree growth stage if using Ecological Succession (Spec 161).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Buildings designated as heavy industry increase global smog.
- [ ] Trees reduce global smog.
- [ ] High smog levels negatively impact solar panel energy generation.

## 7. Technical Guidance
- Implement in `src/layer1/environment/atmosphere.rs`.
- Tie the `smog_level` variable into the Layer 2 planet habitability stats via a bridge system in `src/layer1/integration.rs`.
- Update the Pop mood system to apply a "Gloomy" debuff when global smog passes a specific threshold.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
