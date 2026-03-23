# Gene Splicing

## 1. Overview
Designing the perfect worker for a hellish world. This feature introduces a genetics and trait modification system to Layer 1. Lab research unlocks "Gene Mods" (e.g., Night Vision, Gill-Lungs, Stone-Skin). Applying them costs Medical resources and time, resulting in permanent `Trait` additions to Pops. However, it risks "Rejection" (temporary massive health debuffs) or permanent unwanted mutations (like "Light Blindness"), forcing the player to balance extreme specialization (efficiency) against biological flexibility (universality).

## 2. Dependencies
- `084-pop-traits` (The core trait system)
- `038-medical-care` (Medical resources and treatments)
- `107-biocompatibility` (Planetary sickness mechanics)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_gene_mod_adds_trait() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let pop_entity = app.world.spawn((
            Pop,
            Traits::new(),
            Health { current: 100.0, max: 100.0 },
        )).id();

        // Setup successful gene splicing
        app.world.send_event(GeneSplicingEvent {
            target: pop_entity,
            mod_type: GeneMod::StoneSkin,
            success_chance: 1.0, // 100% success
        });

        app.add_systems(Update, process_gene_splicing_system);
        app.update();

        let traits = app.world.get::<Traits>(pop_entity).unwrap();
        assert!(traits.has_trait(TraitType::StoneSkin), "Pop should acquire StoneSkin trait");
    }

    #[test]
    fn test_gene_mod_rejection_causes_health_damage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let pop_entity = app.world.spawn((
            Pop,
            Traits::new(),
            Health { current: 100.0, max: 100.0 },
        )).id();

        // Setup failed gene splicing
        app.world.send_event(GeneSplicingEvent {
            target: pop_entity,
            mod_type: GeneMod::NightVision,
            success_chance: 0.0, // 0% success
        });

        app.add_systems(Update, process_gene_splicing_system);
        app.update();

        let health = app.world.get::<Health>(pop_entity).unwrap();
        assert!(health.current < 100.0, "Pop should take damage from rejection");

        let traits = app.world.get::<Traits>(pop_entity).unwrap();
        assert!(!traits.has_trait(TraitType::NightVision), "Pop should not acquire the intended trait on failure");
    }

    #[test]
    fn test_gene_mod_failure_causes_mutation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let pop_entity = app.world.spawn((
            Pop,
            Traits::new(),
            Health { current: 100.0, max: 100.0 },
        )).id();

        // Force failure
        app.world.send_event(GeneSplicingEvent {
            target: pop_entity,
            mod_type: GeneMod::GillLungs,
            success_chance: 0.0,
        });

        app.add_systems(Update, process_gene_splicing_system);
        app.update();

        // Assert: A negative mutation trait should be added
        let traits = app.world.get::<Traits>(pop_entity).unwrap();
        assert!(traits.has_trait(TraitType::LightBlindness) || traits.has_trait(TraitType::Frail), "Pop should acquire a negative mutation trait on failure");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use rand::Rng;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GeneMod {
    StoneSkin,
    NightVision,
    GillLungs,
}

#[derive(Event)]
pub struct GeneSplicingEvent {
    pub target: Entity,
    pub mod_type: GeneMod,
    pub success_chance: f32,
}

// Applies gene mods, rolling for success, rejection, or mutation
pub fn process_gene_splicing_system(
    mut events: EventReader<GeneSplicingEvent>,
    mut query: Query<(&mut Traits, &mut Health)>,
) {
    let mut rng = rand::thread_rng();

    for ev in events.read() {
        if let Ok((mut traits, mut health)) = query.get_mut(ev.target) {
            let roll: f32 = rng.gen();

            if roll <= ev.success_chance {
                // Success
                let new_trait = match ev.mod_type {
                    GeneMod::StoneSkin => TraitType::StoneSkin,
                    GeneMod::NightVision => TraitType::NightVision,
                    GeneMod::GillLungs => TraitType::GillLungs,
                };
                traits.add_trait(new_trait);
            } else {
                // Failure - Rejection and Mutation
                health.current -= 40.0; // Severe damage

                // 50% chance of a negative mutation on failure
                if rng.gen_bool(0.5) {
                    let mutation = if rng.gen_bool(0.5) {
                        TraitType::LightBlindness
                    } else {
                        TraitType::Frail
                    };
                    traits.add_trait(mutation);
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactor 1:** `process_gene_splicing_system` hardcodes the success/failure rolls. `success_chance` should be dynamically calculated based on the operating Doctor's `Medical` skill and the colony's current Tech Level.
- **Refactor 2:** `GeneSplicingEvent` should deduct `Medical` resources from `ColonyResources` before attempting the operation.
- **Refactor 3:** Create a `ChronicleEvent` for both successful ("A new breed of worker...") and failed ("A horrific mutation...") procedures.

## 6. Acceptance Criteria
- [ ] `cargo test` passes all RED phase tests with 0 failures.
- [ ] `cargo clippy -- -D warnings` returns 0 warnings.
- [ ] Test coverage hits at least 85% for `src/layer1/genetics.rs`.
- [ ] Successful `GeneSplicingEvent` permanently adds the targeted `TraitType` to the Pop.
- [ ] Failed `GeneSplicingEvent` reduces Pop Health by 40 and has a 50% chance to add a negative mutation `TraitType`.
- [ ] The `GeneMod` enum supports `StoneSkin`, `NightVision`, and `GillLungs`.

## 7. Technical Guidance
- Ensure `TraitType::StoneSkin`, `NightVision`, etc. are fully registered in `084-pop-traits`.
- Connect the `GeneSplicingEvent` to the Utility AI by treating it as an `ActionType::Surgery` that requires the Pop to be in a Hospital bed.
- Negative mutations like `LightBlindness` should heavily penalize efficiency when the Pop is exposed to daylight or bright `LightGrid` tiles.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
