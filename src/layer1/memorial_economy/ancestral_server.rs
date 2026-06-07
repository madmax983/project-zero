use bevy_ecs::prelude::*;

use crate::layer1::energy::PowerConsumer;
use crate::layer1::entities::pop::PopDied;

#[derive(Component)]
pub struct AncestralServer {
    pub stored_engrams: u32,
}

pub fn handle_engram_upload_system(
    mut events: EventReader<PopDied>,
    mut servers: Query<&mut AncestralServer>,
) {
    let mut server_opt = servers.iter_mut().next();

    if let Some(ref mut server) = server_opt {
        for _event in events.read() {
            server.stored_engrams += 1;
        }
    } else {
        // If no server, we still need to consume the events so they don't leak
        for _ in events.read() {}
    }
}

pub fn scale_server_power_demand_system(
    mut servers: Query<(&AncestralServer, &mut PowerConsumer)>,
) {
    for (server, mut consumer) in servers.iter_mut() {
        // Base demand is 10, each engram adds 0.1, scaling quadratically for "exponential" feel
        let engrams_f32 = server.stored_engrams as f32;
        consumer.demand = 10.0 + (engrams_f32 * 0.1) + (engrams_f32 * engrams_f32 * 0.001);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::entities::pop::{Pop, PopDied};
    use crate::layer1::energy::PowerConsumer;

    // RED Phase Test Setup
    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.add_systems(Update, (
            handle_engram_upload_system,
            scale_server_power_demand_system,
        ));
        app
    }

    #[test]
    fn test_pop_death_uploads_engram() {
        let mut app = setup_app();

        let server = app.world_mut().spawn(AncestralServer { stored_engrams: 0 }).id();
        let pop = app.world_mut().spawn(Pop).id();

        app.world_mut().send_event(PopDied {
            entity: pop,
            name: "Test Pop".to_string(),
            tick: 1,
            reason: "Old Age".to_string(),
        });
        app.update();

        let server_data = app.world().get::<AncestralServer>(server).unwrap();
        assert_eq!(server_data.stored_engrams, 1, "Pop death should increase stored engrams on the server");
    }

    #[test]
    fn test_engram_count_increases_power_demand() {
        let mut app = setup_app();

        let server = app.world_mut().spawn((
            AncestralServer { stored_engrams: 100 },
            PowerConsumer { demand: 10.0, active: true },
        )).id();

        app.update();

        let consumer = app.world().get::<PowerConsumer>(server).unwrap();
        assert!(consumer.demand > 10.0, "High engram count should scale the power demand exponentially");
    }
}
