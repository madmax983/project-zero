# 095: System Generation

## Overview

Implements procedural generation for Layer 2 (System View). At game start, the game will generate a star system consisting of a central Star, orbiting Planets, and Moons.

One planet will be selected as the `ColonyLocation` (where the player starts on Layer 1). The generated traits of this planet (e.g., Gravity, Atmosphere) will be propagated to the global `PlanetaryTraits` resource defined in Spec 080, linking the system simulation to the colony simulation.

## Dependencies

- `094` — System View Architecture (for `OrbitalBody`, `Orbit` components)
- `080` — Planetary Quirks (for `PlanetaryTraits` resource and `PlanetaryTrait` enum)
- `001` — Project Scaffold (for `App` structure)

## RED Phase: Tests First

Write these tests in `src/layer2/generation_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::system::{OrbitalBody, Orbit};
    use crate::layer2::generation::{generate_system, Star, Planet, Moon, ColonyLocation, PlanetaryTraitsComponent};
    use crate::layer1::quirks::{PlanetaryTraits, PlanetaryTrait};

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(PlanetaryTraits::default());
        // Register required components if using reflection (optional for tests)
        world
    }

    #[test]
    fn test_generate_system_creates_star() {
        let mut world = setup_world();

        // Run generation system
        let mut schedule = Schedule::default();
        schedule.add_systems(generate_system);
        schedule.run(&mut world);

        // Assert exactly one Star exists
        let stars = world.query::<&Star>().iter(&world).count();
        assert_eq!(stars, 1, "Should generate exactly one star");

        // Assert Star has OrbitalBody (visuals) but NO Orbit (it's the center)
        let (star_entity, _) = world.query::<(Entity, &Star)>().single(&world);
        assert!(world.get::<OrbitalBody>(star_entity).is_some());
        assert!(world.get::<Orbit>(star_entity).is_none());
    }

    #[test]
    fn test_generate_system_creates_planets() {
        let mut world = setup_world();
        let mut schedule = Schedule::default();
        schedule.add_systems(generate_system);
        schedule.run(&mut world);

        // Assert at least one Planet exists
        let planet_count = world.query::<&Planet>().iter(&world).count();
        assert!(planet_count > 0, "Should generate at least one planet");

        // Assert Planets orbit the Star
        let (star_entity, _) = world.query::<(Entity, &Star)>().single(&world);
        for (planet_entity, orbit) in world.query::<(Entity, &Orbit)>().iter(&world) {
            // Check if this entity is a planet
            if world.get::<Planet>(planet_entity).is_some() {
                assert_eq!(orbit.parent, star_entity, "Planet should orbit the star");
            }
        }
    }

    #[test]
    fn test_colony_location_assignment() {
        let mut world = setup_world();
        let mut schedule = Schedule::default();
        schedule.add_systems(generate_system);
        schedule.run(&mut world);

        // Assert exactly one ColonyLocation exists
        let colony_locations = world.query::<&ColonyLocation>().iter(&world).count();
        assert_eq!(colony_locations, 1, "Should assign exactly one colony location");

        // Assert ColonyLocation is on a Planet
        let (colony_entity, _) = world.query::<(Entity, &ColonyLocation)>().single(&world);
        assert!(world.get::<Planet>(colony_entity).is_some(), "Colony should be on a planet");
    }

    #[test]
    fn test_planetary_traits_propagation() {
        let mut world = setup_world();

        // Mock: Ensure generation creates a planet with specific traits
        // This might require seeding or inspecting internal logic, but for black-box testing:
        // We verify that the Global Resource matches the Component on the Colony Planet.

        let mut schedule = Schedule::default();
        schedule.add_systems(generate_system);
        schedule.run(&mut world);

        let (colony_entity, _) = world.query::<(Entity, &ColonyLocation)>().single(&world);
        let component_traits = world.get::<PlanetaryTraitsComponent>(colony_entity).expect("Colony planet should have traits component");

        let global_traits = world.resource::<PlanetaryTraits>();

        // Assert they match
        // Note: PlanetaryTraits struct in Spec 080 wraps a Vec.
        // We need to ensure PartialEq is derived or check contents manually.
        assert_eq!(component_traits.traits, global_traits.0, "Global traits should match starting planet traits");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer2/generation.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer2::system::{OrbitalBody, Orbit};
use crate::layer1::quirks::{PlanetaryTraits, PlanetaryTrait};
use ratatui::style::Color;
use rand::prelude::*; // standard RNG

#[derive(Component)]
pub struct Star;

#[derive(Component)]
pub struct Planet;

#[derive(Component)]
pub struct Moon;

#[derive(Component)]
pub struct ColonyLocation;

#[derive(Component)]
pub struct PlanetaryTraitsComponent {
    pub traits: Vec<PlanetaryTrait>,
}

pub fn generate_system(mut commands: Commands, mut global_traits: ResMut<PlanetaryTraits>) {
    let mut rng = thread_rng();

    // 1. Create Star
    let star_entity = commands.spawn((
        Star,
        OrbitalBody {
            name: "Sun".to_string(),
            radius: 5.0,
            color: Color::Yellow,
            char: '☼',
        },
    )).id();

    // 2. Create Planets
    let num_planets = rng.gen_range(3..=8);
    let mut planets = Vec::new();

    for i in 0..num_planets {
        let distance = 20.0 + (i as f32 * 15.0) + rng.gen_range(-5.0..5.0);

        // Generate random traits for this planet
        let mut planet_traits = Vec::new();
        if rng.gen_bool(0.3) {
            planet_traits.push(PlanetaryTrait::HighGravity); // Simplified selection
        }

        let planet = commands.spawn((
            Planet,
            OrbitalBody {
                name: format!("Planet {}", i + 1),
                radius: rng.gen_range(1.0..3.0),
                color: Color::Blue, // Placeholder
                char: 'O',
            },
            Orbit {
                parent: star_entity,
                radius: distance,
                speed: 1.0 / distance.sqrt(), // Kepler-ish
                angle: rng.gen_range(0.0..6.28),
            },
            PlanetaryTraitsComponent {
                traits: planet_traits.clone(),
            },
        )).id();

        planets.push((planet, planet_traits));
    }

    // 3. Select Colony Location
    // Pick a random planet to be the colony
    if let Some((colony_entity, traits)) = planets.choose(&mut rng) {
        commands.entity(*colony_entity).insert(ColonyLocation);

        // 4. Propagate Traits
        global_traits.0 = traits.clone();
    }
}
```

## REFACTOR Phase: Quality & Design

- **Seeded RNG**: Replace `thread_rng()` with a seeded RNG (e.g., `ChaCha8Rng`) stored in a `WorldSeed` resource (from Spec 001/Project Scaffold if available, or create one). This ensures every run with the same seed generates the same system.
- **Planet Types**: Introduce `PlanetType` enum (Rocky, GasGiant, IceWorld).
  - Gas Giants shouldn't be `ColonyLocation` candidates (unless we support floating cities).
  - Visuals (`OrbitalBody.color`) should match type.
- **Orbit Logic**: Move orbital math (Kepler's laws) to a helper function.
- **Moons**: Add a loop to generate moons around planets (nested orbits).
- **Trait Logic**: Implement a weighted table for `PlanetaryTrait` generation based on `PlanetType` (e.g., High Gravity more likely on large Rocky worlds).

## Acceptance Criteria

- [ ] `generate_system` runs at startup.
- [ ] A Star, 3-8 Planets, and Colony Location are created.
- [ ] Global `PlanetaryTraits` resource is populated from the Colony Location.
- [ ] Planet entities have correct `OrbitalBody` and `Orbit` components.
- [ ] Tests pass.

## Technical Guidance

- Use `bevy::math` for constants like `PI`.
- Ensure `generate_system` is added to `Startup` schedule in `main.rs` or `layer2/mod.rs`.
- `PlanetaryTraits` resource must be initialized (inserted) *before* this system runs. Default initialization in `main.rs` is sufficient.
