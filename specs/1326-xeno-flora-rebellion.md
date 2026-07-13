# 1326: Xeno-Flora Rebellion

## 1. Overview
Watching the alien environment actively fight back against industrial colonization through slow, creeping biological subversion. Native plant life on certain biomes reacts to high pollution or rapid deforestation by rapidly mutating and aggressively overgrowing colony infrastructure. Overgrown buildings suffer efficiency drops and require continuous manual clearing by Pops.

## 2. Dependencies
- `092` Antagonistic Flora
- `049` Industrial Waste / Pollution Map
- `017` Designation System (for clearing tasks)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::{GridPosition, TerrainGrid};
    use crate::layer1::pollution::PollutionGrid;
    use crate::layer1::flora::{XenoFlora, FloraState, update_flora_aggression};
    use crate::layer1::building::Building;

    #[test]
    fn test_high_pollution_triggers_aggression() {
        let mut flora = XenoFlora { state: FloraState::Dormant, aggression: 0.0 };
        let local_pollution = 80.0; // High pollution

        update_flora_aggression(&mut flora, local_pollution);

        assert!(flora.aggression > 0.0);
        assert_eq!(flora.state, FloraState::Aggressive);
    }

    #[test]
    fn test_aggressive_flora_overgrows_building() {
        let mut world = World::new();
        let pos = GridPosition { x: 5, y: 5 };

        let building = world.spawn((Building { efficiency: 1.0 }, pos.clone())).id();
        let mut flora = world.spawn((XenoFlora { state: FloraState::Aggressive, aggression: 100.0 }, pos)).id();

        crate::layer1::flora::apply_overgrowth_penalty(&mut world, building, flora);

        let b = world.get::<Building>(building).unwrap();
        assert!(b.efficiency < 1.0); // Efficiency reduced
    }

    #[test]
    fn test_flora_clearing_designation() {
        // Test that a pop can be assigned to clear the flora
        // and that clearing restores building efficiency
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal implementation in src/layer1/flora.rs
use bevy_ecs::prelude::*;
use crate::layer1::building::Building;

#[derive(PartialEq, Debug)]
pub enum FloraState {
    Dormant,
    Aggressive,
}

#[derive(Component)]
pub struct XenoFlora {
    pub state: FloraState,
    pub aggression: f32,
}

pub fn update_flora_aggression(flora: &mut XenoFlora, local_pollution: f32) {
    if local_pollution > 50.0 {
        flora.aggression += (local_pollution - 50.0) * 0.1;
        if flora.aggression > 20.0 {
            flora.state = FloraState::Aggressive;
        }
    }
}

pub fn apply_overgrowth_penalty(world: &mut World, building: Entity, flora: Entity) {
    if let Some(f) = world.get::<XenoFlora>(flora) {
        if f.state == FloraState::Aggressive {
            if let Some(mut b) = world.get_mut::<Building>(building) {
                // Reduce efficiency based on aggression, min 0.1
                b.efficiency = (1.0 - (f.aggression / 200.0)).max(0.1);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate this directly into the simulation tick. `apply_overgrowth_penalty` should be a system that queries overlapping `Building` and `XenoFlora` components.
- The pollution check should read from the `PollutionGrid` resource.
- Make the `Clear Flora` AI task available so pops can fight it back.

## 6. Acceptance Criteria (Testable!)
- [ ] Tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] High pollution causes Flora to enter the Aggressive state
- [ ] Aggressive Flora overlapping a Building reduces that building's efficiency

## 7. Technical Guidance
- Overlap detection can use `GridPosition`.
- Ensure that clearing the flora (setting aggression to 0 or despawning it) allows the building to return to 1.0 efficiency.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
