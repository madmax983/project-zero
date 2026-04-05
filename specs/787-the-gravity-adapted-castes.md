# 787: The Gravity-Adapted Castes

## 1. Overview
**Layer:** 1 -> Cross-layer
**Fantasy:** Your people evolve to fit the diverse worlds they colonize.
**Mechanic:** Pops that live on high-gravity or low-gravity worlds for multiple generations develop physical adaptations. High-G pops are slow but incredibly strong and resilient, making perfect heavy miners or shock troops. Low-G pops are agile and fragile, ideal for zero-G orbital construction.
**Emergence:** You try to transfer High-G miners to an orbital station to speed up construction, but their heavy frames cause constant structural damage to the delicate station, forcing you to maintain separate, specialized logistical chains for different genetic castes.
**Tension:** Optimizing labor efficiency through specialization vs. the growing logistical nightmare and potential cultural schisms of maintaining drastically different biological castes.

## 2. Dependencies
- Layer 1 Population System (`Pop`, `Stats`)
- Layer 1 Generation Tracking (`GenerationalTraits`)
- Layer 2 Planetary Gravity/Environment (`PlanetEnvironment`)
- Layer 1/2 Logistics/Transfer System (`PopTransfer`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Stats;
    use crate::layer1::structure::Structure;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            apply_gravity_adaptation_traits_system,
            process_gravity_adaptation_damage_system
        ));
        app
    }

    #[test]
    fn test_high_gravity_pops_gain_strength_lose_speed() {
        let mut app = setup_app();

        let entity = app.world_mut().spawn((
            Stats { strength: 10.0, speed: 10.0, ..Default::default() },
            GravityAdaptation { gravity_type: GravityType::High },
        )).id();

        app.update();

        let stats = app.world().get::<Stats>(entity).unwrap();
        assert!(stats.strength > 10.0, "High-G pops should have increased strength");
        assert!(stats.speed < 10.0, "High-G pops should have decreased speed");
    }

    #[test]
    fn test_high_gravity_pops_damage_delicate_structures() {
        let mut app = setup_app();

        // Spawn a delicate orbital station structure
        let structure_entity = app.world_mut().spawn((
            Structure { current_hp: 100.0, max_hp: 100.0 },
            DelicateStructure,
        )).id();

        // Spawn a High-G pop interacting with the delicate structure (represented by BeingOnStation)
        app.world_mut().spawn((
            GravityAdaptation { gravity_type: GravityType::High },
            OccupyingStructure { structure_entity },
        ));

        app.update();

        let structure = app.world().get::<Structure>(structure_entity).unwrap();
        assert!(structure.current_hp < 100.0, "High-G pops should damage delicate orbital structures");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::pop::Stats;
use crate::layer1::structure::Structure;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GravityType {
    High,
    Low,
}

#[derive(Component)]
pub struct GravityAdaptation {
    pub gravity_type: GravityType,
}

#[derive(Component)]
pub struct DelicateStructure;

#[derive(Component)]
pub struct OccupyingStructure {
    pub structure_entity: Entity,
}

pub fn apply_gravity_adaptation_traits_system(
    mut query: Query<(&GravityAdaptation, &mut Stats), Changed<GravityAdaptation>>,
) {
    for (adaptation, mut stats) in query.iter_mut() {
        match adaptation.gravity_type {
            GravityType::High => {
                stats.strength *= 1.5;
                stats.speed *= 0.5;
            }
            GravityType::Low => {
                stats.strength *= 0.5;
                stats.speed *= 1.5;
            }
        }
    }
}

pub fn process_gravity_adaptation_damage_system(
    pop_query: Query<(&GravityAdaptation, &OccupyingStructure)>,
    mut structure_query: Query<&mut Structure, With<DelicateStructure>>,
) {
    for (adaptation, occupying) in pop_query.iter() {
        if adaptation.gravity_type == GravityType::High {
            if let Ok(mut structure) = structure_query.get_mut(occupying.structure_entity) {
                structure.current_hp -= 1.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Generational Evolution:** The trait shouldn't appear instantly. We need a system that tracks `GenerationsInGravity` and assigns the `GravityAdaptation` component once a threshold is reached.
- **Low-G Complications:** High-G pops damage structures, but Low-G pops should suffer health damage if placed on High-G worlds without exosuits.
- **Cultural Schisms:** Add traits or modifiers so different gravity castes form separate social classes or factions over time.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `gravity_adapted_castes`.
- [ ] High-G pops gain strength and lose speed; Low-G pops vice versa.
- [ ] High-G pops passively deal minor damage to delicate structures they occupy.

## 7. Technical Guidance
- Integrate with the existing `Stats` or `Traits` system in `src/layer1/pop`.
- The `DelicateStructure` marker should be added to Orbital Stations and fragile outposts.
- Provide clear UI warnings when transferring adapted pops to unsuitable environments.

## 8. Questions
*Builder: add questions here if spec is unclear.*
