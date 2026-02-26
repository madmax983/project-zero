use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::stress::StressTracker;
use crate::layer1::utility_types::{ActionType, PopAction};
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct HumMap {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
}

impl HumMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![0.0; width * height],
        }
    }

    pub fn get(&self, x: i32, y: i32) -> f32 {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return 0.0;
        }
        self.values[y as usize * self.width + x as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, value: f32) {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return;
        }
        self.values[y as usize * self.width + x as usize] = value;
    }
}

#[derive(Component)]
pub struct HumSource {
    pub radius: f32,
    pub intensity: f32,
}

pub fn update_hum_system(mut hum_map: ResMut<HumMap>, sources: Query<(&HumSource, &GridPosition)>) {
    // Reset map
    hum_map.values.fill(0.0);

    for (source, pos) in sources.iter() {
        let radius_i = source.radius.ceil() as i32;
        let center_x = pos.x;
        let center_y = pos.y;
        let radius_sq = source.radius * source.radius;

        for dy in -radius_i..=radius_i {
            for dx in -radius_i..=radius_i {
                let dist_sq = (dx * dx + dy * dy) as f32;
                if dist_sq <= radius_sq {
                    let dist = dist_sq.sqrt();
                    // Linear falloff: 1.0 at center, 0.0 at radius
                    let falloff = (1.0 - (dist / source.radius)).max(0.0);
                    let intensity = source.intensity * falloff;

                    let x = center_x + dx;
                    let y = center_y + dy;

                    // Use max to avoid excessive stacking, simulating a field
                    let current = hum_map.get(x, y);
                    hum_map.set(x, y, current.max(intensity));
                }
            }
        }
    }
}

pub fn execute_listen_to_hum_system(
    mut query: Query<(&mut Needs, &mut StressTracker, &PopAction)>,
) {
    for (mut needs, mut stress, action) in &mut query {
        if action.current == ActionType::ListenToTheHum {
            // Restore Leisure
            needs.leisure = (needs.leisure + 0.005).min(1.0);

            // Increase Stress
            stress.accumulated_stress += 0.1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::stress::StressTracker;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::utility_ai::ActionType;
    use crate::layer1::utility_eval_types::{PopEvalData, ScorableCandidate, UtilityAIBuffer};
    use crate::layer1::utility_types::{PopAction, UtilityWeights};

    #[test]
    fn test_hum_map_propagation() {
        let mut world = World::new();
        world.insert_resource(HumMap::new(10, 10));
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        // Spawn a Hum Source at (5, 5)
        world.spawn((
            HumSource { radius: 3.0, intensity: 1.0 },
            GridPosition { x: 5, y: 5 },
        ));

        // Run propagation
        let mut schedule = Schedule::default();
        schedule.add_systems(update_hum_system);
        schedule.run(&mut world);

        let map = world.resource::<HumMap>();

        // Center should be max intensity
        assert!((map.get(5, 5) - 1.0).abs() < f32::EPSILON);

        // Falloff check (e.g. at distance 1)
        // 1.0 - (1.0 / 3.0) = 0.666...
        assert!(map.get(6, 5) < 1.0);
        assert!(map.get(6, 5) > 0.0);

        // Out of range (distance 4)
        assert_eq!(map.get(9, 9), 0.0);
    }

    #[test]
    fn test_sensitive_trait_exists() {
        // Just verify the enum variant exists and compiles
        let sensitive = Trait::Sensitive;
        assert_eq!(sensitive.label(), "Sensitive");
    }

    #[test]
    fn test_evaluate_listen_to_hum_sensitive_pop() {
        // Setup AI context
        let mut buffer = UtilityAIBuffer::default();
        let hum_pos = GridPosition { x: 5, y: 5 };

        let pop_pos = GridPosition { x: 0, y: 0 };
        let mut traits = Traits::default();
        traits.add(Trait::Sensitive);

        let data = PopEvalData {
            entity: Entity::from_raw(0),
            pos: pop_pos,
            needs: Needs::default(), // Not stressed yet
            traits: Some(traits),
            weights: UtilityWeights::default(),
            action: Default::default(),
            equipment: None,
            carrying: None,
            carrying_item: None,
            mental_state: None,
            drafted: None,
            faction_member: None,
            penal_labor: None,
            breakdown: None,
            stress: 0.0,
            hobby_type: None,
            chemical_state: None,
            is_memetic_carrier: false,
            health: None,
            insulation: 0.0,
        };

        // Add a Hum Source candidate
        let mut candidate = ScorableCandidate::new(Entity::from_raw(1), hum_pos);
        candidate.score_bonus = 1.0; // Intensity
        buffer.hum_sources.push(candidate);

        // This function must be implemented in the Green phase
        // We use the direct path once the module is created
        let (action, score, target) = crate::layer1::actions::evaluate_listen_to_hum(&data, &buffer);

        assert_eq!(action, ActionType::ListenToTheHum);
        assert!(score > 0.0);
        assert_eq!(target, Some(Entity::from_raw(1)));
    }

    #[test]
    fn test_evaluate_listen_to_hum_normal_pop_ignores_it() {
        let mut buffer = UtilityAIBuffer::default();
        let hum_pos = GridPosition { x: 5, y: 5 };

        let mut candidate = ScorableCandidate::new(Entity::from_raw(1), hum_pos);
        candidate.score_bonus = 1.0;
        buffer.hum_sources.push(candidate);

        let data = PopEvalData {
            entity: Entity::from_raw(0),
            pos: GridPosition { x: 0, y: 0 },
            needs: Needs::default(),
            weights: UtilityWeights::default(),
            traits: Some(Traits::default()), // Normal pop
            action: Default::default(),
            equipment: None,
            carrying: None,
            carrying_item: None,
            mental_state: None,
            drafted: None,
            faction_member: None,
            penal_labor: None,
            breakdown: None,
            stress: 0.0,
            hobby_type: None,
            chemical_state: None,
            is_memetic_carrier: false,
            health: None,
            insulation: 0.0,
        };

        let (action, score, _) = crate::layer1::actions::evaluate_listen_to_hum(&data, &buffer);

        // Should return Idle/None or very low score (0.0)
        if action == ActionType::ListenToTheHum {
            assert_eq!(score, 0.0);
        }
    }

    #[test]
    fn test_execute_listen_to_hum_system() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(execute_listen_to_hum_system);

        let pop = world
            .spawn((
                Needs {
                    leisure: 0.1,
                    ..Default::default()
                },
                StressTracker::default(),
                PopAction {
                    current: ActionType::ListenToTheHum,
                    ..Default::default()
                },
            ))
            .id();

        schedule.run(&mut world);

        let needs = world.get::<Needs>(pop).unwrap();
        let stress = world.get::<StressTracker>(pop).unwrap();

        assert!(needs.leisure > 0.1, "Leisure should increase");
        assert!(stress.accumulated_stress > 0.0, "Stress should increase");
    }
}
