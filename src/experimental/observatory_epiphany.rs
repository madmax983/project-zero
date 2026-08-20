use bevy_ecs::prelude::*;
use crate::layer1::architecture::building::{Building, BuildingType};
use crate::layer1::biology::health::Health;
use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::traits::{Trait, Traits};
use crate::layer1::resources::ColonyResources;

/// Pops with the Prophet trait who stand near an Observatory at Night
/// will gaze into the void, gaining Knowledge for the colony but losing Health.
pub fn observatory_epiphany_system(
    cycle: Option<Res<DayNightCycle>>,
    resources: Option<ResMut<ColonyResources>>,
    buildings: Query<(&GridPosition, &Building)>,
    mut pops: Query<(&GridPosition, &Traits, &mut Health), With<Pop>>,
) {
    let cycle = match cycle {
        Some(c) => c,
        None => return,
    };

    if cycle.time_of_day != TimeOfDay::Night {
        return;
    }

    let mut res = match resources {
        Some(r) => r,
        None => return,
    };

    let observatory_positions: Vec<GridPosition> = buildings
        .iter()
        .filter(|(_, b)| b.building_type == BuildingType::Observatory)
        .map(|(pos, _)| *pos)
        .collect();

    if observatory_positions.is_empty() {
        return;
    }

    for (pop_pos, traits, mut health) in pops.iter_mut() {
        if traits.has(Trait::Prophet) {
            for obs_pos in &observatory_positions {
                // Manually calculate Manhattan distance to avoid missing trait issues
                let dx = pop_pos.x.abs_diff(obs_pos.x);
                let dy = pop_pos.y.abs_diff(obs_pos.y);
                if (dx + dy) <= 3 {
                    // The Void stares back...
                    health.current -= 0.5;
                    res.knowledge += 0.5;
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::architecture::building::{Building, BuildingType};
    use crate::layer1::biology::health::Health;
    use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
    use crate::layer1::pop::Pop;
    use crate::layer1::psychology::traits::{Trait, Traits};
    use crate::layer1::resources::ColonyResources;

    #[test]
    fn test_observatory_epiphany() {
        let mut world = World::new();
        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Night,
            ..Default::default()
        });
        world.insert_resource(ColonyResources::default());

        // Spawn Observatory
        world.spawn((
            GridPosition { x: 5, y: 5 },
            Building {
                building_type: BuildingType::Observatory,
            },
        ));

        // Spawn Prophet near observatory (dist 1)
        let mut prophet_traits = Traits::default();
        prophet_traits.add(Trait::Prophet);
        let prophet = world.spawn((
            Pop,
            GridPosition { x: 6, y: 5 },
            prophet_traits,
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            },
        )).id();

        // Spawn non-prophet
        let non_prophet = world.spawn((
            Pop,
            GridPosition { x: 6, y: 5 },
            Traits::default(),
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            },
        )).id();

        // Spawn Prophet far from observatory
        let mut prophet_traits = Traits::default();
        prophet_traits.add(Trait::Prophet);
        let prophet_far = world.spawn((
            Pop,
            GridPosition { x: 10, y: 10 },
            prophet_traits,
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            },
        )).id();

        // Spawn Prophet at coordinate less than observatory to test abs_diff underflow protection
        let mut prophet_traits = Traits::default();
        prophet_traits.add(Trait::Prophet);
        let prophet_under = world.spawn((
            Pop,
            GridPosition { x: 4, y: 5 },
            prophet_traits,
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(observatory_epiphany_system);
        schedule.run(&mut world);

        let prophet_health = world.get::<Health>(prophet).unwrap();
        assert!(prophet_health.current < 100.0, "Prophet near should take damage");

        let non_prophet_health = world.get::<Health>(non_prophet).unwrap();
        assert_eq!(non_prophet_health.current, 100.0, "Non-prophet should not take damage");

        let prophet_far_health = world.get::<Health>(prophet_far).unwrap();
        assert_eq!(prophet_far_health.current, 100.0, "Prophet far should not take damage");

        let prophet_under_health = world.get::<Health>(prophet_under).unwrap();
        assert!(prophet_under_health.current < 100.0, "Prophet near (under) should take damage");

        let res = world.resource::<ColonyResources>();
        assert!(res.knowledge > 0.0, "Colony should gain knowledge");
    }
}
