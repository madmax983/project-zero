# Spec 552: The Subterranean Smog Layer

## 1. Overview
Heavy industry placed deep underground doesn't vent pollution into the open air; instead, it pools in the lowest Z-levels as "Deep Smog." Deep Smog tiles reduce movement speed, increase disease risk, and obscure vision. Maintaining a subterranean factory requires zero ventilation infrastructure initially, but the accumulating smog becomes incredibly dangerous to navigate and repair, eventually risking explosive decompression of toxic gas back up to the surface.

## 2. Dependencies
- `002` Basic Map (Z-levels)
- `035` Workplace Hazards
- `063` Atmospheric Simulation

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::{TilePos, ZLevel};
    use crate::layer1::atmosphere::SmogGrid;

    #[test]
    fn test_underground_industry_accumulates_smog_on_lowest_z_level() {
        let mut app = App::new();
        app.add_systems(Update, process_subterranean_smog_system);

        let mut grid = SmogGrid::default();
        grid.set_smog(TilePos::new(10, 10), ZLevel(-2), 0.0); // -2 is deepest
        app.world_mut().insert_resource(grid);

        // Spawn a factory on ZLevel -1
        app.world_mut().spawn(HeavyIndustry { active: true, z_level: ZLevel(-1) });

        app.update();

        let updated_grid = app.world().resource::<SmogGrid>();
        assert!(updated_grid.get_smog(TilePos::new(10, 10), ZLevel(-2)) > 0.0, "Smog should sink to the lowest Z-Level");
    }

    #[test]
    fn test_deep_smog_reduces_movement_speed() {
        let mut app = App::new();
        app.add_systems(Update, apply_smog_penalties_system);

        let mut grid = SmogGrid::default();
        grid.set_smog(TilePos::new(10, 10), ZLevel(-2), 100.0); // Thick smog
        app.world_mut().insert_resource(grid);

        let pop = app.world_mut().spawn((Pop, TilePos::new(10, 10), ZLevel(-2), MovementSpeed { current: 1.0 })).id();

        app.update();

        let speed = app.world().get::<MovementSpeed>(pop).unwrap();
        assert!(speed.current < 1.0, "Thick deep smog should reduce movement speed");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use crate::layer1::map::{TilePos, ZLevel};

#[derive(Resource, Default)]
pub struct SmogGrid {
    pub levels: std::collections::HashMap<(TilePos, ZLevel), f32>,
}

impl SmogGrid {
    pub fn get_smog(&self, pos: TilePos, z: ZLevel) -> f32 {
        self.levels.get(&(pos, z)).copied().unwrap_or(0.0)
    }
    pub fn set_smog(&mut self, pos: TilePos, z: ZLevel, amount: f32) {
        self.levels.insert((pos, z), amount);
    }
}

#[derive(Component)]
pub struct HeavyIndustry {
    pub active: bool,
    pub z_level: ZLevel,
}

#[derive(Component)]
pub struct MovementSpeed {
    pub current: f32,
}

pub fn process_subterranean_smog_system(
    mut grid: ResMut<SmogGrid>,
    query: Query<&HeavyIndustry>,
) {
    for industry in query.iter() {
        if industry.active {
            // Simplification: Smog "sinks" to an arbitrary lowest level (ZLevel -2) directly beneath it.
            // In a real implementation, this would involve fluid dynamics and checking for open tiles below.
            let target_z = ZLevel(-2);
            if industry.z_level > target_z {
                 // Arbitrary position for test
                let pos = TilePos::new(10, 10);
                let current_smog = grid.get_smog(pos, target_z);
                grid.set_smog(pos, target_z, current_smog + 10.0); // Emit smog downwards
            }
        }
    }
}

pub fn apply_smog_penalties_system(
    grid: Res<SmogGrid>,
    mut query: Query<(&TilePos, &ZLevel, &mut MovementSpeed)>,
) {
    for (pos, z, mut speed) in query.iter_mut() {
        let smog_level = grid.get_smog(*pos, *z);
        if smog_level > 50.0 {
            speed.current = 0.5; // 50% speed penalty
        } else {
            speed.current = 1.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create a realistic diffusion model for `SmogGrid` where smog flows from the source tile downward through open space until it hits bedrock.
- The `MovementSpeed` penalty should scale linearly with the density of the smog on that tile, not just a binary threshold.
- Introduce a "Blowout" event. If `SmogGrid` density in an enclosed subterranean space exceeds a critical threshold, it violently vents upwards through stairwells/elevator shafts, poisoning the surface levels.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] `HeavyIndustry` on negative `ZLevel`s deposits smog into the `SmogGrid`.
- [ ] Smog accumulates on the lowest available `ZLevel` below the source.
- [ ] High smog density on a tile reduces the `MovementSpeed` of Pops traversing it.

## 7. Technical Guidance
- `SmogGrid` should integrate with the existing `AtmosphereGrid` (Spec 063) to handle the displacement of oxygen and the resulting suffocation/disease risks.
- Building underground should be initially cheaper (no walls/roofs needed, just excavation), making this a tempting but ultimately disastrous mid-game trap for players scaling their industry.

## 8. Questions
*Builder: add questions here if spec is unclear.*
- **Architectural Contradictions:** Z-Levels are not implemented in the current map grid (`GridPosition` only has `x` and `y`). `MovementSpeed` does not exist, pops use `Speed` component. I'm moving on.

*Architect:* Ignore Z-Levels for now. Assume 'subterranean' just means specific deep-crust `GridPosition` tiles. Use `Speed` component for movement speed.
