use crate::layer1::biology::health::Health;
use crate::layer1::morale::Morale;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct DerelictArrivalEvent {
    pub faction: String,
}

#[derive(Event)]
pub struct SwarmArrivalEvent;

#[derive(Resource)]
pub struct SwarmEfficiencyBoost {
    pub multiplier: f32,
}

#[derive(Resource)]
pub struct SwarmCorruption {
    pub level: f32,
}

#[derive(Event)]
pub struct SwarmHostileEvent;

pub fn check_for_derelict_arrival(
    mut arrivals: EventReader<DerelictArrivalEvent>,
    mut trigger_ew: EventWriter<SwarmArrivalEvent>,
    mut commands: Commands,
) {
    for arrival in arrivals.read() {
        if arrival.faction == "DerelictSwarm" {
            trigger_ew.send(SwarmArrivalEvent);
            commands.insert_resource(SwarmCorruption { level: 0.0 });
        }
    }
}

pub fn apply_swarm_efficiency_boost(
    mut arrivals: EventReader<SwarmArrivalEvent>,
    mut commands: Commands,
) {
    for _ in arrivals.read() {
        commands.insert_resource(SwarmEfficiencyBoost { multiplier: 1.5 });
    }
}

pub fn increase_swarm_corruption(
    corruption: Option<ResMut<SwarmCorruption>>,
    time: Option<Res<SimulationTime>>,
) {
    if let (Some(mut corr), Some(_sim_time)) = (corruption, time) {
        corr.level += 1.0; // Per tick
    }
}

pub fn trigger_swarm_hostility(
    mut corruption: Option<ResMut<SwarmCorruption>>,
    mut hostile_ew: EventWriter<SwarmHostileEvent>,
    morales: Query<&Morale>,
) {
    if let Some(ref mut corr) = corruption {
        if corr.level >= 100.0 {
            // Check for low morale (as a proxy for worker strike triggers)
            let mut has_low_morale = false;
            for morale in morales.iter() {
                if morale.value < 0.2 {
                    has_low_morale = true;
                    break;
                }
            }

            if has_low_morale {
                hostile_ew.send(SwarmHostileEvent);
                corr.level = 0.0; // Reset corruption to avoid triggering infinitely
            }
        }
    }
}

pub fn apply_swarm_damage(
    mut hostile_events: EventReader<SwarmHostileEvent>,
    mut pops: Query<&mut Health>,
) {
    for _ in hostile_events.read() {
        for mut health in pops.iter_mut() {
            health.take_damage(10.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::biology::health::Health;
    use crate::layer1::morale::Morale;

    #[test]
    fn test_swarm_arrival_boosts_efficiency() {
        let mut app = bevy_app::App::new();
        app.add_event::<DerelictArrivalEvent>()
            .add_event::<SwarmArrivalEvent>()
            .add_event::<SwarmHostileEvent>()
            .add_systems(
                bevy_app::Update,
                (
                    check_for_derelict_arrival,
                    apply_swarm_efficiency_boost,
                    increase_swarm_corruption,
                    trigger_swarm_hostility,
                    apply_swarm_damage,
                ),
            );

        app.insert_resource(SwarmEfficiencyBoost { multiplier: 1.0 });

        // Act: Derelict arrives
        app.world_mut().send_event(DerelictArrivalEvent {
            faction: "DerelictSwarm".to_string(),
        });
        app.update();
        app.update(); // Second update to process boosts

        // Assert: Efficiency increased
        let efficiency = app.world().resource::<SwarmEfficiencyBoost>();
        assert!(
            efficiency.multiplier > 1.0,
            "Swarm should boost work efficiency upon arrival."
        );
    }

    #[test]
    fn test_swarm_corruption_increases_over_time() {
        let mut app = bevy_app::App::new();
        app.add_event::<DerelictArrivalEvent>()
            .add_event::<SwarmArrivalEvent>()
            .add_event::<SwarmHostileEvent>()
            .add_systems(
                bevy_app::Update,
                (
                    check_for_derelict_arrival,
                    apply_swarm_efficiency_boost,
                    increase_swarm_corruption,
                    trigger_swarm_hostility,
                    apply_swarm_damage,
                ),
            );
        app.insert_resource(SwarmCorruption { level: 0.0 });
        app.insert_resource(crate::shared::time::SimulationTime {
            tick: 0,
            speed: crate::shared::time::SimSpeed::Normal,
        });

        app.update();

        let corruption = app.world().resource::<SwarmCorruption>();
        assert!(
            corruption.level > 0.0,
            "Corruption level should increase over time."
        );
    }

    #[test]
    fn test_critical_corruption_triggers_hostility() {
        let mut app = bevy_app::App::new();
        app.add_event::<DerelictArrivalEvent>()
            .add_event::<SwarmArrivalEvent>()
            .add_event::<SwarmHostileEvent>()
            .add_systems(
                bevy_app::Update,
                (
                    check_for_derelict_arrival,
                    apply_swarm_efficiency_boost,
                    increase_swarm_corruption,
                    trigger_swarm_hostility,
                    apply_swarm_damage,
                ),
            );
        app.insert_resource(SwarmCorruption { level: 100.0 }); // Critical level

        app.world_mut().spawn(Morale {
            value: 0.1,
            modifiers: vec![],
        }); // Low morale triggers it

        app.update();

        let events = app.world().resource::<Events<SwarmHostileEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(
            reader.read(events).len(),
            1,
            "Critical corruption should trigger hostility."
        );
    }

    #[test]
    fn test_critical_corruption_does_not_trigger_hostility_if_high_morale() {
        let mut app = bevy_app::App::new();
        app.add_event::<DerelictArrivalEvent>()
            .add_event::<SwarmArrivalEvent>()
            .add_event::<SwarmHostileEvent>()
            .add_systems(
                bevy_app::Update,
                (
                    check_for_derelict_arrival,
                    apply_swarm_efficiency_boost,
                    increase_swarm_corruption,
                    trigger_swarm_hostility,
                    apply_swarm_damage,
                ),
            );
        app.insert_resource(SwarmCorruption { level: 100.0 }); // Critical level

        app.world_mut().spawn(Morale {
            value: 0.9,
            modifiers: vec![],
        }); // High morale prevents it

        app.update();

        let events = app.world().resource::<Events<SwarmHostileEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(
            reader.read(events).len(),
            0,
            "Critical corruption should NOT trigger hostility without low morale."
        );
    }

    #[test]
    fn test_hostile_swarm_damages_pops() {
        let mut app = bevy_app::App::new();
        app.add_event::<DerelictArrivalEvent>()
            .add_event::<SwarmArrivalEvent>()
            .add_event::<SwarmHostileEvent>()
            .add_systems(
                bevy_app::Update,
                (
                    check_for_derelict_arrival,
                    apply_swarm_efficiency_boost,
                    increase_swarm_corruption,
                    trigger_swarm_hostility,
                    apply_swarm_damage,
                ),
            );

        let pop_entity = app
            .world_mut()
            .spawn(Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            })
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
