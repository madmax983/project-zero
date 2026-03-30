# Spec 757: The "Mother" Lode

## 1. Overview
**Layer:** 1
**Fantasy:** The legendary vein that never ends.
**Mechanic:** A procedural ore node that is incredibly rich/infinite but increases in "Hazard" (Heat, Radiation, Hardness) the more you mine it.
**Emergence:** You build your economy around the Mother Lode. It becomes so radioactive that miners can only work 1-hour shifts. You keep sending them in.
**Tension:** Greed vs. Worker Health.

## 2. Dependencies
- Base Layer 1 infrastructure (Mining, Actions).
- `TerrainGrid` and `ResourceNode` tracking.
- Health/Status systems (e.g., Radiation/Heat tracking on Pops).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::execution::mining::{MiningAction, Mineable};
    use crate::layer1::needs::Health;

    #[test]
    fn test_mother_lode_increases_hazard_on_mining() {
        let mut app = App::new();

        let node_id = app.world_mut().spawn((
            Mineable { resource: "mother_ore".into(), base_yield: 10 },
            MotherLode { current_hazard_level: 0.0, hazard_growth_rate: 0.1 }
        )).id();

        // Simulate a mining cycle completion
        app.add_systems(Update, process_mother_lode_hazard_system);
        app.world_mut().resource_mut::<Events<MiningCompletedEvent>>()
            .send(MiningCompletedEvent { entity: node_id, miner: Entity::PLACEHOLDER });

        app.update();

        let lode = app.world().get::<MotherLode>(node_id).unwrap();
        assert!(lode.current_hazard_level > 0.0, "Hazard level should increase after mining");
    }

    #[test]
    fn test_miner_receives_hazard_damage() {
        let mut app = App::new();

        let miner_id = app.world_mut().spawn(Health { current: 100.0, max: 100.0 }).id();
        let node_id = app.world_mut().spawn((
            Mineable { resource: "mother_ore".into(), base_yield: 10 },
            MotherLode { current_hazard_level: 5.0, hazard_growth_rate: 0.1 }
        )).id();

        app.add_systems(Update, apply_mother_lode_hazard_to_miners_system);

        // Add an active mining action
        app.world_mut().spawn(MiningAction { target: node_id, worker: miner_id });

        app.update();

        let health = app.world().get::<Health>(miner_id).unwrap();
        assert!(health.current < 100.0, "Miner should receive damage from high hazard level");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct MotherLode {
    pub current_hazard_level: f32,
    pub hazard_growth_rate: f32,
}

#[derive(Event)]
pub struct MiningCompletedEvent {
    pub entity: Entity,
    pub miner: Entity,
}

// Dummy struct to satisfy tests
#[derive(Component)]
pub struct MiningAction {
    pub target: Entity,
    pub worker: Entity,
}

pub fn process_mother_lode_hazard_system(
    mut events: EventReader<MiningCompletedEvent>,
    mut query: Query<&mut MotherLode>,
) {
    for event in events.read() {
        if let Ok(mut lode) = query.get_mut(event.entity) {
            lode.current_hazard_level += lode.hazard_growth_rate;
        }
    }
}

pub fn apply_mother_lode_hazard_to_miners_system(
    action_query: Query<&MiningAction>,
    lode_query: Query<&MotherLode>,
    mut health_query: Query<&mut crate::layer1::needs::Health>,
) {
    for action in action_query.iter() {
        if let Ok(lode) = lode_query.get(action.target) {
            if lode.current_hazard_level > 0.0 {
                if let Ok(mut health) = health_query.get_mut(action.worker) {
                    health.current -= lode.current_hazard_level;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Damage Types**: Instead of directly subtracting `Health.current`, emit a generic `ApplyDamageEvent` (or specific `RadiationDamage` / `HeatDamage` components) so that pops with resistant traits or protective gear can mitigate the damage.
- **Yield Scaling**: Consider reducing the ore yield or increasing the time-to-mine as the hazard grows, representing the "hardness" described in the mechanic.
- **Utility AI**: Integrate hazard awareness into the `UtilityEvaluator`. Pops with low health should heavily down-weight mining actions directed at the Mother Lode to prevent a mass-suicide pipeline.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code
- [ ] Pops take damage (or specific status effects) when mining highly hazardous nodes.

## 7. Technical Guidance
- Hooks into `Layer1SystemSet::Execution` alongside regular mining.
- Consider adding a visual indicator (VFX) or UI tooltip that scales with the `current_hazard_level` so the player understands why their miners are dying.

## 8. Questions
*Builder: add questions here if spec is unclear.*
