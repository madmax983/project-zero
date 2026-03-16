# 468: Escape Velocity Economics

## 1. Overview
Physics dictates the economy. Launch costs from Layer 1 to Layer 2 are proportional to planetary gravity. On High-G worlds, exporting heavy raw materials is unprofitable because fuel costs far exceed the resource value. You must refine them into high-value, low-mass tech. This forces players to adapt their economic and industrial strategies based on the physical properties of the planet they settle.

## 2. Dependencies
- `039` Trade System
- `105` Launch Logistics
- `080` Planetary Quirks

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    // Assuming these components/resources exist
    // use scale_core::layer2::planet::PlanetaryGravity;
    // use scale_core::layer1::trade::{TradeManifest, CargoItem};
    // use scale_core::layer1::logistics::FuelReserve;

    #[derive(Resource)]
    struct PlanetaryGravity {
        pub g_force: f32, // 1.0 is Earth normal
    }

    #[derive(Component, Clone)]
    struct CargoItem {
        pub mass: f32, // in tons
        pub value: f32, // in credits
    }

    #[derive(Component)]
    struct TradeManifest {
        pub items: Vec<CargoItem>,
    }

    fn calculate_launch_cost(gravity: &PlanetaryGravity, manifest: &TradeManifest) -> f32 {
        let total_mass: f32 = manifest.items.iter().map(|i| i.mass).sum();
        // Base cost + (Mass * Gravity * Fuel Constant)
        100.0 + (total_mass * gravity.g_force * 10.0)
    }

    #[test]
    fn test_high_gravity_increases_launch_cost_significantly() {
        let gravity_normal = PlanetaryGravity { g_force: 1.0 };
        let gravity_high = PlanetaryGravity { g_force: 2.5 };

        let manifest = TradeManifest {
            items: vec![
                CargoItem { mass: 50.0, value: 500.0 }, // 50 tons of iron ore
            ],
        };

        let cost_normal = calculate_launch_cost(&gravity_normal, &manifest);
        let cost_high = calculate_launch_cost(&gravity_high, &manifest);

        assert!(cost_high > cost_normal * 2.0, "High gravity should vastly increase launch costs");
        assert_eq!(cost_normal, 600.0); // 100 + (50 * 1 * 10)
        assert_eq!(cost_high, 1350.0); // 100 + (50 * 2.5 * 10)
    }

    #[test]
    fn test_low_mass_high_value_goods_are_profitable_on_high_g() {
        let gravity_high = PlanetaryGravity { g_force: 2.5 };

        let raw_ore = TradeManifest {
            items: vec![CargoItem { mass: 100.0, value: 1000.0 }],
        };

        let refined_chips = TradeManifest {
            items: vec![CargoItem { mass: 5.0, value: 5000.0 }],
        };

        let cost_ore = calculate_launch_cost(&gravity_high, &raw_ore);
        let cost_chips = calculate_launch_cost(&gravity_high, &refined_chips);

        let profit_ore = raw_ore.items[0].value - cost_ore;
        let profit_chips = refined_chips.items[0].value - cost_chips;

        assert!(profit_ore < 0.0, "Exporting heavy raw ore on High G should be unprofitable");
        assert!(profit_chips > 0.0, "Exporting light refined tech on High G should be profitable");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct PlanetaryGravity {
    pub g_force: f32, // e.g., 0.5 for a moon, 1.0 for Earth, 2.5 for a super-earth
}

impl Default for PlanetaryGravity {
    fn default() -> Self {
        Self { g_force: 1.0 }
    }
}

#[derive(Component, Clone)]
pub struct CargoItem {
    pub mass: f32,
    pub value: f32,
}

#[derive(Component)]
pub struct TradeManifest {
    pub items: Vec<CargoItem>,
}

#[derive(Event)]
pub struct LaunchShipEvent {
    pub manifest_entity: Entity,
}

pub fn calculate_launch_cost(gravity: &PlanetaryGravity, manifest: &TradeManifest) -> f32 {
    let total_mass: f32 = manifest.items.iter().map(|i| i.mass).sum();
    // Base cost 100.0 fuel units + 10.0 fuel units per ton per G
    100.0 + (total_mass * gravity.g_force * 10.0)
}

// In the actual system processing the launch, we would deduct this cost from the Colony's FuelReserve
pub fn process_launch_system(
    gravity: Res<PlanetaryGravity>,
    mut events: EventReader<LaunchShipEvent>,
    query: Query<&TradeManifest>,
    // mut fuel: ResMut<FuelReserve>,
) {
    for event in events.read() {
        if let Ok(manifest) = query.get(event.manifest_entity) {
            let cost = calculate_launch_cost(&gravity, manifest);
            // Example pseudo-code:
            // if fuel.amount >= cost {
            //     fuel.amount -= cost;
            //     // Launch the ship
            // } else {
            //     // Launch failed due to insufficient fuel
            // }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Fuel Types**: Introduce different grades of fuel (e.g., Chemical vs Antimatter). High-G launches might strictly require advanced fuels, making early-game extraction impossible.
- **Space Elevators**: Once built, Space Elevators should bypass the gravity-mass penalty entirely, serving as a massive late-game economic shift for High-G worlds.
- **UI Integration**: The Trade UI must clearly display the "Launch Cost" vs "Cargo Value" to prevent players from accidentally bankrupting themselves by launching rocks into orbit.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Launch costs scale linearly with the product of `PlanetaryGravity` and total cargo mass.

## 7. Technical Guidance
- Ensure `CargoItem` mass values are balanced. A stack of 100 steel plates should be heavy, while a stack of 100 quantum processors should be light.
- The `PlanetaryGravity` resource should be initialized during world generation based on the planet's traits (`080 Planetary Quirks`).

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
