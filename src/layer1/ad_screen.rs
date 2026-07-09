//! The `AdScreen` module manages corporate advertising entities that extract credits from nearby `Pop`s.
//!
//! # The Story
//! In the hyper-capitalist frontier, even leisure time is a commodity. `AdScreen` structures emit
//! subliminal and overt advertisements, extracting value (`accumulated_credits`) from any `Pop` that
//! wanders too close. However, this corporate intrusion accelerates the decay of the `Pop`'s `leisure`
//! needs, forcing them to seek entertainment more frequently.
//!
//! # Usage
//! Attach an `AdScreen` component to any entity with a `GridPosition`. Pops within the `radius` will
//! automatically be affected when `update_ad_screens_system` runs.
//!

use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

/// A structure representing an active advertising screen.
///
/// When a `Pop` enters the `radius` of this screen:
/// 1. The screen generates `credits_per_pop`.
/// 2. The `Pop`'s `leisure` need decays faster based on the `need_decay_multiplier`.
#[derive(Component)]
pub struct AdScreen {
    /// The radius of effect in which Pops are affected.
    pub radius: f32,
    /// Credits generated per Pop within the radius.
    pub credits_per_pop: f32,
    /// Multiplier for the Pop's leisure need decay rate.
    pub need_decay_multiplier: f32,
    /// Total credits extracted by this screen.
    pub accumulated_credits: f32,
}

/// Processes all active `AdScreen` entities and applies their effects to nearby `Pop`s.
///
/// For every `Pop` within the `radius` of an `AdScreen`, this system:
/// 1. Adds `credits_per_pop` to the screen's `accumulated_credits`.
/// 2. Accelerates the decay of the `Pop`'s `leisure` need.
///
/// # Examples
///
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::ad_screen::{AdScreen, update_ad_screens_system};
/// use scale::layer1::map::GridPosition;
/// use scale::layer1::needs::Needs;
/// use scale::layer1::pop::Pop;
///
/// let mut world = World::new();
///
/// // Spawn a Pop with high leisure
/// let pop_id = world.spawn((
///     Pop,
///     GridPosition { x: 0, y: 0 },
///     Needs { leisure: 1.0, ..Default::default() },
/// )).id();
///
/// // Spawn an AdScreen nearby
/// let screen_id = world.spawn((
///     GridPosition { x: 1, y: 0 },
///     AdScreen {
///         radius: 5.0,
///         credits_per_pop: 10.0,
///         need_decay_multiplier: 2.0, // Doubles the standard 0.0015 decay
///         accumulated_credits: 0.0,
///     },
/// )).id();
///
/// // Run the system
/// let mut schedule = Schedule::default();
/// schedule.add_systems(update_ad_screens_system);
/// schedule.run(&mut world);
///
/// // The screen has generated credits
/// let screen = world.get::<AdScreen>(screen_id).unwrap();
/// assert_eq!(screen.accumulated_credits, 10.0);
///
/// // The Pop's leisure has decayed faster than normal
/// let needs = world.get::<Needs>(pop_id).unwrap();
/// assert!(needs.leisure < 1.0);
/// ```
pub fn update_ad_screens_system(
    mut screens: Query<(&GridPosition, &mut AdScreen)>,
    mut pops: Query<(&GridPosition, &mut Needs), With<Pop>>,
) {
    for (screen_pos, mut screen) in &mut screens {
        let radius_sq = screen.radius * screen.radius;

        for (pop_pos, mut needs) in &mut pops {
            let dist_sq = (screen_pos.x as f32 - pop_pos.x as f32).powi(2) + (screen_pos.y as f32 - pop_pos.y as f32).powi(2);
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
