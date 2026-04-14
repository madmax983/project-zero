# 1016: Mobile Architecture

## 1. Overview
Mobile Architecture allows buildings to be constructed on "Chassis" foundations. These buildings can transform into a mobile mode (Walker/Treads) to relocate slowly across the map, consuming fuel. This supports nomadic colony styles, allowing players to perpetually move a "City on Wheels" to stay in sunlight or relocate mining towns when ore runs dry.

## 2. Dependencies
- Layer 1 `TerrainGrid` and pathfinding.
- Layer 1 `Building` structures.
- Layer 1 `Economy` (Fuel consumption).
- Layer 1 `Movement` system.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::terrain::GridPosition;
    use crate::layer1::buildings::{Building, BuildingState};
    use crate::layer1::economy::FuelReserves;
    use crate::layer1::movement::MoveCommand;

    #[test]
    fn test_mobile_building_transforms_state() {
        let mut app = App::new();
        app.add_event::<TransformCommand>();
        app.add_systems(Update, transform_building_system);

        let building = app.world_mut().spawn((
            Building,
            MobileChassis { mode: MobilityMode::Stationary },
            BuildingState::Operational,
        )).id();

        app.world_mut().resource_mut::<Events<TransformCommand>>().send(TransformCommand {
            target: building,
            new_mode: MobilityMode::Mobile,
        });

        app.update();

        let chassis = app.world().get::<MobileChassis>(building).unwrap();
        let state = app.world().get::<BuildingState>(building).unwrap();

        assert_eq!(chassis.mode, MobilityMode::Mobile, "Building chassis should change to Mobile mode.");
        assert_eq!(*state, BuildingState::Offline, "Building must go Offline while mobile.");
    }

    #[test]
    fn test_mobile_building_consumes_fuel_to_move() {
        let mut app = App::new();
        app.insert_resource(FuelReserves { amount: 100 });
        app.add_event::<MoveCommand>();
        app.add_systems(Update, move_mobile_building_system);

        let building = app.world_mut().spawn((
            Building,
            GridPosition { x: 0, y: 0, z: 0 },
            MobileChassis { mode: MobilityMode::Mobile },
        )).id();

        app.world_mut().resource_mut::<Events<MoveCommand>>().send(MoveCommand {
            entity: building,
            destination: GridPosition { x: 1, y: 0, z: 0 },
        });

        app.update();

        let fuel = app.world().resource::<FuelReserves>();
        let pos = app.world().get::<GridPosition>(building).unwrap();

        assert_eq!(fuel.amount, 90, "Moving a building should consume fuel (MVP cost 10).");
        assert_eq!(pos.x, 1, "Building should have moved to the destination.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/mobile_architecture.rs
use bevy::prelude::*;
use crate::layer1::terrain::GridPosition;
use crate::layer1::buildings::{Building, BuildingState};
use crate::layer1::economy::FuelReserves;
use crate::layer1::movement::MoveCommand;

#[derive(Clone, PartialEq, Debug)]
pub enum MobilityMode {
    Stationary,
    Mobile,
}

#[derive(Component)]
pub struct MobileChassis {
    pub mode: MobilityMode,
}

#[derive(Event)]
pub struct TransformCommand {
    pub target: Entity,
    pub new_mode: MobilityMode,
}

const MOVE_FUEL_COST: u32 = 10;

pub fn transform_building_system(
    mut events: EventReader<TransformCommand>,
    mut query: Query<(&mut MobileChassis, &mut BuildingState)>,
) {
    for event in events.read() {
        if let Ok((mut chassis, mut state)) = query.get_mut(event.target) {
            chassis.mode = event.new_mode.clone();
            if chassis.mode == MobilityMode::Mobile {
                *state = BuildingState::Offline;
            } else {
                *state = BuildingState::Operational;
            }
        }
    }
}

pub fn move_mobile_building_system(
    mut events: EventReader<MoveCommand>,
    mut fuel: ResMut<FuelReserves>,
    mut query: Query<(&MobileChassis, &mut GridPosition)>,
) {
    for event in events.read() {
        if let Ok((chassis, mut pos)) = query.get_mut(event.entity) {
            if chassis.mode == MobilityMode::Mobile && fuel.amount >= MOVE_FUEL_COST {
                // Update position
                *pos = event.destination.clone();
                // Consume fuel
                fuel.amount -= MOVE_FUEL_COST;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Multi-Tile Buildings:** If buildings occupy multiple tiles on the `TerrainGrid`, the `MoveCommand` needs to handle footprint translation and collision detection, not just a single `GridPosition`.
- **Pathfinding:** The MVP teleports the building 1 tile at a time. It needs to hook into the A* pathfinding system so large structures can navigate around terrain obstacles.
- **Transformation Time:** Transforming from Stationary to Mobile shouldn't be instant. It should require an Action duration and perhaps Engineer Pops to execute.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_mobile_building_transforms_state` passes.
- [ ] Test `test_mobile_building_consumes_fuel_to_move` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- Ensure `BuildingState::Offline` prevents the building from performing its normal functions (e.g., generating power, refining ore) while the chassis is in motion.
- You may need a distinct `MoveCommand` for large structures vs Pops if their collision metrics differ significantly.

## 8. Questions
*Builder: add questions here if spec is unclear.*
