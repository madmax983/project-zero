# 270: The Organ Market

## 1. Overview

The darkest possible logistics chain. Your people are just spare parts. A `Biomass Extractor` building allows harvesting `Organs` from dead (or living, but arrested) Pops. These organs can be sold on the Galactic Market for massive profits, or used to instantly cure critical injuries. However, the process generates massive `Paranoia` and `Horror` (Stress) across the colony, leading to moral collapse.

## 2. Dependencies

- `006` Building Placement
- `038` Medical Care
- `039` Trade System
- `050` Civil Unrest
- `072` Justice System (for arrested Pops)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::{StressTracker, Morale};
    use crate::layer1::inventory::Inventory;
    use crate::layer1::resources::{ResourceType, ResourceItem};
    use crate::layer1::pop::Pop;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, handle_biomass_extraction_system);
        app
    }

    #[test]
    fn test_harvesting_organ_generates_organs_and_causes_colony_stress() {
        let mut app = setup_app();

        let pop_id_1 = app.world_mut().spawn((
            Pop,
            StressTracker { accumulated_stress: 0.0, max: 100.0 },
        )).id();
        let pop_id_2 = app.world_mut().spawn((
            Pop,
            StressTracker { accumulated_stress: 0.0, max: 100.0 },
        )).id();

        let dead_pop_id = app.world_mut().spawn((
            Pop, // Dead flag would go here
            Dead,
        )).id();

        let extractor_id = app.world_mut().spawn((
            BiomassExtractor { processing: Some(dead_pop_id) },
            Inventory::new(10.0),
        )).id();

        app.world_mut().send_event(OrganHarvestEvent {
            target: dead_pop_id,
            extractor: extractor_id,
        });

        app.update(); // Tick 1

        let extractor_inventory = app.world().get::<Inventory>(extractor_id).unwrap();
        // The dead pop was processed and an organ was added
        assert_eq!(extractor_inventory.count(ResourceType::Organs), 1);

        // The living pops are horrified
        let stress_1 = app.world().get::<StressTracker>(pop_id_1).unwrap();
        let stress_2 = app.world().get::<StressTracker>(pop_id_2).unwrap();
        assert!(stress_1.accumulated_stress > 20.0);
        assert!(stress_2.accumulated_stress > 20.0);

        // The dead pop was consumed
        assert!(app.world().get_entity(dead_pop_id).is_none());
    }

    #[test]
    fn test_harvesting_living_prisoner_causes_extreme_stress() {
        let mut app = setup_app();

        let pop_id_1 = app.world_mut().spawn((
            Pop,
            StressTracker { accumulated_stress: 0.0, max: 100.0 },
        )).id();

        let prisoner_id = app.world_mut().spawn((
            Pop,
            Arrested, // Needs Justice System
        )).id();

        let extractor_id = app.world_mut().spawn((
            BiomassExtractor { processing: Some(prisoner_id) },
            Inventory::new(10.0),
        )).id();

        app.world_mut().send_event(OrganHarvestEvent {
            target: prisoner_id,
            extractor: extractor_id,
        });

        app.update();

        // Living pop should have even more stress
        let stress_1 = app.world().get::<StressTracker>(pop_id_1).unwrap();
        assert!(stress_1.accumulated_stress > 50.0); // Extreme horror penalty
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::inventory::Inventory;
use crate::layer1::resources::{ResourceType, ResourceItem};
use crate::layer1::pop::{Pop, Dead, Arrested};
use crate::layer1::needs::StressTracker;

#[derive(Event)]
pub struct OrganHarvestEvent {
    pub target: Entity,
    pub extractor: Entity,
}

#[derive(Component)]
pub struct BiomassExtractor {
    pub processing: Option<Entity>,
}

pub fn handle_biomass_extraction_system(
    mut events: EventReader<OrganHarvestEvent>,
    mut extractors: Query<&mut Inventory>,
    mut pops: Query<&mut StressTracker, With<Pop>>,
    targets: Query<(Has<Dead>, Has<Arrested>)>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok(mut inv) = extractors.get_mut(event.extractor) {
            if let Ok((is_dead, is_arrested)) = targets.get(event.target) {
                // Determine stress penalty
                let penalty = if is_dead { 25.0 } else if is_arrested { 60.0 } else { 0.0 };

                // Add organ
                inv.add(ResourceType::Organs, 1.0);

                // Panic the colony
                for mut stress in pops.iter_mut() {
                    stress.accumulated_stress += penalty;
                }

                // Consume the target
                commands.entity(event.target).despawn();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Rumor Web Integration**: Instead of instantly applying stress globally, the event should probably emit a `Rumor` (Spec 055) of type `Horror`, which spreads organically and stresses Pops who hear it.
- **Resource Item**: Ensure `ResourceType::Organs` is added to the enum. It should probably be extremely perishable (Spec 032) unless stored in a freezer.
- **Ethics**: If the colony has an "Amoral" or "Transhumanist" civic ideology (Spec 197), the stress penalty should be mitigated or removed entirely.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] Test coverage $\ge$ 85% for the new module.
- [ ] Processing a corpse yields an `Organ` item and adds stress to the colony.
- [ ] Processing a living (arrested) Pop yields an `Organ` item and adds massive stress to the colony.

## 7. Technical Guidance

- Implement `BiomassExtractor` in `src/layer1/buildings/extractor.rs`.
- Ensure `ResourceType::Organs` exists in `src/layer1/resources.rs`.
- Add an action for medical Pops to use `Organs` to instantly cure `Injury` or `Sickness` (skipping the normal healing time).

## 8. Questions

*Builder: add questions here if spec is unclear.*
*Architect:* Reviewed and verified. No further questions.
