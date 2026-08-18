use crate::layer2::fleet::MovementSpeed;
use bevy::prelude::*;

#[derive(Component)]
pub struct Hyperlane {
    pub warp_wake_intensity: f32,
}

#[derive(Component)]
pub struct CurrentHyperlane {
    pub lane: Entity,
}

pub fn apply_warp_wake_system(
    mut fleet_query: Query<(&mut MovementSpeed, &CurrentHyperlane)>,
    lane_query: Query<&Hyperlane>,
) {
    for (mut speed, current_lane) in fleet_query.iter_mut() {
        if let Ok(lane) = lane_query.get(current_lane.lane) {
            let reduction_factor = 1.0 - (lane.warp_wake_intensity * 0.1).min(0.9);
            speed.current = speed.base * reduction_factor;
        }
    }
}

pub fn decay_warp_wake_system(mut lane_query: Query<&mut Hyperlane>) {
    for mut lane in lane_query.iter_mut() {
        if lane.warp_wake_intensity > 0.0 {
            lane.warp_wake_intensity = (lane.warp_wake_intensity - 0.1).max(0.0);
        }
    }
}


pub fn generate_warp_wake_system(
    mut events: EventReader<
        crate::layer2::navigation::chronological_stutter::HyperlaneTransitEvent,
    >,
    fleet_query: Query<&CurrentHyperlane>,
    mut lane_query: Query<&mut Hyperlane>,
) {
    for ev in events.read() {
        if let Ok(current_lane) = fleet_query.get(ev.fleet) {
            if let Ok(mut lane) = lane_query.get_mut(current_lane.lane) {
                lane.warp_wake_intensity += 1.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::fleet::{Fleet, MovementSpeed};
    use crate::shared::time::SimulationTime;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(SimulationTime {
            tick: 0,
            ..Default::default()
        });
        app.add_systems(Update, (apply_warp_wake_system, decay_warp_wake_system));
        app
    }

    #[test]
    fn test_warp_wake_slows_down_fleets() {
        let mut app = setup_app();

        // Setup hyperlane with wake
        let hyperlane = app
            .world_mut()
            .spawn(Hyperlane {
                warp_wake_intensity: 5.0, // High intensity wake
            })
            .id();

        // Spawn a fleet on the lane
        let fleet = app
            .world_mut()
            .spawn((
                Fleet,
                MovementSpeed {
                    base: 10.0,
                    current: 10.0,
                },
                CurrentHyperlane { lane: hyperlane },
            ))
            .id();

        app.update();

        let speed = app.world().get::<MovementSpeed>(fleet).unwrap();

        // Speed should be reduced by wake
        assert!(speed.current < speed.base);
    }

    #[test]
    fn test_warp_wake_decays_over_time() {
        let mut app = setup_app();

        // Setup hyperlane with wake
        let hyperlane = app
            .world_mut()
            .spawn(Hyperlane {
                warp_wake_intensity: 5.0,
            })
            .id();

        app.world_mut().resource_mut::<SimulationTime>().tick += 100;
        app.update();

        let lane = app.world().get::<Hyperlane>(hyperlane).unwrap();

        // Wake intensity should have decreased
        assert!(lane.warp_wake_intensity < 5.0);
    }
}
