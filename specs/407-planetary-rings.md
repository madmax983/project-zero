# 407: Planetary Rings

## 1. Overview
Not all planets are simple spheres. Planets can spawn with "Planetary Rings" in Layer 2.

Rings represent incredibly dense, high-yield resource zones for mining ships. However, they are composed of high-velocity micro-debris. Flying a fleet through or stationing it within the ring causes constant structural attrition (damage). Additionally, the ring casts a massive dynamic shadow on the Layer 1 colony surface, affecting solar power and temperature based on the planet's axial tilt and seasonal rotation.

## 2. Dependencies
- `094-system-view.md` (System generation)
- `184-orbital-debris.md` (For the attrition/damage logic, which rings will heavily utilize)
- `404-thermal-management.md` (For the shadow/temperature impacts)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::system::SystemBody;
    use crate::layer2::movement::Position;
    use crate::layer2::fleet::{Fleet, FleetHealth};

    #[test]
    fn test_ring_attrition_damage() {
        let mut app = App::new();
        app.add_systems(Update, ring_attrition_system);

        let ring_ent = app.world_mut().spawn((
            SystemBody,
            PlanetaryRing { radius_min: 5.0, radius_max: 10.0, density: 0.8 },
            Position { x: 0.0, y: 0.0 }, // Center of planet
        )).id();

        // Fleet inside the ring (distance 7.0)
        let fleet_ent = app.world_mut().spawn((
            Fleet,
            FleetHealth { current: 100.0, max: 100.0 },
            Position { x: 7.0, y: 0.0 },
        )).id();

        app.update();

        let health = app.world().get::<FleetHealth>(fleet_ent).unwrap();
        assert!(health.current < 100.0, "Fleet inside ring should take density-based damage");
    }

    #[test]
    fn test_fleet_outside_ring_takes_no_damage() {
        let mut app = App::new();
        app.add_systems(Update, ring_attrition_system);

        let ring_ent = app.world_mut().spawn((
            SystemBody,
            PlanetaryRing { radius_min: 5.0, radius_max: 10.0, density: 0.8 },
            Position { x: 0.0, y: 0.0 },
        )).id();

        // Fleet outside the ring (distance 15.0)
        let fleet_ent = app.world_mut().spawn((
            Fleet,
            FleetHealth { current: 100.0, max: 100.0 },
            Position { x: 15.0, y: 0.0 },
        )).id();

        app.update();

        let health = app.world().get::<FleetHealth>(fleet_ent).unwrap();
        assert_eq!(health.current, 100.0, "Fleet outside ring should not take damage");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer2::system::SystemBody;
use crate::layer2::movement::Position;
use crate::layer2::fleet::{Fleet, FleetHealth};

#[derive(Component)]
pub struct PlanetaryRing {
    pub radius_min: f32,
    pub radius_max: f32,
    pub density: f32, // Multiplier for damage and resource yield
}

pub fn ring_attrition_system(
    ring_query: Query<(&PlanetaryRing, &Position)>,
    mut fleet_query: Query<(&mut FleetHealth, &Position), With<Fleet>>,
) {
    for (ring, r_pos) in ring_query.iter() {
        for (mut health, f_pos) in fleet_query.iter_mut() {
            let dx = r_pos.x - f_pos.x;
            let dy = r_pos.y - f_pos.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist >= ring.radius_min && dist <= ring.radius_max {
                let damage = ring.density * 2.0; // Base damage per tick
                health.current = (health.current - damage).max(0.0);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Resource Mining:** Integrate with the `SystemMining` module so that placing a mining ship in the ring yields highly valuable rare resources (like "Diamond Dust").
- **Shadow Projection:** In Layer 1, the `Climate` system should check if the planet has a `PlanetaryRing`. If so, a certain band of tiles on the Y-axis should receive a massive penalty to Insolation.
- **Visuals:** The Layer 2 renderer should draw a circular gradient around the planet representing the ring bounds.

## 6. Acceptance Criteria (Testable!)
- [ ] `PlanetaryRing` component added and defines an inner/outer boundary.
- [ ] Fleets positioned within the boundary continuously lose `FleetHealth`.
- [ ] Tests pass cleanly.

## 7. Technical Guidance
- Distance checking can be expensive. If fleets become numerous, only check fleets that are orbiting the specific planet that owns the ring.

## 8. Questions
*Builder: add questions here if spec is unclear.*
