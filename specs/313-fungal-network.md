# Specification 313: The Fungal Network

## 1. Overview
This feature introduces "The Fungal Network," a planet-wide mycelial network. Building "Spore Taps" allows instant, zero-cost resource transfer. However, extended connection slowly alters the ethics of connected pops towards "Collectivism" and occasionally sends "Urges" (mandatory quests).

## 2. Dependencies
- `map` system (`src/layer1/map.rs`).
- `needs`/`morals` system (`src/layer1/needs.rs` or `src/layer1/social/grievances.rs`).

## 3. RED Phase: Tests First

```rust
// src/layer1/fungal_network.rs
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, process_spore_taps_system);
        app.insert_resource(SporeNetwork { active_taps: 0 });
        app
    }

    #[test]
    fn test_spore_tap_increases_network_activity() {
        let mut app = setup_app();

        // Spawn a tap
        app.world_mut().spawn(SporeTap);
        app.update();

        let network = app.world().resource::<SporeNetwork>();
        assert_eq!(network.active_taps, 1, "The network should count active taps.");
    }

    #[test]
    fn test_network_alters_pop_ethics() {
        let mut app = setup_app();

        app.world_mut().spawn(SporeTap);
        let pop = app.world_mut().spawn(PopCollectivism { level: 0.0 }).id();

        app.update(); // Tick 1

        let collectivism = app.world().get::<PopCollectivism>(pop).unwrap();
        assert!(collectivism.level > 0.0, "Collectivism should increase over time when a tap is active.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/fungal_network.rs
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct SporeNetwork {
    pub active_taps: u32,
}

#[derive(Component)]
pub struct SporeTap;

#[derive(Component)]
pub struct PopCollectivism {
    pub level: f32,
}

pub fn process_spore_taps_system(
    mut network: ResMut<SporeNetwork>,
    tap_query: Query<&SporeTap>,
    mut pop_query: Query<&mut PopCollectivism>,
) {
    network.active_taps = tap_query.iter().count() as u32;

    if network.active_taps > 0 {
        for mut pop in pop_query.iter_mut() {
            pop.level += 0.1 * network.active_taps as f32; // Increase collectivism
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: The actual "zero-cost resource transfer" needs to interface with `ColonyResources` or a unified storage pool, bypassing hauling logic.
- **Lore**: A unique event when the network "wakes up" and makes its first demand should be sent to the Chronicle.
- **Urges**: The network occasionally sending "Urges" can be modeled as temporary quests or priority overrides for connected Pops.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for `fungal_network.rs`.
- [ ] Active taps increase global `active_taps` count.
- [ ] Pops near or connected to the network gain `PopCollectivism`.

## 7. Technical Guidance
- The collectivism increase should be extremely slow but persistent.
- "SporeTaps" act like buildings and should be placeable on the grid like typical `BuildingType`s.

## 8. Questions
*Builder: add questions here if spec is unclear.*
