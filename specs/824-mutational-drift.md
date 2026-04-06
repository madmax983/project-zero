# 824: Mutational Drift

## 1. Overview
Generations of colonists born on worlds with extreme conditions (high radiation, extreme cold, toxic atmospheres) naturally develop minor, beneficial genetic mutations. Over time, these mutations accumulate, effectively turning the population into a new sub-species highly adapted to their local environment. However, the homeworld empire (Layer 3) views these physiological changes with deep suspicion. As mutational drift increases, the core worlds apply mounting diplomatic penalties, trade tariffs, and eventually outright hostility, forcing the colony to choose between localized efficiency and galactic integration.

## 2. Dependencies
- `src/layer1/pop.rs` for tracking Pop generations and environment exposure.
- `src/layer1/nature/environment.rs` for tracking planet hazard levels.
- `src/layer3/diplomacy.rs` for applying diplomatic penalties from the homeworld.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_pops_born_in_radiation_gain_mutational_drift() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<PlanetaryEnvironment>();
        app.add_systems(Update, process_pop_birth_mutations);

        app.world_mut().resource_mut::<PlanetaryEnvironment>().hazard_level = HazardLevel::HighRadiation;

        let newborn_pop = app.world_mut().spawn((
            Pop { generation: 2, ..default() },
            MutationLevel { drift: 0 },
        )).id();

        // Act
        app.update();

        // Assert
        let mutation = app.world().get::<MutationLevel>(newborn_pop).unwrap();
        // The newborn should inherit/develop mutation drift due to the high radiation environment
        assert!(mutation.drift > 0);
    }

    #[test]
    fn test_high_mutational_drift_causes_diplomatic_penalty() {
        // Test that if the average mutational drift of the colony exceeds a threshold,
        // the Layer 3 Homeworld Diplomatic relations suffer a negative modifier.
    }

    #[test]
    fn test_mutated_pops_ignore_local_hazards() {
        // Test that Pops with high mutational drift do not suffer stress or health penalties
        // when working in the extreme environment that caused the mutation.
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct MutationLevel {
    pub drift: u32,
}

#[derive(PartialEq, Debug)]
pub enum HazardLevel {
    None,
    HighRadiation,
}

#[derive(Resource, Default)]
pub struct PlanetaryEnvironment {
    pub hazard_level: HazardLevel,
}

pub fn process_pop_birth_mutations(
    env: Res<PlanetaryEnvironment>,
    mut query: Query<&mut MutationLevel, Added<Pop>>,
) {
    if env.hazard_level == HazardLevel::HighRadiation {
        for mut mutation in query.iter_mut() {
            // New births in extreme conditions gain base mutational drift
            mutation.drift += 10;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate average colony mutational drift efficiently, perhaps caching it as a Resource on Layer 1 to easily bridge it to Layer 3 diplomacy.
- Allow mutations to provide localized productivity buffs in hazardous jobs to counter the diplomatic penalties.
- Add specific tags for *types* of mutations based on the specific hazard (e.g., "Radiation-Hardened", "Cold-Adapted").

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] `Added<Pop>` correctly assigns initial mutational drift based on current `PlanetaryEnvironment` hazard.

## 7. Technical Guidance
- `Added<Pop>` query is the perfect place to inject birth mechanics without needing a complex event system, as long as it's checked exactly once when the Pop is spawned.
- Be careful with how diplomatic penalties scale; it shouldn't instantly trigger a war just because a few babies were born with glowing eyes. The threshold should be a percentage of the total population or an average drift value.

## 8. Questions
*Builder: add questions here if spec is unclear.*
