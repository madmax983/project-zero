# 637: Echoes of the Predecessors

## Overview

Settling on worlds containing Layer 3 ruins (remnants of a fallen galactic empire) provides steady research bonuses. However, actions on Layer 1—such as deep drilling, massive energy spikes, or even specific population densities—can inadvertently awaken dormant planetary systems left by the Predecessors. This might trigger an ancient weather-control array plunging the planet into permanent spring, or activate an automated quarantine protocol surrounding the world with an impenetrable orbital shield that blocks all Layer 2 trade and communication.

## Dependencies

- `042` Energy System
- `095` System Generation
- `280` Archaeological Layers

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::energy::EnergyGrid;
    use crate::layer1::resources::ColonyResources;
    use crate::layer2::planet::PlanetaryTraits;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            predecessor_ruins_passive_bonus_system,
            predecessor_ruins_awakening_system,
        ));
        app
    }

    #[test]
    fn test_predecessor_ruins_provide_passive_research_bonus() {
        let mut app = setup_app();

        app.insert_resource(ColonyResources { knowledge: 0.0, ..Default::default() });
        app.world_mut().spawn(PredecessorRuin { awakened: false, research_bonus_rate: 5.0 });

        app.insert_resource(Time::default());
        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert!(resources.knowledge > 0.0, "Passive research bonus should be applied");
    }

    #[test]
    fn test_massive_energy_spike_awakens_dormant_systems() {
        let mut app = setup_app();

        let ruin_entity = app.world_mut().spawn(PredecessorRuin {
            awakened: false,
            research_bonus_rate: 5.0
        }).id();

        // Create an energy grid with a massive spike
        app.world_mut().spawn(EnergyGrid { total_generation: 10000.0, total_consumption: 50.0 });

        app.update();

        // Assert ruin is awakened
        let ruin = app.world().get::<PredecessorRuin>(ruin_entity).unwrap();
        assert!(ruin.awakened, "Massive energy spike should awaken the ruin");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::energy::EnergyGrid;
use crate::layer1::resources::ColonyResources;

#[derive(Component)]
pub struct PredecessorRuin {
    pub awakened: bool,
    pub research_bonus_rate: f32,
}

pub fn predecessor_ruins_passive_bonus_system(
    mut resources: ResMut<ColonyResources>,
    query: Query<&PredecessorRuin>,
    time: Res<Time>,
) {
    for ruin in query.iter() {
        if !ruin.awakened {
            resources.knowledge += ruin.research_bonus_rate * time.delta_seconds();
        }
    }
}

pub fn predecessor_ruins_awakening_system(
    mut commands: Commands,
    mut ruins_query: Query<(Entity, &mut PredecessorRuin)>,
    energy_query: Query<&EnergyGrid>,
) {
    let mut total_grid_energy = 0.0;
    for grid in energy_query.iter() {
        total_grid_energy += grid.total_generation;
    }

    // Arbitrary threshold for awakening
    if total_grid_energy > 5000.0 {
        for (entity, mut ruin) in ruins_query.iter_mut() {
            if !ruin.awakened {
                ruin.awakened = true;

                // For MVP, just spawn a marker component representing an orbital shield
                commands.entity(entity).insert(PredecessorOrbitalShield);
            }
        }
    }
}

#[derive(Component)]
pub struct PredecessorOrbitalShield;
```

## REFACTOR Phase: Quality & Design

- **Event-Driven Awakening**: Move the awakening check into an event-driven model. Have systems that track energy spikes, deep drilling depth, or population cap emit a `WorldTriggerEvent`. The ruins listen for these events.
- **Dynamic Consequences**: Instead of always spawning `PredecessorOrbitalShield`, randomly select from a pool of consequences (e.g., Permanent Spring, Shield, Drone Swarm) when the `PredecessorRuin` awakens.
- **Lore Integration**: Hook into the `Chronicle` system to document the exact trigger and consequence of the awakening.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] `PredecessorRuin` components grant passive knowledge bonuses while unawakened.
- [ ] Extreme colony conditions (like energy spikes) cause the ruin to awaken and trigger a global effect.

## Technical Guidance

- When an orbital shield or weather array activates, ensure the effect permeates to Layer 2 (e.g., blocking `Fleet` pathfinding into the system node) or modifies the `AtmosphericSimulation` components globally.

## Questions

*Builder: add questions here if spec is unclear.*
