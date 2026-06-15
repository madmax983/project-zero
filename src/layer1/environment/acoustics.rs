use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
use crate::layer1::ActionType;
use crate::layer1::PopAction;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct NoiseEmitter {
    pub volume: f32,
}

#[derive(Resource)]
pub struct AcousticMap {
    pub width: i32,
    pub height: i32,
    pub grid: Vec<f32>,
}

impl AcousticMap {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            grid: vec![0.0; (width * height) as usize],
        }
    }

    pub fn get(&self, x: i32, y: i32) -> f32 {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return 0.0;
        }
        self.grid[(y * self.width + x) as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, val: f32) {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            self.grid[(y * self.width + x) as usize] = val;
        }
    }

    pub fn clear(&mut self) {
        self.grid.fill(0.0);
    }
}

pub fn propagate_noise_system(
    mut acoustic_map: ResMut<AcousticMap>,
    emitters: Query<(&GridPosition, &NoiseEmitter)>,
) {
    acoustic_map.clear();

    // Very naive flood fill for MVP
    for (pos, emitter) in emitters.iter() {
        let max_radius = emitter.volume as i32;

        for dx in -max_radius..=max_radius {
            for dy in -max_radius..=max_radius {
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist <= emitter.volume {
                    let current = acoustic_map.get(pos.x + dx, pos.y + dy);
                    let new_val = current.max(emitter.volume - dist);
                    acoustic_map.set(pos.x + dx, pos.y + dy, new_val);
                }
            }
        }
    }
}

pub fn apply_noise_stress_system(
    acoustic_map: Res<AcousticMap>,
    mut pops: Query<(&GridPosition, &mut StressTracker, Option<&PopAction>), With<Pop>>,
) {
    for (pos, mut stress, action) in pops.iter_mut() {
        let local_noise = acoustic_map.get(pos.x, pos.y);
        if local_noise > 0.0 {
            let is_sleeping = action.is_some_and(|a| matches!(a.current, ActionType::SatisfyRest));
            let multiplier = if is_sleeping { 2.0 } else { 1.0 };
            stress.accumulated_stress += local_noise * 0.1 * multiplier;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::stress::StressTracker;
    use crate::layer1::ActionType;
    use crate::layer1::PopAction;
    use bevy_app::App;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(AcousticMap::new(50, 50));
        app.add_systems(
            bevy_app::Update,
            (propagate_noise_system, apply_noise_stress_system),
        );
        app
    }

    #[test]
    fn test_noise_propagates_from_source() {
        let mut app = setup_app();

        // Spawn a loud machine
        app.world_mut()
            .spawn((NoiseEmitter { volume: 10.0 }, GridPosition { x: 10, y: 10 }));

        app.update();

        let acoustic_map = app.world().get_resource::<AcousticMap>().unwrap();

        // Epicenter should be loud
        assert_eq!(acoustic_map.get(10, 10), 10.0);
        // Adjacent tiles should be quieter but not 0
        assert!(acoustic_map.get(11, 10) > 0.0);
        assert!(acoustic_map.get(11, 10) < 10.0);
    }

    #[test]
    fn test_noise_increases_pop_stress() {
        let mut app = setup_app();

        // Set background noise high
        app.world_mut().resource_mut::<AcousticMap>().set(5, 5, 8.0);

        let pop_id = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                StressTracker {
                    accumulated_stress: 0.0,
                    ..Default::default()
                },
                PopAction {
                    current: ActionType::SatisfyRest,
                    ..Default::default()
                }, // Noise affects sleepers more
            ))
            .id();

        app.update();

        let stress = app.world().get::<StressTracker>(pop_id).unwrap();
        assert!(stress.accumulated_stress > 0.0);
    }
}
