# 1148: The Biomass Dividend

## 1. Overview
The "Biomass Dividend" feature introduces a dark survival mechanic where the corpses of dead Souls can be processed in Recyclers to produce raw Biomass (food/fertilizer) during shortages. However, consuming this recycled food attaches a hidden "Grief" penalty to the resulting rations. When Souls eat this tainted food, it triggers long-term psychological degradation, massive stress spikes, and potential colony-wide violent uprisings, creating a tension between immediate physical survival and deferred psychological collapse.

## 2. Dependencies
- `bevy_ecs` setup.
- Layer 1 colony components for `Soul` (population), `Health`, and `Stress`/`Mood`.
- Systems managing `DeathEvent` or corpse entities.
- Resource management systems for `Biomass` or `Food` stockpiles, including item metadata to track "taint."

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_corpse_recycling_produces_tainted_biomass() {
        // Arrange
        let mut world = World::new();

        let corpse = world.spawn((
            Corpse,
            Recyclable { biomass_yield: 10.0 },
        )).id();

        let recycler = world.spawn((
            Building,
            Recycler { active: true },
        )).id();

        world.init_resource::<Events<RecycleEvent>>();
        let mut events = world.resource_mut::<Events<RecycleEvent>>();
        events.send(RecycleEvent { target: corpse, processor: recycler });

        // Act
        world.run_system_once(process_corpse_recycling_system);

        // Assert
        let query = world.query::<&BiomassRation>().iter(&world).collect::<Vec<_>>();
        assert_eq!(query.len(), 1, "Should produce one batch of Biomass");
        assert!(query[0].has_grief_taint, "Biomass produced from a Corpse MUST have the grief taint");
    }

    #[test]
    fn test_consuming_tainted_biomass_increases_stress() {
        // Arrange
        let mut world = World::new();

        let soul = world.spawn((
            Soul,
            Stress(0.0),
            DigestionQueue(vec![BiomassRation { has_grief_taint: true, nutritional_value: 10.0 }]),
        )).id();

        // Act
        world.run_system_once(consume_rations_system);

        // Assert
        let stress = world.get::<Stress>(soul).unwrap();
        assert!(stress.0 > 0.0, "Consuming tainted biomass must increase the Soul's Stress");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Corpse;

#[derive(Component)]
pub struct Recyclable {
    pub biomass_yield: f32,
}

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct Recycler {
    pub active: bool,
}

#[derive(Event)]
pub struct RecycleEvent {
    pub target: Entity,
    pub processor: Entity,
}

#[derive(Component, Clone)]
pub struct BiomassRation {
    pub has_grief_taint: bool,
    pub nutritional_value: f32,
}

#[derive(Component)]
pub struct Soul;

#[derive(Component)]
pub struct Stress(pub f32);

#[derive(Component)]
pub struct DigestionQueue(pub Vec<BiomassRation>);

pub fn process_corpse_recycling_system(
    mut commands: Commands,
    mut events: EventReader<RecycleEvent>,
    corpse_query: Query<&Recyclable, With<Corpse>>,
) {
    for event in events.read() {
        if let Ok(recyclable) = corpse_query.get(event.target) {
            // Despawn the corpse
            commands.entity(event.target).despawn();

            // Spawn tainted biomass
            commands.spawn(BiomassRation {
                has_grief_taint: true,
                nutritional_value: recyclable.biomass_yield,
            });
        }
    }
}

pub fn consume_rations_system(
    mut commands: Commands,
    mut souls_query: Query<(Entity, &mut Stress, &mut DigestionQueue), With<Soul>>,
) {
    for (entity, mut stress, mut queue) in souls_query.iter_mut() {
        // Consume one ration if available
        if !queue.0.is_empty() {
            let ration = queue.0.remove(0);

            // Apply psychological penalty if tainted
            if ration.has_grief_taint {
                stress.0 += 50.0; // High stress spike
                // In a fuller implementation, this might add a permanent trauma trait
            }

            // Note: Nutritional benefits would be applied here
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Hardcoded `50.0` stress spike. This should be moved to a `PsychologicalConfig` resource or derived from the specific relation the Soul had to the deceased.
- **Resource Stacking**: `BiomassRation` is currently an entity component. In an inventory system, we need a way to stack tainted and untainted food without losing the taint flag. Perhaps a `TaintedBiomass` specific resource pool is required, or inventory slots must track metadata.
- **Event Handling**: Despawning inside the event iteration might conflict if multiple systems try to recycle the same entity. Consider a `Despawning` marker component or checking entity validity.
- **Knowledge Spread**: A Soul might not know the food is tainted immediately. A refactor could delay the stress hit until an `EpiphanyEvent` or a rumor spreads through social interactions.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Recycling a `Corpse` successfully produces a `BiomassRation` with `has_grief_taint = true`.
- [ ] A `Soul` consuming a tainted ration experiences an immediate increase in their `Stress` component.

## 7. Technical Guidance
- **Inventory Integration**: Pay close attention to how food items stack. Tainted food MUST NOT be diluted or cleansed by mixing it with normal food.
- **Lexicon**: Ensure you use `Soul` for population entities and `Substrate` for player directives/UI actions in any lore or logging text.
- **Emergence**: Hook this into the existing `Emotional Contagion` or `Utility AI` systems. A Soul discovering they ate their coworker should probably trigger a violent `ActionType::Riot` or `ActionType::Breakdown`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
