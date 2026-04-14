# 1034: Gene-Banks

## 1. Overview
Gene-Banks act as a "Noah's Ark" for DNA, allowing players to store samples of flora, fauna, and Pops in cryogenic storage. If a species goes extinct due to plague or over-harvesting, it can be cloned back into existence. However, the cloning process introduces "Genetic Drift" (slight random mutations), leading to emergent consequences—like resurrecting a docile herd animal only to find the new cloned generation is aggressively carnivorous.

## 2. Dependencies
- Layer 1 `Flora`/`Fauna` entity spawning and trait systems.
- Layer 1 `Buildings` (Cryo-Bank structure).
- Layer 1 `Extinction` event tracking.
- Layer 1 `Genetics` or `Mutation` mechanics.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::buildings::CryoBank;
    use crate::layer1::fauna::{Fauna, SpeciesId, Traits};
    use crate::layer1::genetics::{CloneEvent, ExtinctionEvent};

    #[test]
    fn test_cloning_extinct_species_applies_genetic_drift() {
        let mut app = App::new();
        app.add_event::<CloneEvent>();
        app.add_systems(Update, process_cloning_system);

        let bank = app.world_mut().spawn(CryoBank {
            stored_samples: vec![SpeciesId { id: "WoolyGrox".to_string() }],
        }).id();

        // Simulate cloning the extinct Grox
        app.world_mut().resource_mut::<Events<CloneEvent>>().send(CloneEvent {
            bank_entity: bank,
            species_id: "WoolyGrox".to_string(),
        });

        app.update();

        // Verify the newly spawned fauna has mutated traits
        let mut found_clone = false;
        let mut query = app.world_mut().query::<(&SpeciesId, &Traits)>();
        for (species, traits) in query.iter(app.world()) {
            if species.id == "WoolyGrox" {
                found_clone = true;
                assert!(traits.list.contains(&"Mutated".to_string()), "Cloned extinct species should gain the 'Mutated' trait due to genetic drift.");
                break;
            }
        }

        assert!(found_clone, "A CloneEvent should spawn a new instance of the species.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/gene_banks.rs
use bevy::prelude::*;
use crate::layer1::buildings::CryoBank;
use crate::layer1::fauna::{Fauna, SpeciesId, Traits};
use crate::layer1::genetics::{CloneEvent, ExtinctionEvent};

pub fn process_cloning_system(
    mut commands: Commands,
    mut events: EventReader<CloneEvent>,
    query: Query<&CryoBank>,
) {
    for event in events.read() {
        if let Ok(bank) = query.get(event.bank_entity) {
            // Check if sample exists
            if bank.stored_samples.iter().any(|s| s.id == event.species_id) {
                // Spawn the clone with genetic drift
                commands.spawn((
                    Fauna,
                    SpeciesId { id: event.species_id.clone() },
                    Traits { list: vec!["Mutated".to_string()] }, // MVP static mutation
                ));
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **RNG Drift:** Hardcoding "Mutated" is boring. It needs to pull from a weighted pool of traits (e.g., +Size, -Speed, +Aggressive, -Docile) using a seeded RNG so every resurrection is slightly unpredictable.
- **DNA Collection:** How do samples get *into* the bank? There needs to be a "Take Sample" action for Medical/Science Pops to execute on living entities before they go extinct.
- **Extinction Tracker:** The game needs a global resource to track if `count(Species_X) == 0`. The cloning UI should probably only allow resurrection if the species is actually extinct or critically endangered to balance the feature.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_cloning_extinct_species_applies_genetic_drift` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- The `Traits` component must be the same one used by the rest of the Fauna behavior AI so that adding `Aggressive` actually makes them attack Pops.
- Consider adding a cost (Energy/Biomass) to the `CloneEvent` execution so it's not a free "undo" button for ecological collapse.

## 8. Questions
*Builder: add questions here if spec is unclear.*
