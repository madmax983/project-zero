# 444: Orbital Kessler Rings

## 1. Overview
Every ship destroyed in orbit around the planet adds debris. Over time, this forms a Kessler ring. The ring provides excellent scavenging opportunities but blocks sunlight to Layer 1 (reducing solar power and crop yields) and makes launching new ships extremely dangerous.

## 2. Dependencies
- `159` Fleet Combat Resolution (To generate debris)
- `105` Launch Logistics (For launch risk)
- `213` Solar Cycles (For light blocking logic)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::combat::ShipDestroyedEvent;

    #[test]
    fn test_ship_destruction_adds_debris() {
        let mut app = App::new();
        app.insert_resource(OrbitalDebris { density: 0.0 });
        app.add_event::<ShipDestroyedEvent>();
        app.add_system(accumulate_kessler_debris);

        app.world.resource_mut::<Events<ShipDestroyedEvent>>().send(ShipDestroyedEvent);
        app.update();

        let debris = app.world.resource::<OrbitalDebris>();
        assert!(debris.density > 0.0);
    }

    #[test]
    fn test_high_debris_reduces_solar_efficiency() {
        let mut app = App::new();
        app.insert_resource(OrbitalDebris { density: 0.8 }); // 80% density
        app.insert_resource(SolarIllumination { multiplier: 1.0 });
        app.add_system(apply_kessler_shadow);

        app.update();

        let solar = app.world.resource::<SolarIllumination>();
        // High density should reduce illumination
        assert!(solar.multiplier < 1.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct OrbitalDebris {
    pub density: f32, // 0.0 to 1.0
}

#[derive(Resource)]
pub struct SolarIllumination {
    pub multiplier: f32,
}

pub struct ShipDestroyedEvent;

pub fn accumulate_kessler_debris(
    mut events: EventReader<ShipDestroyedEvent>,
    mut debris: ResMut<OrbitalDebris>,
) {
    for _ in events.iter() {
        debris.density += 0.05;
        debris.density = debris.density.min(1.0);
    }
}

pub fn apply_kessler_shadow(
    debris: Res<OrbitalDebris>,
    mut solar: ResMut<SolarIllumination>,
) {
    // Inverse relationship: more debris = less light
    solar.multiplier = 1.0 - (debris.density * 0.5); // Max 50% reduction
}
```

## 5. REFACTOR Phase: Quality & Design
- Move `OrbitalDebris` to be a component on a `Planet` entity if the game supports multiple planets.
- The scalar `0.05` should depend on the size/mass of the destroyed ship.
- Hook into the `LaunchLogistics` system to apply a failure probability based on `debris.density`.

## 6. Acceptance Criteria
- [ ] `ShipDestroyedEvent` increases `OrbitalDebris` density.
- [ ] High `OrbitalDebris` reduces `SolarIllumination` multiplier.
- [ ] Debris density caps at 1.0.
- [ ] Tests pass and test coverage is ≥85%.

## 7. Technical Guidance
- Ensure that the `apply_kessler_shadow` system runs *before* systems that calculate solar power output or crop growth so the multiplier is applied correctly for that tick.

## 8. Questions
