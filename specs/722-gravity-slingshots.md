# Gravity Slingshots (Spec 722)

## 1. Overview
**Layer:** 2 (System)
**Fantasy:** Flying like a leaf on the wind.
**Mechanic:** Moving adjacent to massive gravity wells (Stars, Gas Giants) grants a "Slingshot" momentum bonus (free movement) for the next turn but locks the trajectory. Miscalculation flings ships into deep space or the atmosphere.
**Emergence:** A pirate fleet chases you. You dive into the gravity well of the local sun. You survive the G-force; they burn up.
**Tension:** Safe, slow direct path vs. Fast, dangerous gravity assist.

## 2. Dependencies
- `layer2::fleet::Fleet` movement systems.
- `layer2::system::OrbitalBody` with gravity definitions.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::fleet::{Fleet, FleetMovementEvent};
    use crate::layer2::system::{OrbitalBody, SystemBody};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<FleetMovementEvent>();
        app.add_systems(Update, process_gravity_slingshots);
        app
    }

    #[test]
    fn test_fleet_entering_gravity_well_gains_momentum() {
        let mut app = setup_app();

        let sun = app.world_mut().spawn((
            SystemBody,
            OrbitalBody { name: "Sol".to_string(), radius: 5.0, color: Color::YELLOW },
            GravityWell { radius: 10.0, strength: 2.0 },
            Transform::from_translation(Vec3::ZERO),
        )).id();

        let fleet = app.world_mut().spawn((
            Fleet::default(),
            Transform::from_translation(Vec3::new(15.0, 0.0, 0.0)),
            SlingshotMomentum { active: false, bonus: 0.0, locked_vector: Vec2::ZERO },
        )).id();

        // Move fleet into gravity well
        app.world_mut().get_mut::<Transform>(fleet).unwrap().translation = Vec3::new(8.0, 0.0, 0.0);

        app.update();

        let momentum = app.world().get::<SlingshotMomentum>(fleet).unwrap();
        assert!(momentum.active);
        assert_eq!(momentum.bonus, 2.0); // based on strength
    }

    #[test]
    fn test_fleet_with_momentum_moves_further_next_turn() {
        let mut app = setup_app();

        let fleet = app.world_mut().spawn((
            Fleet::default(),
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
            SlingshotMomentum { active: true, bonus: 2.0, locked_vector: Vec2::new(1.0, 0.0) },
        )).id();

        // Move fleet
        app.world_mut().send_event(FleetMovementEvent {
            fleet,
            target: Vec3::new(12.0, 0.0, 0.0), // Request 2 units of movement
        });

        app.update();

        let transform = app.world().get::<Transform>(fleet).unwrap();
        // Base movement 2 + bonus 2 = 4 units moved, but locked vector forces direction
        assert_eq!(transform.translation.x, 14.0);

        // Momentum should be consumed
        let momentum = app.world().get::<SlingshotMomentum>(fleet).unwrap();
        assert!(!momentum.active);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer2::fleet::Fleet;
use crate::layer2::system::SystemBody;

#[derive(Component)]
pub struct GravityWell {
    pub radius: f32,
    pub strength: f32,
}

#[derive(Component, Default)]
pub struct SlingshotMomentum {
    pub active: bool,
    pub bonus: f32,
    pub locked_vector: Vec2,
}

#[derive(Event)]
pub struct FleetMovementEvent {
    pub fleet: Entity,
    pub target: Vec3,
}

pub fn process_gravity_slingshots(
    mut fleets: Query<(&mut Transform, &mut SlingshotMomentum), With<Fleet>>,
    gravity_wells: Query<(&Transform, &GravityWell), (With<SystemBody>, Without<Fleet>)>,
) {
    for (fleet_transform, mut momentum) in fleets.iter_mut() {
        // If no active momentum, check for entry into a gravity well
        if !momentum.active {
            for (well_transform, well) in gravity_wells.iter() {
                let distance = fleet_transform.translation.distance(well_transform.translation);
                if distance <= well.radius {
                    momentum.active = true;
                    momentum.bonus = well.strength;

                    // Simple tangent calculation for locked vector
                    let dir = (fleet_transform.translation - well_transform.translation).normalize();
                    momentum.locked_vector = Vec2::new(-dir.y, dir.x); // orthogonal
                    break;
                }
            }
        }
    }
}

pub fn process_fleet_movement_with_slingshot(
    mut events: EventReader<FleetMovementEvent>,
    mut fleets: Query<(&mut Transform, &mut SlingshotMomentum), With<Fleet>>,
) {
    for event in events.read() {
        if let Ok((mut transform, mut momentum)) = fleets.get_mut(event.fleet) {
            let requested_move = event.target - transform.translation;
            let move_distance = requested_move.length();
            let move_dir = requested_move.normalize_or_zero();

            if momentum.active {
                // Apply locked trajectory and bonus distance
                let total_distance = move_distance + momentum.bonus;
                transform.translation += Vec3::new(momentum.locked_vector.x, momentum.locked_vector.y, 0.0) * total_distance;

                // Consume momentum
                momentum.active = false;
                momentum.bonus = 0.0;
            } else {
                // Normal movement
                transform.translation += move_dir * move_distance;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathing Conflicts**: What if the slingshot vector points directly into a planet or hazard? Integrate collision detection to deal hull damage (the "miscalculation flings ships into atmosphere" fantasy).
- **Control Loss**: The locked trajectory should override player pathfinding for that tick. Ensure UI path preview reflects this forced vector.
- **Velocity Tracking**: Layer 2 might not use continuous velocity physics. Ensure this fits cleanly within the turn/tick-based movement system.

## 6. Acceptance Criteria
- [ ] `GravityWell` component creates slingshot zones.
- [ ] `SlingshotMomentum` activates when entering the well.
- [ ] Active momentum forces the next movement along a locked vector with bonus distance.
- [ ] Miscalculations (hitting celestial bodies while locked) deal hull damage.
- [ ] Over 85% test coverage.

## 7. Technical Guidance
- Ensure `FleetMovementEvent` exists or integrate tightly with whatever `layer2::fleet` uses for pathing (e.g., `process_fleet_movement`).
- Orbit lines/UI pathing needs to calculate the slingshot vector ahead of time so the player knows what direction they are committing to.

## 8. Questions
- How long does the momentum last? Just one tick/movement order?
- Should gravity wells have varying "bands" of strength?
