# 906: The Resource Hoard

## 1. Overview
Generational wealth and greed become physical obstacles. Extremely old or wealthy Pops start physically hoarding high-value resources in their homes instead of depositing them in colony storage. If they die, the hoard is inherited by their descendants. This creates localized wealth disparity and artificial shortages in the main supply chain.

## 2. Dependencies
- `layer1::pops::PopAge` or `PopWealth`
- `layer1::pops::PopDiedEvent`
- `layer1::resources::ResourceInventory` (or equivalent per-entity storage)
- `layer1::lineage::PopLineage` (for inheritance)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pops::{PopAge, PopDiedEvent};
    use crate::layer1::lineage::PopLineage;

    #[test]
    fn test_elderly_pop_starts_hoarding() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_hoarding_desire_system);

        // Arrange
        let pop = app.world_mut().spawn((
            PopAge { age_years: 85 }, // Very old
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<ResourceHoarder>(pop).is_some());
    }

    #[test]
    fn test_hoard_inheritance_on_death() {
        let mut app = App::new();
        app.add_event::<PopDiedEvent>();
        app.add_systems(Update, process_hoard_inheritance_system);

        // Arrange
        let descendant = app.world_mut().spawn_empty().id();
        let hoarder = app.world_mut().spawn((
            PopLineage { descendant_entity: Some(descendant) },
            ResourceHoard { amount: 50.0 },
        )).id();

        app.world_mut().send_event(PopDiedEvent { entity: hoarder });

        // Act
        app.update();

        // Assert
        let descendant_hoard = app.world().get::<ResourceHoard>(descendant).unwrap();
        assert_eq!(descendant_hoard.amount, 50.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pops::{PopAge, PopDiedEvent};
use crate::layer1::lineage::PopLineage;

#[derive(Component)]
pub struct ResourceHoarder;

#[derive(Component)]
pub struct ResourceHoard {
    pub amount: f32,
}

pub fn evaluate_hoarding_desire_system(
    mut commands: Commands,
    query: Query<(Entity, &PopAge), Without<ResourceHoarder>>,
) {
    for (entity, age) in query.iter() {
        if age.age_years >= 80 {
            commands.entity(entity).insert(ResourceHoarder);
            commands.entity(entity).insert(ResourceHoard { amount: 0.0 });
        }
    }
}

pub fn process_hoard_inheritance_system(
    mut commands: Commands,
    mut death_events: EventReader<PopDiedEvent>,
    query: Query<(&PopLineage, &ResourceHoard)>,
) {
    for event in death_events.read() {
        if let Ok((lineage, hoard)) = query.get(event.entity) {
            if let Some(descendant) = lineage.descendant_entity {
                commands.entity(descendant).insert(ResourceHoard {
                    amount: hoard.amount,
                });
            } else {
                // If no descendant, potentially scatter to colony storage (not implemented in minimal green)
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Hoarding Trigger**: The hardcoded age check (80) should be replaced with a probability curve, or a configuration component, taking into account Pop traits (like "Greedy" or "Paranoid").
- **Resource Types**: Currently using an abstract `amount: f32`. This should be refactored to use the actual `ResourceType` and `ResourceInventory` systems to intercept specific high-value goods (like Rare Metals or Medicine).

## 6. Acceptance Criteria
- [ ] All tests pass.
- [ ] Test coverage >= 85%.
- [ ] Pops over a certain age threshold gain the `ResourceHoarder` and `ResourceHoard` components.
- [ ] When a Pop dies, their `ResourceHoard` is transferred to their `descendant_entity` via `PopLineage`.

## 7. Technical Guidance
- Be careful with the inheritance system if the descendant is dead or despawned. Ensure you handle missing entities gracefully.
- The actual interception of goods (taking from colony storage into the hoard) is an integration point that will likely require hooking into the `ColonyResources` or logistics/pathfinding gathering jobs.

## 8. Questions
