# 1025: Gravity Plating Failure

## 1. Overview
If gravity generators fail, affected base zones enter "Zero-G". This dramatically changes Layer 1 simulation mechanics: Movement becomes "Drifting," unanchored objects float away, and combat rules change. This introduces emergency management challenges but also allows for emergent strategies, like intentionally cutting gravity to neutralize raiders while zero-G trained militia pick them off.

## 2. Dependencies
- Layer 1 `TerrainGrid` / `Zone` system.
- Layer 1 `Power` system.
- Layer 1 `Movement` and `Physics` (Drifting).
- Layer 1 `Combat` and `Pop` traits (`ZeroGTraining`).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::power::{PowerNode, PowerGridEvent};
    use crate::layer1::terrain::{Zone, GravityState};
    use crate::layer1::movement::{MovementType, Velocity};
    use crate::layer1::pop::{Pop, TraitList};

    #[test]
    fn test_power_failure_disables_zone_gravity() {
        let mut app = App::new();
        app.add_event::<PowerGridEvent>();
        app.add_systems(Update, monitor_gravity_generator_power_system);

        let zone = app.world_mut().spawn(Zone { id: 1 }).id();

        let generator = app.world_mut().spawn((
            PowerNode { current_power: 0, required_power: 100 }, // Failed
            GravityGenerator { target_zone: zone },
        )).id();

        app.world_mut().resource_mut::<Events<PowerGridEvent>>().send(PowerGridEvent::NodeFailed(generator));

        app.update();

        // Zone should now have ZeroG state
        let gravity = app.world().get::<GravityState>(zone).unwrap();
        assert_eq!(*gravity, GravityState::ZeroG, "Zone should enter Zero-G when its gravity generator loses power.");
    }

    #[test]
    fn test_pops_in_zero_g_zone_switch_to_drifting() {
        let mut app = App::new();
        app.add_systems(Update, apply_zero_g_movement_system);

        let zone = app.world_mut().spawn((Zone { id: 1 }, GravityState::ZeroG)).id();

        // Un-trained Pop
        let pop = app.world_mut().spawn((
            Pop,
            MovementType::Walking,
            Velocity { x: 0.0, y: 0.0 },
            CurrentZone { zone },
            TraitList { traits: vec![] },
        )).id();

        // Trained Pop
        let trained_pop = app.world_mut().spawn((
            Pop,
            MovementType::Walking,
            Velocity { x: 0.0, y: 0.0 },
            CurrentZone { zone },
            TraitList { traits: vec!["ZeroGTraining".to_string()] },
        )).id();

        app.update();

        let move_type = app.world().get::<MovementType>(pop).unwrap();
        assert_eq!(*move_type, MovementType::Drifting, "Untrained Pops in Zero-G must drift.");

        let trained_move_type = app.world().get::<MovementType>(trained_pop).unwrap();
        assert_eq!(*trained_move_type, MovementType::ZeroGControlled, "Trained Pops in Zero-G should retain controlled movement.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/gravity_plating.rs
use bevy::prelude::*;
use crate::layer1::power::{PowerNode, PowerGridEvent};
use crate::layer1::terrain::{Zone, GravityState};
use crate::layer1::movement::{MovementType, Velocity};
use crate::layer1::pop::{Pop, TraitList};

#[derive(Component)]
pub struct GravityGenerator {
    pub target_zone: Entity,
}

#[derive(Component)]
pub struct CurrentZone {
    pub zone: Entity,
}

pub fn monitor_gravity_generator_power_system(
    mut events: EventReader<PowerGridEvent>,
    generator_query: Query<&GravityGenerator>,
    mut zone_query: Query<&mut GravityState>,
) {
    for event in events.read() {
        if let PowerGridEvent::NodeFailed(node_entity) = event {
            if let Ok(gen) = generator_query.get(*node_entity) {
                if let Ok(mut grav_state) = zone_query.get_mut(gen.target_zone) {
                    *grav_state = GravityState::ZeroG;
                }
            }
        }
    }
}

pub fn apply_zero_g_movement_system(
    zone_query: Query<&GravityState>,
    mut pop_query: Query<(&CurrentZone, &mut MovementType, &TraitList), With<Pop>>,
) {
    for (current_zone, mut move_type, traits) in pop_query.iter_mut() {
        if let Ok(grav_state) = zone_query.get(current_zone.zone) {
            if *grav_state == GravityState::ZeroG {
                if traits.traits.contains(&"ZeroGTraining".to_string()) {
                    *move_type = MovementType::ZeroGControlled;
                } else {
                    *move_type = MovementType::Drifting;
                }
            } else {
                *move_type = MovementType::Walking;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Physics Integration:** `MovementType::Drifting` needs actual physics integration. Drifting Pops should continue moving in a straight line until they hit a wall, ignoring pathfinding.
- **Combat Accuracy:** A `CombatModifier` component should be applied to drifting Pops, severely penalizing their aim or melee capability.
- **Item Floating:** Extend `apply_zero_g_movement_system` to handle loose Items on the floor, applying random drift velocities so the room becomes chaotic.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_power_failure_disables_zone_gravity` passes.
- [ ] Test `test_pops_in_zero_g_zone_switch_to_drifting` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- If `PowerGridEvent` doesn't exist, this might need to hook directly into the power `Update` tick to poll for `current_power < required_power`.
- Ensure `GravityState` enum is accessible by all movement/physics related systems.

## 8. Questions
*Builder: add questions here if spec is unclear.*
