# Specification 307: Xenological Pets

## 1. Overview
This feature introduces tamable alien fauna that act as "Pets" for colonists. A Pop can adopt a Pet, which follows them and provides a constant passive morale boost. However, Pets require specialized upkeep (e.g., `XenoRations`). When a Pet dies (from age, starvation, or violence), the owning Pop suffers a severe, long-lasting "Grief" penalty, dramatically increasing their stress and potentially causing a cascading failure if the owner is critical to the colony.

## 2. Dependencies
- `Fauna` component (`src/layer1/fauna.rs`)
- `Pop` needs and morale (`src/layer1/needs.rs`, `src/layer1/stress.rs`)
- `Grief` mechanics (`src/layer1/social/grief.rs` - conceptually extendable)
- `Item` system (`src/layer1/items.rs` for `XenoRations`)

## 3. RED Phase: Tests First

```rust
// src/layer1/social/pets.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::stress::StressTracker;
    use crate::layer1::needs::Needs;
    use crate::layer1::fauna::Fauna;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<PetDeathEvent>();
        app.add_systems(Update, (
            update_pet_morale_buff_system,
            process_pet_death_system,
        ));
        app
    }

    #[test]
    fn test_pet_provides_morale_buff_to_owner() {
        let mut app = setup_app();

        let owner = app.world_mut().spawn(StressTracker { accumulated_stress: 50.0 }).id();
        let pet = app.world_mut().spawn((
            Fauna { species_id: 1, age: 5 },
            Pet { owner_entity: owner },
        )).id();

        app.update();

        // Morale buff applied: Stress is reduced per tick or max stress is lowered
        let stress = app.world().get::<StressTracker>(owner).unwrap();
        assert!(stress.accumulated_stress < 50.0);
    }

    #[test]
    fn test_pet_death_causes_severe_grief() {
        let mut app = setup_app();

        let owner = app.world_mut().spawn(StressTracker { accumulated_stress: 10.0 }).id();
        let pet = app.world_mut().spawn((
            Fauna { species_id: 1, age: 100 },
            Pet { owner_entity: owner },
        )).id();

        app.world_mut().resource_mut::<Events<PetDeathEvent>>().send(
            PetDeathEvent { pet_entity: pet, owner_entity: owner }
        );

        app.update();

        // Owner should now have Grief component and massive stress spike
        assert!(app.world().get::<GrievingPet>(owner).is_some());
        let stress = app.world().get::<StressTracker>(owner).unwrap();
        assert!(stress.accumulated_stress >= 60.0); // +50 stress penalty
    }

    #[test]
    fn test_grief_decays_over_time() {
        let mut app = setup_app();

        let owner = app.world_mut().spawn((
            StressTracker { accumulated_stress: 60.0 },
            GrievingPet { remaining_ticks: 100 },
        )).id();

        app.update();

        // Grief duration decreased
        let grief = app.world().get::<GrievingPet>(owner).unwrap();
        assert_eq!(grief.remaining_ticks, 99);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/social/pets.rs

use bevy::prelude::*;
use crate::layer1::stress::StressTracker;
use crate::layer1::fauna::Fauna;

#[derive(Component, Clone, Debug)]
pub struct Pet {
    pub owner_entity: Entity,
}

#[derive(Component, Clone, Debug)]
pub struct GrievingPet {
    pub remaining_ticks: u32,
}

#[derive(Event, Clone, Debug)]
pub struct PetDeathEvent {
    pub pet_entity: Entity,
    pub owner_entity: Entity,
}

pub fn update_pet_morale_buff_system(
    mut query: Query<(&Pet, &Fauna)>,
    mut owner_query: Query<&mut StressTracker>,
) {
    for (pet, _fauna) in query.iter() {
        if let Ok(mut stress) = owner_query.get_mut(pet.owner_entity) {
            stress.accumulated_stress = (stress.accumulated_stress - 0.5).max(0.0);
        }
    }
}

pub fn process_pet_death_system(
    mut commands: Commands,
    mut events: EventReader<PetDeathEvent>,
    mut owner_query: Query<&mut StressTracker>,
) {
    for ev in events.read() {
        if let Ok(mut stress) = owner_query.get_mut(ev.owner_entity) {
            stress.accumulated_stress += 50.0;
            commands.entity(ev.owner_entity).insert(GrievingPet { remaining_ticks: 1000 });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding AI**: The pet should use pathfinding to stay within a specific distance of the owner, running its own localized Utility AI to find food or sleep when not actively following.
- **Upkeep Cost**: Integrate `Pet` with a new `ConsumeXenoRations` task in the Utility AI for the owner, forcing them to acquire food for the pet.
- **Narrative**: Fire a `AddChronicleEvent` on pet death describing the tragic end of a beloved xeno-fauna companion.
- **Modularity**: Consolidate `GrievingPet` into a more generic `Grief` tracker if one exists or is planned, simply tagged with a source enum (e.g., `GriefSource::Pet`).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for `pets.rs`.
- [ ] Pop with a pet has lower stress decay/accumulates less stress passively.
- [ ] Death of pet inserts `GrievingPet` and spikes stress.
- [ ] `GrievingPet` duration ticks down properly.

## 7. Technical Guidance
- **Lifecycle tracking**: Handle cases where the owner dies before the pet. The pet could become feral (losing the `Pet` component and reverting to wild fauna behavior) or die of broken heart.
- **Bevy ECS Query optimization**: When updating pet morale, iterating through pets and looking up owners is fine, but caching the `QueryState` or reversing the lookup (if a Pop has an `OwnedPet(Entity)` component) might be cleaner depending on how often a Pop references its pet.

## 8. Questions
*Builder: add questions here if spec is unclear.*
