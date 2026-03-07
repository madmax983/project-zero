# 401: Asteroid Redirect

## 1. Overview
The player can use specialized Layer 2 orbital ships (Tugs) or construct "Mass Drivers" directly on asteroids to alter their trajectories.

An asteroid can be redirected into a stable planetary orbit for safe, long-term mining operations. Alternatively, it can be redirected as a massive kinetic weapon, devastating an enemy planet or fleet, but completely destroying the resource potential of the rock.

## 2. Dependencies
- `094-system-view.md` (System node and Asteroid entities)
- `101-system-mining.md` (Asteroid resources)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::movement::{Position, Velocity};
    use crate::layer2::system::SystemBody;

    #[test]
    fn test_asteroid_redirect_changes_velocity() {
        let mut app = App::new();
        app.add_event::<RedirectAsteroidEvent>();
        app.add_systems(Update, process_asteroid_redirect_system);

        let asteroid = app.world_mut().spawn((
            SystemBody,
            Position { x: 0.0, y: 0.0 },
            Velocity { x: 1.0, y: 0.0 }, // Moving right
        )).id();

        // Redirect it to move UP
        app.world_mut().send_event(RedirectAsteroidEvent {
            target_asteroid: asteroid,
            new_velocity: Velocity { x: 0.0, y: 1.0 },
        });

        app.update();

        let velocity = app.world().get::<Velocity>(asteroid).unwrap();
        assert_eq!(velocity.x, 0.0);
        assert_eq!(velocity.y, 1.0);
    }

    #[test]
    fn test_asteroid_kinetic_impact() {
        let mut app = App::new();
        app.add_systems(Update, check_asteroid_impacts_system);

        // Planet at 10,10
        let planet = app.world_mut().spawn((
            SystemBody,
            crate::layer2::system::Planet { health: 1000.0 },
            Position { x: 10.0, y: 10.0 },
        )).id();

        // Asteroid hits planet
        let asteroid = app.world_mut().spawn((
            SystemBody,
            KineticWeapon { damage: 500.0 },
            Position { x: 10.0, y: 10.0 }, // Same pos
        )).id();

        app.update();

        // Planet takes damage
        let p_data = app.world().get::<crate::layer2::system::Planet>(planet).unwrap();
        assert_eq!(p_data.health, 500.0);

        // Asteroid is destroyed
        assert!(app.world().get_entity(asteroid).is_err() || app.world().get::<SystemBody>(asteroid).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer2::movement::{Position, Velocity};
use crate::layer2::system::{SystemBody, Planet};

#[derive(Component)]
pub struct KineticWeapon {
    pub damage: f32,
}

#[derive(Event)]
pub struct RedirectAsteroidEvent {
    pub target_asteroid: Entity,
    pub new_velocity: Velocity,
}

pub fn process_asteroid_redirect_system(
    mut events: EventReader<RedirectAsteroidEvent>,
    mut query: Query<&mut Velocity, With<SystemBody>>,
) {
    for event in events.read() {
        if let Ok(mut vel) = query.get_mut(event.target_asteroid) {
            vel.x = event.new_velocity.x;
            vel.y = event.new_velocity.y;
            // Note: We might also attach the KineticWeapon component here if speed > threshold
        }
    }
}

pub fn check_asteroid_impacts_system(
    mut commands: Commands,
    asteroid_query: Query<(Entity, &KineticWeapon, &Position)>,
    mut planet_query: Query<(&mut Planet, &Position)>,
) {
    for (a_ent, weapon, a_pos) in asteroid_query.iter() {
        for (mut planet, p_pos) in planet_query.iter_mut() {
            let dx = a_pos.x - p_pos.x;
            let dy = a_pos.y - p_pos.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < 1.0 { // Collision radius
                planet.health = (planet.health - weapon.damage).max(0.0);
                commands.entity(a_ent).despawn();
                // To avoid multiple impacts if iterating multiple planets (edge case), break
                break;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Stable Orbits:** Establishing a stable orbit shouldn't just be setting a vector. It should change the asteroid's pathing component to an `InOrbit` component tied to the planet.
- **Failures:** Miscalculating thrust vectors or experiencing an engine failure mid-burn should result in the asteroid de-orbiting into the planet accidentally.
- **UI:** A specialized targeting UI arrow vector should be added to the System Map.

## 6. Acceptance Criteria (Testable!)
- [ ] `RedirectAsteroidEvent` updates the velocity of a SystemBody.
- [ ] `KineticWeapon` components cause damage to `Planet` health upon collision.
- [ ] Asteroids are despawned upon impact.
- [ ] Tests pass cleanly.

## 7. Technical Guidance
- Integrate with Layer 1: An impact should trigger an `ImpactEvent` in `layer1` to obliterate tiles on the surface, not just subtract health points.

## 8. Questions
*Builder: add questions here if spec is unclear.*
