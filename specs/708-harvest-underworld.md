# 708: The Harvest Underworld

## Overview

When a colony faces severe, prolonged food or organic material shortages, desperate measures are taken. Pops with low morale and high survival traits may secretly start a "Harvest Ring," processing their own dead (or "disappeared" pops) into highly efficient, generic "Biomass". This bypasses normal food chains but carries a heavy stress penalty for those consuming it without knowing its source—and a catastrophic penalty if the truth is revealed.

## Dependencies

- None explicitly, but relies on existing Layer 1 needs (Hunger, Morale, Stress) and the `ColonyResources` system.

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::social::morale::Morale;
    use crate::layer1::needs::{Hunger, Stress};
    use crate::layer1::traits::PopTraits;
    use crate::shared::resources::ColonyResources;

    #[test]
    fn test_harvest_ring_formation() {
        let mut app = App::new();
        app.insert_resource(ColonyResources::default());
        app.add_systems(Update, evaluate_harvest_rings);

        // Arrange: severe food shortage
        let mut resources = app.world_mut().resource_mut::<ColonyResources>();
        resources.food = 0.0;

        let entity = app.world_mut().spawn((
            Morale { value: 10.0 }, // low morale
            PopTraits { survival_instinct: 0.9, ..default() }, // high survival
            Hunger { value: 90.0 }, // starving
        )).id();

        // Act
        app.update();

        // Assert: Pop should start a HarvestRing
        assert!(app.world().get::<HarvestRing>(entity).is_some());
    }

    #[test]
    fn test_biomass_consumption_causes_stress() {
        let mut app = App::new();
        app.add_systems(Update, consume_biomass_system);

        let entity = app.world_mut().spawn((
            Hunger { value: 80.0 },
            Stress { value: 10.0 },
        )).id();

        // Arrange: Eat biomass
        app.world_mut().entity_mut(entity).insert(EatingBiomass);

        // Act
        app.update();

        // Assert: Hunger reduced, but stress increased
        let hunger = app.world().get::<Hunger>(entity).unwrap();
        let stress = app.world().get::<Stress>(entity).unwrap();

        assert!(hunger.value < 80.0);
        assert!(stress.value > 10.0);
    }

    #[test]
    fn test_truth_reveal_causes_panic() {
        let mut app = App::new();
        app.add_event::<TruthRevealedEvent>();
        app.add_systems(Update, truth_reveal_system);

        let entity = app.world_mut().spawn((
            Morale { value: 50.0 },
            ConsumedBiomassTracker { amount: 10.0 },
        )).id();

        // Act
        app.world_mut().send_event(TruthRevealedEvent);
        app.update();

        // Assert: Morale shattered
        let morale = app.world().get::<Morale>(entity).unwrap();
        assert!(morale.value < 10.0); // catastrophic drop
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::social::morale::Morale;
use crate::layer1::needs::{Hunger, Stress};
use crate::layer1::traits::PopTraits;
use crate::shared::resources::ColonyResources;

#[derive(Component)]
pub struct HarvestRing;

#[derive(Component)]
pub struct EatingBiomass;

#[derive(Component)]
pub struct ConsumedBiomassTracker {
    pub amount: f32,
}

#[derive(Event)]
pub struct TruthRevealedEvent;

pub fn evaluate_harvest_rings(
    mut commands: Commands,
    resources: Res<ColonyResources>,
    query: Query<(Entity, &Morale, &PopTraits, &Hunger), Without<HarvestRing>>,
) {
    if resources.food <= 0.0 {
        for (entity, morale, traits, hunger) in query.iter() {
            if morale.value < 20.0 && traits.survival_instinct > 0.8 && hunger.value > 80.0 {
                commands.entity(entity).insert(HarvestRing);
            }
        }
    }
}

pub fn consume_biomass_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Hunger, &mut Stress, Option<&mut ConsumedBiomassTracker>), With<EatingBiomass>>,
) {
    for (entity, mut hunger, mut stress, mut tracker_opt) in query.iter_mut() {
        hunger.value = (hunger.value - 30.0).max(0.0);
        stress.value = (stress.value + 5.0).min(100.0);
        commands.entity(entity).remove::<EatingBiomass>();

        if let Some(mut tracker) = tracker_opt {
            tracker.amount += 1.0;
        } else {
            commands.entity(entity).insert(ConsumedBiomassTracker { amount: 1.0 });
        }
    }
}

pub fn truth_reveal_system(
    mut events: EventReader<TruthRevealedEvent>,
    mut query: Query<(&mut Morale, &ConsumedBiomassTracker)>,
) {
    for _ in events.read() {
        for (mut morale, tracker) in query.iter_mut() {
            // Drop morale based on how much was consumed
            morale.value = (morale.value - (tracker.amount * 10.0)).max(0.0);
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: `evaluate_harvest_rings` currently iterates over all pops every frame. Add a timer or only run when food levels actually change.
- **Code Smell**: Hardcoded thresholds (`20.0` morale, `0.8` survival, `30.0` hunger reduction). These should be configurable via a `HarvestUnderworldConfig` resource.
- **Feature Depth**: Introduce a `MissingPersonEvent` that the harvest ring triggers, tying into the broader simulation's event log and inspector mechanics.

## Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops form harvest rings only when starving, with low morale and high survival traits.
- [ ] Consuming biomass temporarily relieves hunger but slightly increases stress.
- [ ] Triggering the truth reveal event massively damages morale of those who consumed biomass.

## Technical Guidance
- Create this in `src/layer1/social/harvest_underworld.rs`.
- Ensure systems are registered in the main simulation update loop.
- You'll likely need to bridge `MissingPersonEvent` and `TruthRevealedEvent` to `AddChronicleEvent` for lore purposes.
- Consider adding a `Biomass` resource to `ColonyResources`, or keep it tracked implicitly via the rings.

## Questions
*Builder: add questions here if spec is unclear.*
