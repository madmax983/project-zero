# 678 - The Atmospheric Smugglers

## 1. Overview
The Atmospheric Smugglers is a Cross-layer feature where black market operators hide volatile contraband gases directly inside the colony's primary life support and atmospheric grid (`AtmosphereGrid`). Rather than storing illegal goods in physical containers, Pops vent them into the air, relying on specific atmospheric processors to extract them later. This results in colony-wide physiological effects (hallucinations, respiratory issues) and forces the player to choose between purging the grid (destroying the evidence and crashing the black market) or tracing the extraction point.

## 2. Dependencies
- Layer 1 `AtmosphereGrid` (Fluid dynamics).
- Layer 1 `Pop` health/needs (respiratory effects/hallucinations).
- The Black Market system (from `specs/348-the-black-market.md`).
- `Chronicle` for reporting widespread colony symptoms.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::AtmosphereGrid;
    use crate::layer1::health::{Health, Condition};
    use crate::layer1::economy::black_market::Contraband;

    #[test]
    fn test_smugglers_vent_contraband_gas_into_atmosphere() {
        let mut app = App::new();
        app.add_systems(Update, smuggler_vent_gas_system);

        let grid_res = AtmosphereGrid::new(10, 10);
        app.world_mut().insert_resource(grid_res);

        // Spawn a smuggler entity at position (5,5) with contraband gas
        app.world_mut().spawn((
            Smuggler {},
            Position { x: 5, y: 5 },
            Inventory {
                item: ContrabandType::VolatileGas,
                amount: 100,
            },
        ));

        // Trigger the vent action
        app.world_mut().send_event(VentGasEvent {
            entity: Entity::from_raw(1),
            amount: 50,
        });

        app.update();

        // Assert the atmosphere grid now contains the contraband gas at (5,5)
        let grid = app.world().get_resource::<AtmosphereGrid>().unwrap();
        let gas_level = grid.get_gas_level(5, 5, GasType::Contraband);
        assert!(gas_level >= 50.0, "Contraband gas should be present in the atmosphere");
    }

    #[test]
    fn test_pops_suffer_physiological_effects_from_gas() {
        let mut app = App::new();
        app.add_systems(Update, gas_exposure_system);

        let mut grid_res = AtmosphereGrid::new(10, 10);
        // Fill the grid with contraband gas
        grid_res.set_gas_level(2, 2, GasType::Contraband, 100.0);
        app.world_mut().insert_resource(grid_res);

        // Spawn a pop at the same location
        let pop_ent = app.world_mut().spawn((
            Pop {},
            Position { x: 2, y: 2 },
            Health { value: 100.0 },
            Condition { hallucinations: false },
        )).id();

        app.update();

        // Assert the pop suffered health damage and is hallucinating
        let health = app.world().get::<Health>(pop_ent).unwrap();
        let condition = app.world().get::<Condition>(pop_ent).unwrap();
        assert!(health.value < 100.0, "Pop should take damage from toxic contraband gas");
        assert!(condition.hallucinations, "Pop should be hallucinating from the gas");
    }

    #[test]
    fn test_atmosphere_purge_destroys_contraband() {
        let mut app = App::new();
        app.add_systems(Update, purge_atmosphere_system);

        let mut grid_res = AtmosphereGrid::new(10, 10);
        grid_res.set_gas_level(5, 5, GasType::Contraband, 100.0);
        app.world_mut().insert_resource(grid_res);

        app.world_mut().send_event(PurgeAtmosphereEvent { sector: SectorId(0) });
        app.update();

        let grid = app.world().get_resource::<AtmosphereGrid>().unwrap();
        let gas_level = grid.get_gas_level(5, 5, GasType::Contraband);
        assert_eq!(gas_level, 0.0, "Purge should remove all contraband gas");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Smuggler {}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum GasType {
    Oxygen,
    Toxin,
    Contraband,
}

#[derive(Resource)]
pub struct AtmosphereGrid {
    pub cells: std::collections::HashMap<(i32, i32), std::collections::HashMap<GasType, f32>>,
}

impl AtmosphereGrid {
    pub fn new(width: i32, height: i32) -> Self {
        Self { cells: std::collections::HashMap::new() }
    }

    pub fn get_gas_level(&self, x: i32, y: i32, gas: GasType) -> f32 {
        self.cells.get(&(x, y)).and_then(|g| g.get(&gas)).copied().unwrap_or(0.0)
    }

    pub fn set_gas_level(&mut self, x: i32, y: i32, gas: GasType, amount: f32) {
        self.cells.entry((x, y)).or_default().insert(gas, amount);
    }
}

#[derive(Event)]
pub struct VentGasEvent {
    pub entity: Entity,
    pub amount: u32,
}

#[derive(Event)]
pub struct PurgeAtmosphereEvent {
    pub sector: SectorId,
}

pub fn smuggler_vent_gas_system(
    mut events: EventReader<VentGasEvent>,
    mut grid: ResMut<AtmosphereGrid>,
    query: Query<&Position, With<Smuggler>>,
) {
    for event in events.read() {
        if let Ok(pos) = query.get(event.entity) {
            let current = grid.get_gas_level(pos.x, pos.y, GasType::Contraband);
            grid.set_gas_level(pos.x, pos.y, GasType::Contraband, current + event.amount as f32);
        }
    }
}

pub fn gas_exposure_system(
    grid: Res<AtmosphereGrid>,
    mut query: Query<(&Position, &mut Health, &mut Condition)>,
) {
    for (pos, mut health, mut condition) in query.iter_mut() {
        let gas_level = grid.get_gas_level(pos.x, pos.y, GasType::Contraband);
        if gas_level > 10.0 {
            health.value -= 5.0; // Toxic damage
            condition.hallucinations = true;
        }
    }
}

pub fn purge_atmosphere_system(
    mut events: EventReader<PurgeAtmosphereEvent>,
    mut grid: ResMut<AtmosphereGrid>,
) {
    for _event in events.read() {
        for gas_map in grid.cells.values_mut() {
            gas_map.insert(GasType::Contraband, 0.0);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** Modify the `AtmosphereGrid` integration to use the actual implementation in `src/layer1/map.rs` or `src/layer1/terrain.rs`.
- **Code Smells:** Direct iteration over the grid's hash map during a purge is slow for large maps. It's better to implement a decay function or filter by active bounding boxes.
- **Performance:** Ensure diffusion algorithms for the contraband gas are identical to oxygen/toxins so it spreads naturally.
- **API Improvements:** The purge should probably trigger an `AddChronicleEvent` and cause significant unrest among black market factions.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Contraband gas diffuses through the `AtmosphereGrid` and negatively impacts Pop health.

## 7. Technical Guidance
- **Code Structure:** Place the smuggler logic in `src/layer1/economy/black_market.rs` and extend the grid in `src/layer1/map.rs`.
- **Integration Points:** Hook into the existing `AtmosphereGrid` diffusion systems. Add conditions to the UI so the player can see "Unidentified Gas Concentration".
- **Gotchas:** Make sure the gas isn't infinitely expanding. It should decay naturally over time or be absorbed by processors.

## 8. Questions
*Builder: add questions here if spec is unclear.*
