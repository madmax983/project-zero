use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct AdScreen {
    pub radius: f32,
    pub credits_per_pop: f32,
    pub need_decay_multiplier: f32,
    pub accumulated_credits: f32,
}

pub fn update_ad_screens_system(
    mut screens: Query<(&GridPosition, &mut AdScreen)>,
    mut pops: Query<(&GridPosition, &mut Needs), With<Pop>>,
) {
    for (screen_pos, mut screen) in &mut screens {
        let radius_sq = (screen.radius * screen.radius) as i32;

        for (pop_pos, mut needs) in &mut pops {
            let dist_sq = (screen_pos.x - pop_pos.x).pow(2) + (screen_pos.y - pop_pos.y).pow(2);
            if dist_sq <= radius_sq {
                // Generate credits
                screen.accumulated_credits += screen.credits_per_pop;

                // Inflate need decay
                // In this codebase, LEISURE_DECAY_PER_TICK is 0.0015
                let base_leisure_decay = 0.0015;
                let extra_leisure_decay = base_leisure_decay * (screen.need_decay_multiplier - 1.0);

                needs.leisure = (needs.leisure - extra_leisure_decay).max(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ad_screen_generates_credits_and_inflates_needs() {
        let mut world = World::new();

        let pop_id = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    leisure: 1.0,
                    ..Default::default()
                },
            ))
            .id();

        let screen_id = world
            .spawn((
                GridPosition { x: 1, y: 0 },
                AdScreen {
                    radius: 2.0,
                    credits_per_pop: 0.5,
                    need_decay_multiplier: 2.0,
                    accumulated_credits: 0.0,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_ad_screens_system);
        schedule.run(&mut world);

        let screen = world.get::<AdScreen>(screen_id).unwrap();
        assert_eq!(screen.accumulated_credits, 0.5);

        let needs = world.get::<Needs>(pop_id).unwrap();
        // Assume base decay is 0.0015. Multiplier 2.0 -> extra decay 0.0015
        assert_eq!(needs.leisure, 1.0 - 0.0015);
    }

    #[test]
    fn test_ad_screen_ignores_pops_out_of_range() {
        let mut world = World::new();

        let pop_id = world
            .spawn((
                Pop,
                GridPosition { x: 10, y: 0 }, // Out of range
                Needs {
                    leisure: 1.0,
                    ..Default::default()
                },
            ))
            .id();

        let screen_id = world
            .spawn((
                GridPosition { x: 0, y: 0 },
                AdScreen {
                    radius: 2.0,
                    credits_per_pop: 0.5,
                    need_decay_multiplier: 2.0,
                    accumulated_credits: 0.0,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_ad_screens_system);
        schedule.run(&mut world);

        let screen = world.get::<AdScreen>(screen_id).unwrap();
        assert_eq!(screen.accumulated_credits, 0.0); // No credits generated

        let needs = world.get::<Needs>(pop_id).unwrap();
        // Only base decay applies, which isn't applied in this system
        assert_eq!(needs.leisure, 1.0);
    }
}
