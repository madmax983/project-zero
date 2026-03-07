# 403: Dysonian Swarms

## 1. Overview
The ultimate endgame energy solution. By launching thousands of solar collectors into close stellar orbit (Layer 2), the player can drastically increase their civilization's global energy availability.

However, capturing a star's output comes with a cost: it physically blocks sunlight from reaching the planets in the system. Launching a Dysonian Swarm permanently reduces "Insolation" (sunlight and heat) on Layer 1, plunging the colony into an artificial ice age and destroying traditional agriculture.

## 2. Dependencies
- `042-energy-system.md` (Base colony power grid)
- `094-system-view.md` (System nodes, including the Star)
- `105-launch-logistics.md` (For sending collectors to orbit)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::system::Star;
    use crate::layer1::weather::{Climate, Temperature};
    use crate::layer1::power::GlobalPowerNet;

    #[test]
    fn test_swarm_increases_global_power() {
        let mut app = App::new();
        app.add_systems(Update, calculate_swarm_power_system);

        app.insert_resource(GlobalPowerNet { available: 100, consumed: 0 });

        let star_ent = app.world_mut().spawn((
            Star { luminosity: 100.0 },
            DysonianSwarm { collectors_active: 50 },
        )).id();

        app.update();

        let net = app.world().get_resource::<GlobalPowerNet>().unwrap();
        assert!(net.available > 100, "Swarm should add power to the global net");
    }

    #[test]
    fn test_swarm_reduces_planetary_insolation_and_temp() {
        let mut app = App::new();
        app.add_systems(Update, apply_swarm_shadow_system);

        let star_ent = app.world_mut().spawn((
            Star { luminosity: 100.0 },
            DysonianSwarm { collectors_active: 100 }, // High density
        )).id();

        // Target planet climate
        let planet_ent = app.world_mut().spawn(Climate {
            base_insolation: 1.0,
            current_insolation: 1.0,
            base_temperature: 20.0,
            current_temperature: 20.0,
        }).id();

        app.update();

        let climate = app.world().get::<Climate>(planet_ent).unwrap();
        assert!(climate.current_insolation < 1.0, "Insolation should be reduced");
        assert!(climate.current_temperature < 20.0, "Temperature should drop due to shadow");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer2::system::Star;
use crate::layer1::power::GlobalPowerNet;
use crate::layer1::weather::Climate;

#[derive(Component)]
pub struct DysonianSwarm {
    pub collectors_active: u32,
}

pub fn calculate_swarm_power_system(
    mut power_net: ResMut<GlobalPowerNet>,
    swarm_query: Query<(&Star, &DysonianSwarm)>,
) {
    for (star, swarm) in swarm_query.iter() {
        // Mock formula: 10 power per collector, scaled by star luminosity
        let power_generated = (swarm.collectors_active as f32 * 10.0 * (star.luminosity / 100.0)) as u32;
        power_net.available += power_generated;
    }
}

pub fn apply_swarm_shadow_system(
    swarm_query: Query<&DysonianSwarm, With<Star>>,
    mut climate_query: Query<&mut Climate>,
) {
    let mut total_collectors = 0;
    for swarm in swarm_query.iter() {
        total_collectors += swarm.collectors_active;
    }

    // Shadow factor maxes out at 0.9 (90% blocked) at 1000 collectors
    let shadow_factor = (total_collectors as f32 / 1000.0).clamp(0.0, 0.9);

    for mut climate in climate_query.iter_mut() {
        climate.current_insolation = climate.base_insolation * (1.0 - shadow_factor);

        // Rough temperature scaling: drops drastically as light is blocked
        let temp_drop = 50.0 * shadow_factor; // Max 45 degree drop
        climate.current_temperature = climate.base_temperature - temp_drop;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Maintenance:** Collectors in the swarm should occasionally be destroyed by solar flares, requiring constant replacement launches to maintain output.
- **Lore Context:** Emitting a chronicle event when the swarm first becomes visible from the planet's surface (e.g., "The sky is caged").
- **UI:** The sky renderer should draw a subtle grid or dim the directional light based on `shadow_factor`.

## 6. Acceptance Criteria (Testable!)
- [ ] `DysonianSwarm` component correctly parses on a Star.
- [ ] Active collectors increase the `GlobalPowerNet.available`.
- [ ] Active collectors decrease the `Climate` insolation and temperature.
- [ ] Tests pass cleanly.

## 7. Technical Guidance
- `apply_swarm_shadow_system` should probably run in an environmental update schedule (e.g., hourly in-game), not every tick, since temperature changes slowly.

## 8. Questions
*Builder: add questions here if spec is unclear.*
