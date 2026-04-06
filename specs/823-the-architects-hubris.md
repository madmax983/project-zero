# 823: The Architect's Hubris

## 1. Overview
Constructing massive "Mega-Spires" allows colonies to bypass shuttle launches and grants immense localized bonuses. However, these colossal structures fundamentally disrupt the local atmosphere. They alter weather patterns, drawing catastrophic hyper-storms and creating immense wind shear at their base. This forces a trade-off: the unmatched logistics and prestige of the spire versus the localized environmental devastation and the spiraling, constant maintenance costs required to keep the base from collapsing under the self-inflicted storms.

## 2. Dependencies
- `src/layer1/buildings.rs` for structures and maintenance mechanics.
- `src/layer1/nature/weather.rs` for local weather and storm events.
- `src/layer1/pop.rs` for handling Pop morale/stress regarding localized conditions.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_mega_spire_generates_hyper_storms_locally() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<WeatherSystem>();
        app.add_systems(Update, process_spire_weather_effects);

        let spire_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::MegaSpire, ..default() },
            Position { x: 10, y: 10 },
            MegaSpire { height: 100 },
        )).id();

        // Act
        app.update();

        // Assert
        let weather = app.world().resource::<WeatherSystem>();
        let local_weather = weather.get_condition_at(10, 10);
        // The presence of the Spire should force extreme weather (HyperStorm) at its location
        assert_eq!(local_weather, WeatherCondition::HyperStorm);
    }

    #[test]
    fn test_mega_spire_maintenance_scales_with_storm_intensity() {
        // Test that a MegaSpire in a HyperStorm tile drains significantly more maintenance resources
        // compared to standard buildings to prevent collapse.
    }

    #[test]
    fn test_pops_at_spire_base_suffer_stress_penalties() {
        // Test that Pops assigned to work/live at the base of the Spire (experiencing the storm)
        // take a steady drain to morale/stress due to the brutal conditions.
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct MegaSpire {
    pub height: u32,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum WeatherCondition {
    Clear,
    Rain,
    HyperStorm,
}

#[derive(Resource, Default)]
pub struct WeatherSystem {
    // simplified grid mapping
    pub conditions: std::collections::HashMap<(i32, i32), WeatherCondition>,
}

impl WeatherSystem {
    pub fn get_condition_at(&self, x: i32, y: i32) -> WeatherCondition {
        *self.conditions.get(&(x, y)).unwrap_or(&WeatherCondition::Clear)
    }

    pub fn set_condition(&mut self, x: i32, y: i32, condition: WeatherCondition) {
        self.conditions.insert((x, y), condition);
    }
}

pub fn process_spire_weather_effects(
    mut weather: ResMut<WeatherSystem>,
    spire_query: Query<&Position, With<MegaSpire>>,
) {
    for pos in spire_query.iter() {
        // MegaSpires inevitably cause HyperStorms at their exact location
        weather.set_condition(pos.x, pos.y, WeatherCondition::HyperStorm);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate with existing `MaintenanceSystem` to apply dynamic resource drain rather than static costs.
- Smooth the weather gradient so the storm effect spreads slightly beyond just the central tile (e.g., intense at 0 distance, mild at 1 distance).
- Hook up orbital launch capabilities to actually provide the intended benefit of the spire.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Spire construction forces `WeatherCondition::HyperStorm` on its tile.

## 7. Technical Guidance
- Ensure the `Maintenance` cost modifier applied by the storm doesn't accidentally cause negative resource ticks if the player runs out of materials; instead, it should damage the building if unpaid.
- Use the spatial grid to cleanly separate the "high class" (upper levels of the spire, unaffected) vs "working class" (base level, in the storm) mechanics if Pop altitude tracking is implemented.

## 8. Questions
*Builder: add questions here if spec is unclear.*
