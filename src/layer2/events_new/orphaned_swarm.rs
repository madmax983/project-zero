use crate::layer1::biology::health::Health;
use crate::layer1::economy::WorkEfficiency;
use crate::layer1::pop::Pop;
use crate::layer1::void_weed::{MerchantArrivalEvent, MerchantType};
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct SwarmArrivalEvent;

#[derive(Resource)]
pub struct SwarmCorruption {
    pub level: f32,
}

#[derive(Event)]
pub struct SwarmHostileEvent;

pub fn check_for_derelict_arrival(
    mut arrivals: EventReader<MerchantArrivalEvent>,
    mut trigger_ew: EventWriter<SwarmArrivalEvent>,
    mut commands: Commands,
) {
    for arrival in arrivals.read() {
        if arrival.merchant_type == MerchantType::Enigmatic {
            // Use an existing enum variant
            trigger_ew.send(SwarmArrivalEvent);
            commands.insert_resource(SwarmCorruption { level: 0.0 });
        }
    }
}

pub fn apply_swarm_efficiency_boost(
    mut arrivals: EventReader<SwarmArrivalEvent>,
    mut workers: Query<&mut WorkEfficiency>,
) {
    for _ in arrivals.read() {
        for mut efficiency in workers.iter_mut() {
            efficiency.multiplier += 0.5; // Massive boost
        }
    }
}

pub fn increase_swarm_corruption(mut corruption: Option<ResMut<SwarmCorruption>>) {
    if let Some(ref mut corr) = corruption {
        corr.level += 1.0; // Minimal implementation of increase over time
    }
}

pub fn trigger_swarm_hostility(
    mut commands: Commands,
    corruption: Option<Res<SwarmCorruption>>,
    mut hostile_ew: EventWriter<SwarmHostileEvent>,
) {
    if let Some(corr) = corruption {
        if corr.level >= 100.0 {
            hostile_ew.send(SwarmHostileEvent);
            commands.remove_resource::<SwarmCorruption>();
        }
    }
}

pub fn apply_swarm_damage(
    mut hostile_events: EventReader<SwarmHostileEvent>,
    mut pops: Query<&mut Health, With<Pop>>, // Target pops specifically
) {
    for _ in hostile_events.read() {
        for mut health in pops.iter_mut() {
            health.current -= 10.0; // Minimal damage implementation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_swarm_arrival_boosts_efficiency() {
        let mut app = App::new();
        app.add_event::<MerchantArrivalEvent>();
        app.add_event::<SwarmArrivalEvent>();
        app.add_systems(Update, check_for_derelict_arrival);
        app.add_systems(Update, apply_swarm_efficiency_boost);

        // Setup base worker
        let worker_entity = app
            .world_mut()
            .spawn(WorkEfficiency { multiplier: 1.0 })
            .id();

        // Act: Derelict arrives
        app.world_mut().send_event(MerchantArrivalEvent {
            merchant_type: MerchantType::Enigmatic,
        });
        app.update();
        app.update(); // Second update to process boosts

        // Assert: Efficiency increased
        let efficiency = app.world().get::<WorkEfficiency>(worker_entity).unwrap();
        assert!(
            efficiency.multiplier > 1.0,
            "Swarm should boost work efficiency upon arrival."
        );
    }

    #[test]
    fn test_swarm_corruption_increases_over_time() {
        let mut app = App::new();
        app.insert_resource(SwarmCorruption { level: 0.0 });
        app.add_systems(Update, increase_swarm_corruption);

        app.update();

        let corruption = app.world().resource::<SwarmCorruption>();
        assert!(
            corruption.level > 0.0,
            "Corruption level should increase over time."
        );
    }

    #[test]
    fn test_critical_corruption_triggers_hostility() {
        let mut app = App::new();
        app.insert_resource(SwarmCorruption { level: 100.0 }); // Critical level
        app.add_event::<SwarmHostileEvent>();
        app.add_systems(Update, trigger_swarm_hostility);

        app.update();

        let events = app.world().resource::<Events<SwarmHostileEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(
            reader.read(events).count(),
            1,
            "Critical corruption should trigger hostility."
        );
    }

    #[test]
    fn test_hostile_swarm_damages_pops() {
        let mut app = App::new();
        app.add_event::<SwarmHostileEvent>();
        app.add_systems(Update, apply_swarm_damage);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        app.world_mut().send_event(SwarmHostileEvent);
        app.update();

        let health = app.world().get::<Health>(pop_entity).unwrap();
        assert!(
            health.current < 100.0,
            "Hostile swarm should damage pop health."
        );
    }
}
