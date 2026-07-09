#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::map::GridPosition;
    use scale::layer1::ad_screen::{AdScreen, update_ad_screens_system};
    use scale::layer1::needs::Needs;
    use scale::layer1::pop::Pop;

    #[test]
    fn test_ad_screen_distance_overflow() {
        let mut world = World::new();

        world.spawn((
            GridPosition { x: 2_000_000_000, y: 0 },
            AdScreen {
                radius: 10.0,
                credits_per_pop: 1.0,
                need_decay_multiplier: 1.0,
                accumulated_credits: 0.0,
            },
        ));

        world.spawn((
            Pop,
            GridPosition { x: -2_000_000_000, y: 0 },
            Needs::default(),
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_ad_screens_system);

        // This should not panic anymore
        schedule.run(&mut world);
    }
}
