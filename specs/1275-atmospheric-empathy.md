# 1275 - Atmospheric Empathy

## 1. Overview
A colony so interconnected that the very air they breathe is shaped by their collective mood. Pops with extreme moods (e.g., ecstatic, furious, terrified) in enclosed spaces slightly alter the chemical composition of trace gases in the air. These gases change the tint of the atmosphere and apply subtle mood modifiers to everyone breathing it.

## 2. Dependencies
- `src/layer1/nature/atmosphere.rs` (Atmosphere Grid)
- `src/layer1/social/morale.rs` (Morale and MoodModifier)
- `src/layer1/entities/pop.rs` (Pop component)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::social::morale::{Morale, MoodModifier};
    use crate::layer1::nature::atmosphere::AtmosphereGrid;
    use crate::layer1::map::GridPosition;
    use crate::layer1::entities::pop::Pop;

    #[test]
    fn test_extreme_mood_alters_atmosphere() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(AtmosphereGrid::new(10, 10));
        world.insert_resource(TraceGasGrid::new(10, 10));

        let mut ecstatic_morale = Morale::default();
        ecstatic_morale.value = 1.0;

        let mut terrified_morale = Morale::default();
        terrified_morale.value = 0.0;

        world.spawn((Pop, GridPosition { x: 5, y: 5 }, ecstatic_morale));
        world.spawn((Pop, GridPosition { x: 2, y: 2 }, terrified_morale));

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(emit_trace_gases);
        schedule.run(&mut world);

        // Assert
        let gas_grid = world.resource::<TraceGasGrid>();
        assert!(gas_grid.get_gas(5, 5, GasType::Euphoric) > 0.0, "Ecstatic pop should emit euphoric gas");
        assert!(gas_grid.get_gas(2, 2, GasType::Fear) > 0.0, "Terrified pop should emit fear gas");
    }

    #[test]
    fn test_breathing_trace_gases_alters_mood() {
        // Arrange
        let mut world = World::new();
        let mut gas_grid = TraceGasGrid::new(10, 10);
        gas_grid.add_gas(5, 5, GasType::Euphoric, 0.5);
        world.insert_resource(gas_grid);

        let entity = world.spawn((Pop, GridPosition { x: 5, y: 5 }, Morale::default())).id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_atmospheric_empathy);
        schedule.run(&mut world);

        // Assert
        let morale = world.get::<Morale>(entity).unwrap();
        assert!(
            morale.modifiers.iter().any(|m| m.label == "Breathed Euphoria" && m.value > 0.0),
            "Pop should gain mood modifier from breathing euphoric gas"
        );
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy_ecs::prelude::*;
use crate::layer1::social::morale::{Morale, MoodModifier};
use crate::layer1::nature::atmosphere::AtmosphereGrid;
use crate::layer1::map::GridPosition;
use crate::layer1::entities::pop::Pop;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GasType {
    Euphoric,
    Fear,
    Rage,
}

#[derive(Resource)]
pub struct TraceGasGrid {
    width: usize,
    height: usize,
    gases: Vec<std::collections::HashMap<GasType, f32>>,
}

impl TraceGasGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            gases: vec![std::collections::HashMap::new(); width * height],
        }
    }

    pub fn get_gas(&self, x: usize, y: usize, gas_type: GasType) -> f32 {
        let idx = y * self.width + x;
        *self.gases[idx].get(&gas_type).unwrap_or(&0.0)
    }

    pub fn add_gas(&mut self, x: usize, y: usize, gas_type: GasType, amount: f32) {
        let idx = y * self.width + x;
        *self.gases[idx].entry(gas_type).or_insert(0.0) += amount;
    }
}

pub fn emit_trace_gases(
    mut gas_grid: ResMut<TraceGasGrid>,
    query: Query<(&GridPosition, &Morale), With<Pop>>,
) {
    for (pos, morale) in query.iter() {
        if morale.value > 0.9 {
            gas_grid.add_gas(pos.x as usize, pos.y as usize, GasType::Euphoric, 0.1);
        } else if morale.value < 0.1 {
            gas_grid.add_gas(pos.x as usize, pos.y as usize, GasType::Fear, 0.1);
        }
    }
}

pub fn apply_atmospheric_empathy(
    gas_grid: Res<TraceGasGrid>,
    mut query: Query<(&GridPosition, &mut Morale), With<Pop>>,
) {
    for (pos, mut morale) in query.iter_mut() {
        let euphoric = gas_grid.get_gas(pos.x as usize, pos.y as usize, GasType::Euphoric);
        if euphoric > 0.1 {
            morale.modifiers.push(MoodModifier {
                label: "Breathed Euphoria".to_string(),
                value: 0.05,
                duration: 10,
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Performance**: Integrate trace gases directly into `AtmosphereGrid` using SoA (Struct of Arrays) to avoid HashMap overhead per tile.
- **Ventilation**: Ensure ventilation structures (if they exist) clear trace gases.
- **Visuals**: Hook into the UI rendering system to tint the screen slightly based on ambient gas type.

## 6. Acceptance Criteria (Testable!)
- [ ] RED phase tests pass with minimal implementation.
- [ ] `cargo test` returns 0 failures in the modified module.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code.
- [ ] Extreme moods accurately emit their respective trace gases.
- [ ] Ambient trace gases correctly apply mood modifiers to nearby Pops.

## 7. Technical Guidance
- Integrate the new `TraceGasGrid` (or extended `AtmosphereGrid`) into the main simulation schedule.
- Be careful with gas accumulation; ensure there is a decay factor so gases don't build up to infinity over time.
- Consider adding a `ProtectedFromAtmosphere` check to `apply_atmospheric_empathy` if pops are in sealed suits.

## 8. Questions
*Builder: add questions here if spec is unclear.*
