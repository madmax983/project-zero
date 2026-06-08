// src/layer2/orbit/tether.rs
use bevy::prelude::*;
use crate::layer1::energy::PowerConsumer;

#[derive(Component)]
pub struct AsteroidTether {
    pub stability: f32,
}

#[derive(Event, Debug)]
pub struct AsteroidCrashEvent {
    pub tether_entity: Entity,
}

pub struct AsteroidTetheringPlugin;

impl Plugin for AsteroidTetheringPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AsteroidCrashEvent>()
           .add_systems(Update, tether_decay_system);
    }
}

fn tether_decay_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AsteroidTether, &PowerConsumer)>,
    mut crash_events: EventWriter<AsteroidCrashEvent>,
) {
    for (entity, mut tether, consumer) in query.iter_mut() {
        if !consumer.active {
            tether.stability -= 10.0;
            if tether.stability <= 0.0 {
                crash_events.send(AsteroidCrashEvent { tether_entity: entity });
                commands.entity(entity).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::energy::{PowerConsumer, PowerSource};

    #[test]
    fn test_tether_consumes_power_and_crashes_on_failure() {
        let mut app = App::new();
        app.add_plugins(AsteroidTetheringPlugin);

        // Note: For a complete integration test, the Energy plugin that toggles
        // consumer active states based on grid capacity would also be added here.
        // For this unit test, we manually simulate the grid failing the power by mutating it.
        app.add_event::<AsteroidCrashEvent>();

        // Spawn a power source that produces 10 power
        let _source = app.world_mut().spawn(PowerSource { output: 10.0, active: true }).id();

        // Spawn an asteroid tether demanding 20 power (more than produced)
        let tether = app.world_mut().spawn((
            AsteroidTether { stability: 100.0 },
            PowerConsumer { demand: 20.0, active: true }
        )).id();

        // Simulate tick where demand > supply
        if let Some(mut consumer) = app.world_mut().get_mut::<PowerConsumer>(tether) {
            consumer.active = false;
        }
        app.update();

        // Asteroid stability should decrease due to power failure
        let tether_state = app.world().get::<AsteroidTether>(tether).unwrap();
        assert!(tether_state.stability < 100.0, "Tether stability should decrease when unpowered.");

        // Simulate enough ticks for stability to reach 0
        for _ in 0..10 {
            app.update();
        }

        // Verify crash event was emitted
        let events = app.world().resource::<Events<AsteroidCrashEvent>>();
        let mut reader = events.get_cursor();
        assert!(reader.read(events).next().is_some(), "Crash event should be emitted when stability hits 0.");

        // Verify entity is destroyed
        assert!(app.world().get::<AsteroidTether>(tether).is_none(), "Tether entity should be destroyed after crashing.");
    }
}
