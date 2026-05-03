use crate::layer1::energy::PowerSource;
use crate::layer1::map::GridPosition;
use crate::layer1::physics::acoustic::NoiseMap;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct AcousticGenerator {
    pub conversion_factor: f32,
}

pub fn acoustic_power_generation_system(
    noise_map: Res<NoiseMap>,
    mut generators: Query<(&GridPosition, &mut PowerSource, &AcousticGenerator)>,
) {
    for (pos, mut power_source, acoustic_generator) in generators.iter_mut() {
        let noise_level = noise_map.get(pos.x, pos.y);
        let generated_power = noise_level * acoustic_generator.conversion_factor;

        power_source.output = generated_power;
        power_source.active = generated_power > 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::physics::acoustic::NoiseMap;

    #[test]
    fn test_acoustic_power_generation() {
        let mut world = World::new();
        let mut map = NoiseMap::new(10, 10);
        map.set(5, 5, 0.8);
        world.insert_resource(map);

        let entity = world
            .spawn((
                GridPosition { x: 5, y: 5 },
                PowerSource {
                    output: 0.0,
                    active: false,
                },
                AcousticGenerator {
                    conversion_factor: 10.0,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(acoustic_power_generation_system);
        schedule.run(&mut world);

        let power_source = world.get::<PowerSource>(entity).unwrap();
        assert!((power_source.output - 8.0).abs() < f32::EPSILON);
        assert!(power_source.active);
    }
}
