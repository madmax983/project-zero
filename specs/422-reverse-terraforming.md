# 422: Reverse Terraforming

## 1. Overview
**Layer:** 2 -> 1
**Fantasy:** Poisoning the well to get to the gold.
**Mechanic:** You deploy "Atmo-Strippers" from orbit to intentionally degrade a planet's habitability. This removes annoying flora/fauna and exposes deep crustal resources, but turns the world into a toxic, irradiated wasteland that requires expensive sealed habitats to survive.
**Emergence:** You strip the atmosphere to mine rare "Core-Gems" faster. The ensuing toxic storms destroy your fragile bio-domes, killing your expert miners. You have the gems, but no one left to spend them.
**Tension:** Safe, slow surface extraction vs. Fast, destructive deep extraction.

## 2. Dependencies
- Atmospheric/Environment system
- Orbital deployment/action system
- Resource deposit system
- Terrain grid mutation

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_atmo_stripper_degrades_habitability_and_exposes_resources() {
        // Arrange
        let mut world = World::new();
        let mut app = App::new();
        // Setup initial planetary environment
        let planet = world.spawn((
            Habitability { value: 100.0 },
            CrustalResources { exposed: false },
        )).id();

        // Spawn orbital stripper event/component
        world.insert_resource(AtmoStripperDeployed { active: true });

        // Act
        app.add_systems(Update, reverse_terraforming_system);
        app.update();

        // Assert
        let env = world.get::<Habitability>(planet).unwrap().value;
        let resources = world.get::<CrustalResources>(planet).unwrap().exposed;
        assert!(env < 100.0, "Habitability should decrease when Atmo-Stripper is active");
        assert!(resources, "Deep crustal resources should become exposed");
    }

    #[test]
    fn test_toxic_environment_damages_unsealed_pops() {
        // Arrange
        let mut world = World::new();
        let mut app = App::new();
        // Setup toxic environment (low habitability)
        world.insert_resource(EnvironmentToxicity { level: 80.0 }); // High toxicity

        let unsealed_pop = world.spawn((
            Pop { health: 100.0 },
            SealedSuit { is_sealed: false },
        )).id();

        let sealed_pop = world.spawn((
            Pop { health: 100.0 },
            SealedSuit { is_sealed: true },
        )).id();

        // Act
        app.add_systems(Update, toxicity_damage_system);
        app.update();

        // Assert
        let u_health = world.get::<Pop>(unsealed_pop).unwrap().health;
        let s_health = world.get::<Pop>(sealed_pop).unwrap().health;
        assert!(u_health < 100.0, "Unsealed Pop should take damage in toxic environment");
        assert_eq!(s_health, 100.0, "Sealed Pop should not take damage in toxic environment");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Habitability {
    pub value: f32,
}

#[derive(Component)]
pub struct CrustalResources {
    pub exposed: bool,
}

#[derive(Resource)]
pub struct AtmoStripperDeployed {
    pub active: bool,
}

#[derive(Resource)]
pub struct EnvironmentToxicity {
    pub level: f32,
}

#[derive(Component)]
pub struct Pop {
    pub health: f32,
}

#[derive(Component)]
pub struct SealedSuit {
    pub is_sealed: bool,
}

pub fn reverse_terraforming_system(
    stripper: Option<Res<AtmoStripperDeployed>>,
    mut planet_query: Query<(&mut Habitability, &mut CrustalResources)>,
) {
    if let Some(s) = stripper {
        if s.active {
            for (mut env, mut resources) in planet_query.iter_mut() {
                env.value -= 10.0;
                resources.exposed = true;
            }
        }
    }
}

pub fn toxicity_damage_system(
    toxicity: Option<Res<EnvironmentToxicity>>,
    mut pop_query: Query<(&mut Pop, &SealedSuit)>,
) {
    if let Some(tox) = toxicity {
        if tox.level > 50.0 {
            for (mut pop, suit) in pop_query.iter_mut() {
                if !suit.is_sealed {
                    pop.health -= 5.0; // Toxic damage
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Separate the atmospheric degradation logic into a tick-based system rather than an instantaneous event.
- Connect Habitability loss to an increase in `EnvironmentToxicity` resource automatically.
- Integrate the resource exposure with the map/grid generation system to actually reveal mining tiles.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Deploying Atmo-Stripper correctly mutates planet habitability and exposes deep resources.

## 7. Technical Guidance
- The Habitability and Toxicity should ideally be grid-based (`AtmosphereGrid`) rather than global resources for granular interactions.

## 8. Questions
*Builder: add questions here if spec is unclear.*
