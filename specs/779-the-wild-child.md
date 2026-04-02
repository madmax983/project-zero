# The Wild Child (Spec 779)

## 1. Overview
**Layer:** 1
**Fantasy:** The planet claims your children. Nature vs. Nurture.
**Mechanic:** Children left unattended in "Wild" zones for too long may gain the "Feral" trait. They gain buffs to Movement/Combat/Animal Handling but refuse to sleep indoors, wear clothes, or do "Intellectual" work.
**Emergence:** A famine forces parents to work double shifts. The kids play in the woods. Ten years later, your militia is composed of spear-wielding, wolf-riding beastmasters who barely speak Common.
**Tension:** Protect the youth (Childcare/School) or let the planet toughen them (Feral)?

## 2. Dependencies
- `layer1::pop` (Age, Traits, Morale)
- `layer1::map` (Zone/Terrain types, specifically "Wild" or un-designated zones)
- `layer1::jobs` (Ability to restrict jobs based on traits)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Age, Traits, Trait};
    use crate::layer1::map::{Position, TerrainType};
    use crate::layer1::jobs::JobType;
    use crate::shared::time::SimulationTime;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(SimulationTime::default());
        app.add_systems(Update, (
            wild_child_exposure_system,
            apply_feral_traits_system,
        ));
        app
    }

    #[test]
    fn test_child_in_wild_gains_feral_exposure() {
        let mut app = setup_app();

        let child = app.world_mut().spawn((
            Pop::default(),
            Age { years: 5, ..Default::default() },
            Position { x: 10, y: 10 },
            Traits::default(),
            WildExposure { ticks: 0 },
        )).id();

        // Assume (10, 10) is a Wild tile.
        // For MVP we just use a marker component or resource for the map
        app.world_mut().insert_resource(MockMap { wild_tiles: vec![(10, 10)] });

        app.update();

        let exposure = app.world().get::<WildExposure>(child).unwrap();
        assert!(exposure.ticks > 0, "Child in wild should gain exposure ticks");
    }

    #[test]
    fn test_high_exposure_grants_feral_trait() {
        let mut app = setup_app();

        let child = app.world_mut().spawn((
            Pop::default(),
            Age { years: 8, ..Default::default() },
            Position { x: 10, y: 10 },
            Traits::default(),
            WildExposure { ticks: 1000 }, // Above threshold
        )).id();

        app.update();

        let traits = app.world().get::<Traits>(child).unwrap();
        assert!(traits.has(&Trait::Feral), "High exposure should grant Feral trait");
    }

    #[test]
    fn test_feral_trait_blocks_intellectual_jobs() {
        let mut traits = Traits::default();
        traits.add(Trait::Feral);

        assert!(!traits.can_do_job(&JobType::Research), "Feral pop should not be able to Research");
        assert!(traits.can_do_job(&JobType::Hunt), "Feral pop should be able to Hunt");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::pop::{Pop, Age, Traits, Trait};
use crate::layer1::map::{Position, TerrainType};
use crate::layer1::jobs::JobType;

#[derive(Component, Default)]
pub struct WildExposure {
    pub ticks: u32,
}

// In a real implementation this relies on `TerrainGrid` or similar
#[derive(Resource)]
pub struct MockMap {
    pub wild_tiles: Vec<(i32, i32)>,
}

impl MockMap {
    pub fn is_wild(&self, pos: &Position) -> bool {
        self.wild_tiles.contains(&(pos.x, pos.y))
    }
}

const FERAL_THRESHOLD: u32 = 1000;

pub fn wild_child_exposure_system(
    map: Res<MockMap>, // Or TerrainGrid
    mut query: Query<(&Position, &Age, &mut WildExposure)>,
) {
    for (pos, age, mut exposure) in query.iter_mut() {
        if age.years < 16 && map.is_wild(pos) {
            exposure.ticks += 1;
        } else if exposure.ticks > 0 && !map.is_wild(pos) {
            // Slowly decays when indoors/in civilization
            exposure.ticks -= 1;
        }
    }
}

pub fn apply_feral_traits_system(
    mut query: Query<(&WildExposure, &mut Traits)>,
) {
    for (exposure, mut traits) in query.iter_mut() {
        if exposure.ticks >= FERAL_THRESHOLD {
            if !traits.has(&Trait::Feral) {
                traits.add(Trait::Feral);
            }
        }
    }
}

// Update `Traits::can_do_job` in `src/layer1/pop.rs` to:
// if self.has(&Trait::Feral) && job.is_intellectual() { return false; }
```

## 5. REFACTOR Phase: Quality & Design
- **Map Integration:** `MockMap` must be replaced with the actual `TerrainGrid`. A tile should be considered "Wild" if it has natural flora (`TerrainType::Forest`, `Swamp`, etc.) and no `Room` or `Designation` zone applied.
- **Exposure Balance:** The threshold `FERAL_THRESHOLD` should be configured in a balance file, not hardcoded.
- **Adult Decay:** Consider what happens when a Feral child becomes an adult. Can they be rehabilitated? The MVP assumes the trait is permanent once gained.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Children standing on wild tiles accumulate `WildExposure`.
- [ ] Passing the exposure threshold grants `Trait::Feral`.
- [ ] Feral pops cannot perform Intellectual jobs (Research, Medicine, etc.).

## 7. Technical Guidance
- **Gotchas:** Make sure `WildExposure` is only added to children (`Age < 16`). Adults should not become feral this way.
- **System Placement:** Run `wild_child_exposure_system` on a slow tick (e.g. `SimulationTick` every second, not every frame) to save performance.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
