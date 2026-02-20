# 095: System Generation

## Overview

Procedurally generate a star system for the Layer 2 simulation. This spec defines the logic to populate the `SystemMap` (introduced in `094`) with a Star, Planets, and Moons based on a deterministic seed.

This is the "Level Generation" for the space layer. It ensures that every new game starts with a unique but reproducible star system.

## Dependencies

- `094` System View Architecture (Defines `OrbitalBody`, `Orbit`)
- `001` Project Scaffold (ECS)

## RED Phase: Tests First

These tests define the requirements for the system generator. They must be written in `src/layer2/generation_tests.rs` (or similar) before implementation.

```rust
// src/layer2/generation_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::system::{OrbitalBody, Orbit, SystemMap};
    use crate::layer2::generation::{generate_system, WorldSeed, SystemGeneratorConfig};

    #[test]
    fn test_world_seed_resource() {
        let mut world = World::new();
        world.insert_resource(WorldSeed(12345));
        assert_eq!(world.resource::<WorldSeed>().0, 12345);
    }

    #[test]
    fn test_generate_system_populates_entities() {
        let mut world = World::new();
        world.insert_resource(WorldSeed(42));
        world.insert_resource(SystemMap::default());

        // Act
        generate_system(&mut world);

        // Assert
        let bodies = world.query::<&OrbitalBody>().iter(&world).len();
        assert!(bodies > 0, "System generation should spawn at least one body (the star)");
    }

    #[test]
    fn test_system_structure_star_and_planets() {
        let mut world = World::new();
        world.insert_resource(WorldSeed(100));
        world.insert_resource(SystemMap::default());

        generate_system(&mut world);

        // Find the Star (Body with no Orbit parent)
        let mut stars = 0;
        let mut star_entity = None;

        for (entity, _body, orbit) in world.query::<(Entity, &OrbitalBody, Option<&Orbit>)>().iter(&world) {
            if orbit.is_none() {
                stars += 1;
                star_entity = Some(entity);
            }
        }
        assert_eq!(stars, 1, "There should be exactly one star (root body)");
        let star = star_entity.unwrap();

        // Count planets (Parent is Star)
        let mut planets = 0;
        for (_entity, _body, orbit) in world.query::<(Entity, &OrbitalBody, &Orbit)>().iter(&world) {
            if orbit.parent == star {
                planets += 1;
            }
        }
        assert!(planets >= 1, "There should be at least one planet orbiting the star");
    }

    #[test]
    fn test_determinism() {
        // Run A
        let mut world_a = World::new();
        world_a.insert_resource(WorldSeed(999));
        world_a.insert_resource(SystemMap::default());
        generate_system(&mut world_a);
        let count_a = world_a.query::<&OrbitalBody>().iter(&world_a).len();

        // Run B (Same Seed)
        let mut world_b = World::new();
        world_b.insert_resource(WorldSeed(999));
        world_b.insert_resource(SystemMap::default());
        generate_system(&mut world_b);
        let count_b = world_b.query::<&OrbitalBody>().iter(&world_b).len();

        assert_eq!(count_a, count_b, "Same seed should produce same number of bodies");

        // Run C (Diff Seed)
        let mut world_c = World::new();
        world_c.insert_resource(WorldSeed(111)); // Different seed
        world_c.insert_resource(SystemMap::default());
        generate_system(&mut world_c);
        let count_c = world_c.query::<&OrbitalBody>().iter(&world_c).len();

        // Note: This *could* be equal by chance, but highly unlikely if logic varies count.
        // Better to check positions/properties if counts are equal.
        // For now, we assume simple count variance or property variance.
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `WorldSeed`

Add `WorldSeed` resource to `src/layer2/generation.rs` (or `shared/state.rs` if global scope desired, but keeping local for now is fine for Layer 2 focus).

```rust
use bevy_ecs::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng; // Deterministic RNG

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct WorldSeed(pub u64);

#[derive(Resource, Default, Debug)]
pub struct SystemGeneratorConfig {
    pub min_planets: u32,
    pub max_planets: u32,
    pub min_orbit_dist: f32,
    pub max_orbit_dist: f32,
}
```

### 2. Implement Generation Logic

```rust
use crate::layer2::system::{OrbitalBody, Orbit, SystemMap};

pub fn generate_system(world: &mut World) {
    let seed = world.get_resource::<WorldSeed>().map(|s| s.0).unwrap_or(0);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    // Spawn Star
    let star_entity = world.spawn(OrbitalBody {
        name: "Sun".to_string(),
        radius: 5.0,
        color: ratatui::style::Color::Yellow,
        char: '*',
    }).id();

    // Determine planet count
    let planet_count = rng.gen_range(3..=8);

    // Orbit placement (simple linear spacing for MVP)
    let mut current_dist = 20.0;

    for i in 0..planet_count {
        current_dist += rng.gen_range(10.0..25.0); // Spacing

        let planet_entity = world.spawn((
            OrbitalBody {
                name: format!("Planet {}", i + 1),
                radius: 1.0,
                color: ratatui::style::Color::Cyan, // Randomize later
                char: 'O',
            },
            Orbit {
                parent: star_entity,
                radius: current_dist,
                speed: rng.gen_range(0.01..0.05), // Slower further out? (Kepler later)
                angle: rng.gen_range(0.0..std::f32::consts::TAU),
            }
        )).id();

        // Chance for Moon
        if rng.gen_bool(0.3) {
             world.spawn((
                OrbitalBody {
                    name: format!("Moon {}-A", i + 1),
                    radius: 0.2,
                    color: ratatui::style::Color::Gray,
                    char: '.',
                },
                Orbit {
                    parent: planet_entity,
                    radius: 3.0,
                    speed: 0.1,
                    angle: rng.gen_range(0.0..std::f32::consts::TAU),
                }
            ));
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Kepler's Laws**: Update `Orbit.speed` calculation to decrease with `radius` (sqrt(1/r)).
- **Types**: Introduce `StarType` (Red Dwarf, Yellow Main, Blue Giant) and `PlanetType` (Rocky, Gas Giant, Ice) enums to drive color/char/radius.
- **Names**: Use a procedural name generator (syllables) instead of "Planet 1".
- **Safety**: Ensure orbits don't overlap (collision checks).

## Acceptance Criteria

- [ ] `WorldSeed` resource controls the RNG.
- [ ] `generate_system` spawns 1 Star and multiple Planets.
- [ ] Planets have correct `Orbit` components pointing to the Star.
- [ ] Some planets have Moons.
- [ ] Running with the same seed produces the exact same system layout.
