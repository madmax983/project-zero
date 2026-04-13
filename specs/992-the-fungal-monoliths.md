# Specification: The Fungal Monoliths

## 1. Overview
The Fungal Monoliths introduce ancient, petrified fungal structures into Layer 1 (Colony) that are disguised as normal ruins. When excavated, they release "Euphoric Spores", causing massive morale boosts at the cost of catastrophic, delayed starvation, while completely consuming the surrounding terrain to grow larger.

## 2. Dependencies
- Layer 1 Core (Grid, Pops, Morale, Needs system)
- Layer 1 Buildings (Ruins, Excavation mechanics)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    // Assume existing systems and components exist
    #[derive(Component)]
    struct Pop;

    #[derive(Component)]
    struct Morale(f32);

    #[derive(Component)]
    struct Needs { hunger: f32, sleep: f32 };

    #[derive(Component)]
    struct FungalMonolith {
        growth_stage: u32,
    };

    #[derive(Component)]
    struct EuphoricSporeCloud;

    #[derive(Component)]
    struct Excavatable;

    // The system to test
    fn fungal_monolith_eruption_system(
        mut commands: Commands,
        query: Query<(Entity, &FungalMonolith, &Transform)>,
        mut pops: Query<(&mut Morale, &mut Needs), With<Pop>>,
    ) {
        // Implementation will go here
    }

    fn spawn_test_world() -> World {
        World::new()
    }

    #[test]
    fn test_monolith_spore_euphoria() {
        let mut world = spawn_test_world();

        let pop_entity = world.spawn().id();
        world.entity_mut(pop_entity).insert((
            Pop,
            Morale(50.0),
            Needs { hunger: 50.0, sleep: 50.0 }
        ));

        let monolith_entity = world.spawn().id();
        world.entity_mut(monolith_entity).insert((
            FungalMonolith { growth_stage: 1 },
            Transform::from_xyz(0.0, 0.0, 0.0)
        ));

        // Act: Run the eruption system (spores affect Pops)
        let mut schedule = Schedule::default();
        schedule.add_systems(fungal_monolith_eruption_system);
        schedule.run(&mut world);

        // Assert: Morale is locked at 100%, Needs are ignored/paused
        let morale = world.get::<Morale>(pop_entity).unwrap();
        assert_eq!(morale.0, 100.0, "Euphoric spores should lock Morale at 100%");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation to make the test pass
fn fungal_monolith_eruption_system(
    mut commands: Commands,
    query: Query<(Entity, &FungalMonolith, &Transform)>,
    mut pops: Query<(&mut Morale, &mut Needs), With<Pop>>,
) {
    for (_, _, _) in query.iter() {
        // Simple logic for RED phase: apply effects to all Pops in testing
        for (mut morale, mut needs) in pops.iter_mut() {
            morale.0 = 100.0;
            // Freeze needs conceptually (or reset to full, depending on system implementation)
            needs.hunger = 0.0;
            needs.sleep = 0.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Refactoring:** Update `fungal_monolith_eruption_system` to calculate Euclidean or Manhattan distance from the Monolith `Transform` to only affect Pops within the `EuphoricSporeCloud` radius, rather than all Pops globally.
- **Code Smells:** Avoid hardcoding the 100.0 max morale or 0.0 needs. Utilize existing constants for `MAX_MORALE`.
- **Performance:** Ensure that we don't recalculate spore cloud radius impacts every tick if they are static.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code.
- [ ] Fungal Monoliths erupt, locking Morale at 100% and halting Need decay (or simulating starvation exhaustion).

## 7. Technical Guidance
- Integrate into the `layer1/environment` or `layer1/hazards` modules.
- Ensure the `Pop` system correctly interfaces with the `Needs` freeze—this might require an `Euphoric` trait component appended to the Pop.

## 8. Questions
*Builder: add questions here if spec is unclear.*
