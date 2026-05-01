#[cfg(test)]
mod integration_tests {
    use bevy::prelude::*;
    use scale::layer1::actions::AssignedTo;
    use scale::layer1::core::integration::trigger_shift_end_system;
    use scale::layer1::day_night::{DayNightCycle, TimeOfDay};
    use scale::layer1::mind::utility_types::AssignmentType;
    use scale::layer1::pop::Pop;
    use scale::layer1::social::pop_relationships::{
        update_workplace_relationships_system, ShiftEndEvent,
    };
    use scale::layer1::social::AffinityChange;

    #[test]
    fn test_day_night_cycle_triggers_shift_end_event() {
        let mut app = App::new();
        app.add_event::<ShiftEndEvent>();
        app.add_event::<AffinityChange>();
        app.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Day,
            ticks_per_day: 100,
            day_count: 0,
        });

        app.add_systems(
            Update,
            (
                trigger_shift_end_system,
                update_workplace_relationships_system,
            )
                .chain(),
        );

        app.world_mut().spawn((
            Pop,
            AssignedTo {
                entity: Entity::from_raw(1),
                assignment_type: AssignmentType::FarmWorker,
            },
        ));

        app.world_mut().spawn((
            Pop,
            AssignedTo {
                entity: Entity::from_raw(1),
                assignment_type: AssignmentType::FarmWorker,
            },
        ));

        // First update initializes Local variables
        app.update();

        // Change cycle to Dusk
        app.world_mut().resource_mut::<DayNightCycle>().time_of_day = TimeOfDay::Dusk;

        // Second update triggers the shift end
        app.update();

        let events = app.world().resource::<Events<AffinityChange>>();
        let mut reader = events.get_cursor();
        let emitted: Vec<_> = reader.read(events).collect();

        assert_eq!(
            emitted.len(),
            2,
            "ShiftEndEvent should have triggered relationships updates"
        );
    }
}
