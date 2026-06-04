use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::mind::utility_types::{ActionType, PopAction};
use crate::layer1::temperature::TemperatureGrid;

#[derive(Component)]
pub struct ThermalSignature {
    pub current_heat: f32,
    pub detection_radius: f32,
}

#[derive(Component)]
pub struct ThermalSensor {
    pub sensitivity: f32,
    pub targets: Vec<Entity>,
}

#[derive(Component)]
pub struct ThermalSuit {
    pub dampening_factor: f32,
}

pub fn update_thermal_signatures(
    temp_grid: Option<Res<TemperatureGrid>>,
    mut query: Query<(&GridPosition, &PopAction, &mut ThermalSignature, Option<&ThermalSuit>)>,
) {
    for (pos, action, mut sig, suit) in query.iter_mut() {
        let ambient = temp_grid.as_ref()
            .filter(|_| pos.x >= 0 && pos.y >= 0)
            .map(|g| g.get(pos.x as usize, pos.y as usize)).unwrap_or(10.0);

        match action.current {
            ActionType::Idle => {
                sig.current_heat = (sig.current_heat - 1.0).max(ambient);
            }
            ActionType::Work => {
                sig.current_heat = (sig.current_heat + 5.0).min(100.0);
            }
            ActionType::Fight => {
                sig.current_heat = (sig.current_heat + 20.0).min(200.0);
            }
            _ => {
                sig.current_heat = (sig.current_heat + 1.0).min(50.0);
            }
        }

        // Radius scales with heat
        let dampening = suit.map(|s| s.dampening_factor).unwrap_or(0.0);
        sig.detection_radius = (sig.current_heat * 0.1) * (1.0 - dampening);
    }
}

pub fn predator_detection_system(
    mut predators: Query<(Entity, &GridPosition, &mut ThermalSensor)>,
    targets: Query<(Entity, &GridPosition, &ThermalSignature)>,
) {
    for (pred_ent, pred_pos, mut sensor) in predators.iter_mut() {
        sensor.targets.clear();
        for (target_ent, target_pos, target_sig) in targets.iter() {
            if pred_ent == target_ent {
                continue;
            }
            let dx = pred_pos.x as f32 - target_pos.x as f32;
            let dy = pred_pos.y as f32 - target_pos.y as f32;
            let dist = ((dx * dx + dy * dy) as f32).sqrt();

            if dist <= target_sig.detection_radius + sensor.sensitivity {
                sensor.targets.push(target_ent);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_activity_generates_thermal_signature() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, update_thermal_signatures);

        let idle_pop = app.world_mut().spawn((
            GridPosition { x: 0, y: 0 },
            PopAction { current: ActionType::Idle, ..Default::default() },
            ThermalSignature { current_heat: 30.0, detection_radius: 0.0 },
        )).id();

        let working_pop = app.world_mut().spawn((
            GridPosition { x: 0, y: 0 },
            PopAction { current: ActionType::Work, ..Default::default() },
            ThermalSignature { current_heat: 30.0, detection_radius: 0.0 },
        )).id();

        // Act
        app.update();

        // Assert
        let idle_sig = app.world().get::<ThermalSignature>(idle_pop).unwrap();
        let work_sig = app.world().get::<ThermalSignature>(working_pop).unwrap();

        assert!(work_sig.current_heat > idle_sig.current_heat);
        assert!(work_sig.detection_radius > idle_sig.detection_radius);
    }

    #[test]
    fn test_predator_detects_hot_targets() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, predator_detection_system);

        let predator = app.world_mut().spawn((
            GridPosition { x: 0, y: 0 },
            ThermalSensor { sensitivity: 0.5, targets: vec![] },
        )).id();

        let hot_pop = app.world_mut().spawn((
            GridPosition { x: 5, y: 0 },
            ThermalSignature { current_heat: 100.0, detection_radius: 10.0 },
        )).id();

        let cold_pop = app.world_mut().spawn((
            GridPosition { x: 2, y: 0 }, // Closer, but cold
            ThermalSignature { current_heat: 10.0, detection_radius: 1.0 },
        )).id();

        // Act
        app.update();

        // Assert: Predator detects hot_pop but not cold_pop
        let sensor = app.world().get::<ThermalSensor>(predator).unwrap();
        assert!(sensor.targets.contains(&hot_pop));
        assert!(!sensor.targets.contains(&cold_pop));
    }
}
