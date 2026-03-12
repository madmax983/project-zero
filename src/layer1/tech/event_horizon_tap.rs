use crate::layer1::map::GridPosition;
use crate::layer1::pop::Speed;
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct EventHorizonTap {
    pub is_active: bool,
    pub instability: f32, // 0.0 to 1.0
}

pub fn apply_time_dilation_system(
    taps: Query<(&EventHorizonTap, &GridPosition)>,
    mut movers: Query<(&mut Speed, &GridPosition)>,
) {
    // Collect active zones based on instability
    let mut zones = Vec::new();
    for (tap, pos) in taps.iter() {
        if tap.is_active && tap.instability > 0.0 {
            // Radius scales with instability
            let radius = (tap.instability * 20.0) as u32;
            zones.push((*pos, radius, 0.1f32)); // 10% speed
        }
    }

    // Apply to movers
    for (mut speed, pos) in movers.iter_mut() {
        let mut current_mult: f32 = 1.0;

        for (zone_pos, radius, mult) in &zones {
            if pos.distance_chebyshev(*zone_pos) <= *radius {
                current_mult = current_mult.min(*mult);
            }
        }

        speed.current *= current_mult;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::energy::PowerSource;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Speed;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_event_horizon_tap_provides_infinite_energy() {
        let mut world = setup_world();

        let _tap_entity = world
            .spawn((
                EventHorizonTap {
                    is_active: true,
                    instability: 0.0,
                },
                PowerSource {
                    output: f32::INFINITY,
                    active: true,
                },
            ))
            .id();

        let power_source = world.get::<PowerSource>(_tap_entity).unwrap();
        assert_eq!(power_source.output, f32::INFINITY);
    }

    #[test]
    fn test_time_dilation_zone_slows_movement() {
        let mut world = setup_world();

        let _tap_entity = world
            .spawn((
                EventHorizonTap {
                    is_active: true,
                    instability: 0.5,
                }, // Triggers zone
                GridPosition { x: 10, y: 10 },
            ))
            .id();

        let pop = world
            .spawn((
                GridPosition { x: 10, y: 11 }, // Within radius
                Speed {
                    base: 10.0,
                    current: 5.0, // Assuming modified by something else
                    accumulator: 0.0,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_time_dilation_system);
        schedule.run(&mut world);

        let speed = world.get::<Speed>(pop).unwrap();
        // Movement speed should be heavily reduced (e.g., 10% of current)
        assert_eq!(speed.current, 0.5);
    }

    #[test]
    fn test_time_dilation_outside_zone_unaffected() {
        let mut world = setup_world();

        let _tap_entity = world
            .spawn((
                EventHorizonTap {
                    is_active: true,
                    instability: 0.5,
                },
                GridPosition { x: 10, y: 10 },
            ))
            .id();

        let pop = world
            .spawn((
                GridPosition { x: 50, y: 50 }, // Far away
                Speed {
                    base: 10.0,
                    current: 10.0,
                    accumulator: 0.0,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_time_dilation_system);
        schedule.run(&mut world);

        let speed = world.get::<Speed>(pop).unwrap();
        assert_eq!(speed.current, 10.0); // Unaffected
    }
}
