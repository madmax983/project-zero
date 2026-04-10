# 939 - Language Drift

## 1. Overview
**Layer:** Cross-layer (Layer 2 / 3)
**Fantasy:** Watching isolated colonies evolve distinct, eventually mutually unintelligible cultures.
**Mechanic:** A "Linguistic Drift" value slowly increments over time based on the distance between colonies and the frequency of trade/travel. High drift imposes a "Translation Tax" on diplomacy, trade efficiency, and cross-colony migrations.
**Emergence:** You colonize the galactic rim and leave them to their own devices for centuries. When a crisis forces you to integrate their massive workforce into your core worlds, the sheer translation barrier causes administration to grind to a halt, and your native Pops treat the arrivals as hostile aliens despite sharing the same ancestors.
**Tension:** Spending valuable transport capacity to enforce "cultural synchronization flights" between worlds, versus letting them drift apart to save resources at the cost of eventual deep alienation.

## 2. Dependencies
- Needs Layer 2 `Colony` / `StarSystem` entities.
- Needs a mechanism to track cross-colony trade/travel frequency or distance.
- Needs shared time tracking (e.g., `SimulationTime`).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_linguistic_drift_increases_over_time_without_contact() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<LinguisticNetwork>()
            .add_systems(Update, language_drift_system);

        let colony_a = app.world_mut().spawn(Colony).id();
        let colony_b = app.world_mut().spawn(Colony).id();

        // Act
        app.update(); // Tick time

        // Assert
        let network = app.world().get_resource::<LinguisticNetwork>().unwrap();
        let drift = network.get_drift(colony_a, colony_b);
        assert!(drift > 0.0, "Drift should increase over time when isolated");
    }

    #[test]
    fn test_cultural_synchronization_reduces_drift() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_event::<CulturalSyncEvent>()
            .init_resource::<LinguisticNetwork>()
            .add_systems(Update, (language_drift_system, cultural_sync_system).chain());

        let colony_a = app.world_mut().spawn(Colony).id();
        let colony_b = app.world_mut().spawn(Colony).id();

        // Let drift accumulate
        app.update();

        let initial_drift = app.world().get_resource::<LinguisticNetwork>().unwrap().get_drift(colony_a, colony_b);

        // Act - Spawn a cultural sync event
        app.world_mut().send_event(CulturalSyncEvent {
            colony_a,
            colony_b,
            strength: 10.0,
        });

        app.update();

        // Assert
        let new_drift = app.world().get_resource::<LinguisticNetwork>().unwrap().get_drift(colony_a, colony_b);
        assert!(new_drift < initial_drift, "Cultural sync should reduce linguistic drift");
    }

    #[test]
    fn test_translation_tax_applied_on_trade() {
        // Outline test for translation tax.
        // Assume TradeEvent has a base_value, and the system modifies it based on drift.
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub struct Colony;

#[derive(Resource, Default)]
pub struct LinguisticNetwork {
    // Maps a pair of colonies (sorted) to their linguistic drift
    pub drift_map: HashMap<(Entity, Entity), f32>,
}

impl LinguisticNetwork {
    pub fn get_drift(&self, a: Entity, b: Entity) -> f32 {
        let key = if a < b { (a, b) } else { (b, a) };
        *self.drift_map.get(&key).unwrap_or(&0.0)
    }

    pub fn set_drift(&mut self, a: Entity, b: Entity, value: f32) {
        let key = if a < b { (a, b) } else { (b, a) };
        self.drift_map.insert(key, value.max(0.0));
    }
}

#[derive(Event)]
pub struct CulturalSyncEvent {
    pub colony_a: Entity,
    pub colony_b: Entity,
    pub strength: f32,
}

pub fn language_drift_system(
    colonies: Query<Entity, With<Colony>>,
    mut network: ResMut<LinguisticNetwork>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let drift_rate = 0.1; // Base drift per second

    let entities: Vec<Entity> = colonies.iter().collect();
    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            let a = entities[i];
            let b = entities[j];
            let current = network.get_drift(a, b);
            network.set_drift(a, b, current + drift_rate * dt);
        }
    }
}

pub fn cultural_sync_system(
    mut events: EventReader<CulturalSyncEvent>,
    mut network: ResMut<LinguisticNetwork>,
) {
    for event in events.read() {
        let current = network.get_drift(event.colony_a, event.colony_b);
        network.set_drift(event.colony_a, event.colony_b, current - event.strength);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Iterating over all pairs of colonies every frame is O(N^2) and wasteful.
- **Refactoring**: Tie the drift to a long-running timer (e.g., ticking once a month) instead of `Update`. Use `SimulationTime` instead of real time.
- **Performance**: The sparse matrix representation `HashMap<(Entity, Entity), f32>` is adequate, but we should make sure we only clean up entries when colonies are destroyed.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Linguistic drift increases automatically between colonies.
- [ ] Synchronization events reduce linguistic drift between target colonies.

## 7. Technical Guidance
- **Edge Components vs Resources**: Since drift is a relationship between two entities, representing it as a global Resource `ResMut<LinguisticNetwork>` holding a sparse matrix or graph is much cleaner than component-based maps.
- **Integration**: Tie the `CulturalSyncEvent` into existing Layer 2 trade fleet logic or a specific "Cultural Mission" edict.
- **UI Seams**: Ensure drift values are exposed so they can be rendered on the system map or diplomacy screens.

## 8. Questions
*Builder: add questions here if spec is unclear.*
