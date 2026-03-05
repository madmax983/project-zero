# Specification 308: Black Market Genemods

## 1. Overview
This feature simulates illegal genetic modification clinics ("Ripperdocs") that set up shop when medical quality is low but industrial demand is high. Pops can secretly visit these clinics to gain incredibly powerful but unstable genetic traits (`Genemod`). These mods provide massive temporary buffs to work speed or needs, but carry a ticking hidden "Instability" stat. When instability reaches a threshold, the Pop suffers a "Mutational Meltdown," transforming into a grotesque, hostile anomaly that attacks the colony.

## 2. Dependencies
- `Traits` system (`src/layer1/traits.rs`)
- `Job` system / Work speed modifiers (`src/layer1/execution/general_work.rs`)
- `Combat` system (`src/layer1/combat.rs`)
- `Building` system (`src/layer1/buildings.rs` for the Ripperdoc clinic)

## 3. RED Phase: Tests First

```rust
// src/layer1/tech/black_market_genemods.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::traits::PopTrait;
    use crate::layer1::combat::{Health, Hostile};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            update_genemod_instability_system,
            process_mutational_meltdown_system,
        ));
        app
    }

    #[test]
    fn test_genemod_increases_instability_over_time() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn(UnstableGenemod { instability: 0.0, mod_type: GenemodType::MuscleGraft }).id();

        app.update();

        let genemod = app.world().get::<UnstableGenemod>(pop).unwrap();
        assert!(genemod.instability > 0.0);
    }

    #[test]
    fn test_meltdown_transforms_pop_into_hostile_anomaly() {
        let mut app = setup_app();

        // Spawn pop near meltdown threshold
        let pop = app.world_mut().spawn((
            Health { current: 100.0, max: 100.0 },
            UnstableGenemod { instability: 99.5, mod_type: GenemodType::Wakefulness },
        )).id();

        app.update();
        app.update(); // Push over 100.0

        // Pop should no longer have UnstableGenemod
        assert!(app.world().get::<UnstableGenemod>(pop).is_none());

        // Pop should now be hostile
        assert!(app.world().get::<Hostile>(pop).is_some());

        // Pop health should be massive
        let health = app.world().get::<Health>(pop).unwrap();
        assert!(health.max > 100.0);
    }

    #[test]
    fn test_genemod_buffs_work_speed() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn(UnstableGenemod { instability: 0.0, mod_type: GenemodType::MuscleGraft }).id();

        // Simulating the check inside get_job_efficiency_modifier
        let modifier = get_genemod_efficiency_modifier(&app.world().get::<UnstableGenemod>(pop).unwrap());

        assert!(modifier > 1.0); // Should be a massive buff (e.g., 2.0x)
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/tech/black_market_genemods.rs

use bevy::prelude::*;
use crate::layer1::combat::{Health, Hostile};

#[derive(Clone, Debug, PartialEq)]
pub enum GenemodType {
    MuscleGraft,
    Wakefulness,
}

#[derive(Component, Clone, Debug)]
pub struct UnstableGenemod {
    pub instability: f32,
    pub mod_type: GenemodType,
}

pub fn update_genemod_instability_system(
    mut query: Query<&mut UnstableGenemod>,
) {
    for mut genemod in query.iter_mut() {
        genemod.instability += 0.1; // Slow increase
    }
}

pub fn process_mutational_meltdown_system(
    mut commands: Commands,
    mut query: Query<(Entity, &UnstableGenemod, &mut Health)>,
) {
    for (entity, genemod, mut health) in query.iter_mut() {
        if genemod.instability >= 100.0 {
            // Transform into an anomaly
            health.max = 500.0;
            health.current = 500.0;

            commands.entity(entity)
                .remove::<UnstableGenemod>()
                // In full implementation, remove Pop component and insert Hostile/Monster logic
                .insert(Hostile);
        }
    }
}

pub fn get_genemod_efficiency_modifier(genemod: &UnstableGenemod) -> f32 {
    match genemod.mod_type {
        GenemodType::MuscleGraft => 2.0, // Double work speed
        GenemodType::Wakefulness => 1.5, // 50% faster, plus theoretically lower sleep need
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: The buff `get_genemod_efficiency_modifier` needs to be directly wired into `src/layer1/execution/general_work.rs` alongside traits like `Hunchback` or `SilverTongue`.
- **Ripperdoc logic**: We need a spawning mechanism. When colony health/medical score is low and industrial demand is high, spawn a `ShadowTrader`-esque "Ripperdoc" clinic on the map that Pops can pathfind to.
- **Narrative**: Add an `AddChronicleEvent` for both when a pop gets modded (rumor spreading) and when they melt down (catastrophe).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for `black_market_genemods.rs`.
- [ ] Instability increments correctly per tick.
- [ ] Reaching 100.0 instability transforms the pop into a Hostile entity and buffs its health.
- [ ] Work speed is modified if the pop has a genemod.

## 7. Technical Guidance
- The transformation into a hostile anomaly requires stripping standard `Pop` components (like `Needs`, `Job`, `Civilian`) and inserting standard monster components (`CombatState`, `Hostile`, `WanderAI`).
- The `GenemodType::Wakefulness` should theoretically alter the rate of rest decay in `metabolism_system` (Layer 1 needs).

## 8. Questions
*Builder: add questions here if spec is unclear.*
