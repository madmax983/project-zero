# 681 - Feral Logistics Network

## 1. Overview
The Feral Logistics Network is a Layer 2 mechanic where an abandoned network of autonomous cargo drones spans the star system. You can connect your stations to it, but the drones deliver goods based on a corrupted algorithm rather than your orders. The player might send food and receive radioactive waste, or send nothing and suddenly receive a fleet's worth of weapons. This introduces the tension of potentially incredible output vs. absolute chaos when relying on a broken, alien AI to manage supply lines.

## 2. Dependencies
- Layer 2 `FleetMovement` system (from `specs/099-fleet-movement.md`).
- Layer 1 `ColonyResources` (receiving random goods).
- A corruption mechanism / RNG.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::logistics::FeralLogisticsNetwork;
    use crate::layer1::resources::ColonyResources;
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_feral_network_delivers_random_cargo() {
        let mut app = App::new();
        app.add_systems(Update, process_feral_logistics);

        let target_colony = app.world_mut().spawn(ColonyResources::default()).id();
        app.world_mut().insert_resource(FeralLogisticsNetwork {
            connected_colonies: vec![target_colony],
        });

        // Advance time to trigger a delivery
        app.world_mut().send_event(FeralDeliveryTriggerEvent { target: target_colony });

        app.update();

        let resources = app.world().get::<ColonyResources>(target_colony).unwrap();
        // Since it's randomized, just verify it's no longer entirely empty/default.
        assert!(
            resources.food > 0 || resources.metal > 0 || resources.waste > 0,
            "Feral Logistics should deliver random items to connected colonies"
        );
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use rand::Rng;

#[derive(Resource)]
pub struct FeralLogisticsNetwork {
    pub connected_colonies: Vec<Entity>,
}

#[derive(Component, Default)]
pub struct ColonyResources {
    pub food: u32,
    pub metal: u32,
    pub waste: u32,
}

#[derive(Event)]
pub struct FeralDeliveryTriggerEvent {
    pub target: Entity,
}

pub fn process_feral_logistics(
    mut events: EventReader<FeralDeliveryTriggerEvent>,
    mut colonies: Query<&mut ColonyResources>,
) {
    let mut rng = rand::thread_rng();

    for event in events.read() {
        if let Ok(mut resources) = colonies.get_mut(event.target) {
            let choice = rng.gen_range(0..=2);
            let amount = rng.gen_range(10..=100);
            match choice {
                0 => resources.food += amount,
                1 => resources.metal += amount,
                2 => resources.waste += amount,
                _ => {}
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** Abstract the item types received from the feral network to use generic `ItemType` or `Commodity` enums instead of hardcoding `food/metal/waste`.
- **Code Smells:** Ensure that RNG is properly seeded for determinism, using `bevy_prng` or a centralized seed resource.
- **Performance:** If connecting hundreds of nodes to the feral network, batch the randomized delivery distributions.
- **API Improvements:** Provide UI feedback indicating *what* the network dropped and when, potentially triggering an `AddChronicleEvent`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Connecting to the network introduces chaotic, randomized resource influxes.

## 7. Technical Guidance
- **Code Structure:** Implement in `src/layer2/feral_logistics.rs`.
- **Integration Points:** Connect the `FeralLogisticsNetwork` to `src/layer2/system_map.rs` where orbital nodes can be toggled to connect.
- **Gotchas:** Make sure the negative consequences (like receiving radioactive waste) are clear to the player so it feels like a genuine risk, not just free stuff.

## 8. Questions
*Builder: add questions here if spec is unclear.*
