# 769: The Agony Extract

## 1. Overview
A highly lucrative resource that requires pops to be under extreme stress to harvest, forcing a choice between morality and immense wealth. Certain rare flora or fauna only produce "Agony Extract" (which sells for astronomical prices on Layer 3) when harvested by Pops with critically low Morale or high Stress. Happy workers yield nothing.

## 2. Dependencies
- Layer 1 Morale System (`StressTracker` / `Morale`)
- Layer 1 Economy System (`ColonyResources`, specific inventory items)
- Layer 3 Trading / Economy System (if applicable)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::social::morale::{Morale, StressTracker};
    use crate::layer1::economy::inventory::{Inventory, InventoryItem};

    #[test]
    fn test_harvest_agony_extract_requires_high_stress() {
        let mut app = App::new();
        app.add_systems(Update, process_agony_extract_harvest_system);

        let happy_worker = app.world_mut().spawn((
            Morale { current: 80.0, ..default() },
            StressTracker { accumulation: 10.0 },
            Inventory::default(),
        )).id();

        let stressed_worker = app.world_mut().spawn((
            Morale { current: 10.0, ..default() },
            StressTracker { accumulation: 90.0 },
            Inventory::default(),
        )).id();

        // Assume an event triggering a harvest action
        app.world_mut().send_event(HarvestAgonyExtractEvent { worker: happy_worker });
        app.world_mut().send_event(HarvestAgonyExtractEvent { worker: stressed_worker });

        app.update();

        // Happy worker gets nothing
        let happy_inv = app.world().get::<Inventory>(happy_worker).unwrap();
        assert_eq!(happy_inv.count(InventoryItem::AgonyExtract), 0);

        // Stressed worker successfully harvests
        let stressed_inv = app.world().get::<Inventory>(stressed_worker).unwrap();
        assert_eq!(stressed_inv.count(InventoryItem::AgonyExtract), 1);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::social::morale::{Morale, StressTracker};
use crate::layer1::economy::inventory::{Inventory, InventoryItem};

#[derive(Event)]
pub struct HarvestAgonyExtractEvent {
    pub worker: Entity,
}

pub fn process_agony_extract_harvest_system(
    mut events: EventReader<HarvestAgonyExtractEvent>,
    mut workers: Query<(&Morale, &StressTracker, &mut Inventory)>,
) {
    for event in events.read() {
        if let Ok((morale, stress, mut inventory)) = workers.get_mut(event.worker) {
            // Requires critically low morale or high stress
            if morale.current < 20.0 || stress.accumulation > 80.0 {
                inventory.add(InventoryItem::AgonyExtract, 1);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactor:** Create a configurable resource `AgonyExtractConfig` to define the thresholds for Morale and Stress, instead of hardcoding `20.0` and `80.0`.
- **Smell:** Empty harvest events being sent for happy pops could lead to unnecessary event processing.
- **Design:** Ensure that working in the Agony Extract harvesting facility actively causes an increase in Stress or a drop in Morale to create a self-sustaining cycle of misery.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Agony Extract is successfully harvested only by pops with low Morale or high Stress.
- [ ] Happy pops attempting to harvest yield exactly 0 Agony Extract.

## 7. Technical Guidance
- Integrate with existing pop interaction logic (e.g., job completion systems).
- The "Agony Extract" item should be heavily valued in trading logic (Layer 3 or orbital merchants).
- Consider emitting an event when Agony Extract is successfully harvested for use by integration systems to trigger negative diplomatic consequences or political events.

## 8. Questions
*Builder: add questions here if spec is unclear.*
