use bevy_ecs::prelude::*;
use crate::layer1::social::morale::{Morale, MoodModifier};
use crate::layer1::map::GridPosition;
use crate::layer1::entities::pop::Pop;
use crate::layer1::nature::atmosphere::ProtectedFromAtmosphere;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GasType {
    Euphoric,
    Fear,
    Rage,
}

#[derive(Resource)]
pub struct TraceGasGrid {
    pub width: usize,
    pub height: usize,
    pub euphoric: Vec<f32>,
    pub fear: Vec<f32>,
    pub rage: Vec<f32>,
}

impl TraceGasGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            euphoric: vec![0.0; size],
            fear: vec![0.0; size],
            rage: vec![0.0; size],
        }
    }

    pub fn get_gas(&self, x: usize, y: usize, gas_type: GasType) -> f32 {
        if x >= self.width || y >= self.height {
            return 0.0;
        }
        let idx = y * self.width + x;
        match gas_type {
            GasType::Euphoric => self.euphoric[idx],
            GasType::Fear => self.fear[idx],
            GasType::Rage => self.rage[idx],
        }
    }

    pub fn add_gas(&mut self, x: usize, y: usize, gas_type: GasType, amount: f32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = y * self.width + x;
        match gas_type {
            GasType::Euphoric => self.euphoric[idx] += amount,
            GasType::Fear => self.fear[idx] += amount,
            GasType::Rage => self.rage[idx] += amount,
        }
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
    mut query: Query<(&GridPosition, &mut Morale, Option<&ProtectedFromAtmosphere>), With<Pop>>,
) {
    for (pos, mut morale, protected) in query.iter_mut() {
        if protected.is_some() {
            continue;
        }
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

pub fn decay_trace_gases_system(
    mut gas_grid: ResMut<TraceGasGrid>,
) {
    for val in gas_grid.euphoric.iter_mut() {
        *val *= 0.99;
    }
    for val in gas_grid.fear.iter_mut() {
        *val *= 0.99;
    }
    for val in gas_grid.rage.iter_mut() {
        *val *= 0.99;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
