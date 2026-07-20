use bevy::prelude::*;
use rand::Rng;

#[derive(Event)]
pub struct FtlJumpEvent {
    pub drive_quality: f32,
    pub target_system: Entity,
}

#[derive(Event)]
pub struct SubspaceWakeEvent {
    pub system: Entity,
    pub severity: f32,
}

pub fn generate_subspace_wake_system(
    mut jump_events: EventReader<FtlJumpEvent>,
    mut wake_events: EventWriter<SubspaceWakeEvent>,
) {
    let mut rng = rand::thread_rng();
    for jump in jump_events.read() {
        // Refactored from hardcoded 0.5 to a random roll based on quality
        let threshold = rng.gen_range(0.0..1.0);
        if threshold > jump.drive_quality {
            wake_events.send(SubspaceWakeEvent {
                system: jump.target_system,
                severity: 1.0 - jump.drive_quality,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(Events::<FtlJumpEvent>::default());
        world.insert_resource(Events::<SubspaceWakeEvent>::default());
        world
    }

    #[test]
    fn test_low_quality_drive_creates_wake() {
        let mut world = setup_world();
        let system_entity = world.spawn_empty().id();

        world
            .resource_mut::<Events<FtlJumpEvent>>()
            .send(FtlJumpEvent {
                drive_quality: 0.0, // Force a wake for testing
                target_system: system_entity,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(generate_subspace_wake_system);
        schedule.run(&mut world);

        let wake_events = world.resource::<Events<SubspaceWakeEvent>>();
        let mut reader = wake_events.get_cursor();
        let events: Vec<_> = reader.read(wake_events).collect();

        assert_eq!(events.len(), 1, "A wake event should be generated");
        assert!(
            events[0].severity > 0.0,
            "Wake should have positive severity"
        );
    }
}
