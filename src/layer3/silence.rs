use crate::layer1::energy::PowerSource;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct DetectionRisk {
    pub current_risk: f32,
    pub threshold: f32,
}

impl Default for DetectionRisk {
    fn default() -> Self {
        Self {
            current_risk: 0.0,
            threshold: 100.0,
        }
    }
}

#[derive(Event)]
pub struct HostileSpawnEvent {
    pub severity: u32,
}

pub fn update_detection_risk_system(
    mut risk: ResMut<DetectionRisk>,
    pops: Query<(), With<Pop>>,
    power_sources: Query<&PowerSource>,
) {
    let pop_count = pops.iter().count() as f32;
    // Only count active power sources
    let total_power: f32 = power_sources
        .iter()
        .filter(|p| p.active)
        .map(|p| p.output)
        .sum();

    // Formula: 0.1 per pop, 0.05 per power unit.
    risk.current_risk = (pop_count * 0.1) + (total_power * 0.05);
}

pub fn check_hostile_spawn_system(
    mut risk: ResMut<DetectionRisk>,
    mut spawn_events: EventWriter<HostileSpawnEvent>,
) {
    if risk.current_risk >= risk.threshold {
        spawn_events.send(HostileSpawnEvent {
            severity: (risk.current_risk / 100.0) as u32,
        });

        // Increase threshold for next wave to create escalating tension
        risk.threshold *= 1.5;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::energy::PowerSource;
    use crate::layer1::pop::Pop;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_detection_risk_increases_with_power_and_pops() {
        let mut world = World::new();
        world.insert_resource(DetectionRisk {
            current_risk: 0.0,
            threshold: 100.0,
        });

        // Spawn 10 Pops
        for _ in 0..10 {
            world.spawn(Pop);
        }
        // Spawn Power Generators
        world.spawn(PowerSource {
            output: 50.0,
            active: true,
        });
        world.spawn(PowerSource {
            output: 30.0,
            active: true,
        });
        // Inactive source shouldn't count
        world.spawn(PowerSource {
            output: 100.0,
            active: false,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_detection_risk_system);
        schedule.run(&mut world);

        let risk = world.resource::<DetectionRisk>();
        // e.g. risk = (10 pops * 0.1) + (80 power * 0.05) = 1.0 + 4.0 = 5.0
        assert_eq!(risk.current_risk, 5.0);
    }

    #[test]
    fn test_hostile_spawn_triggered_when_threshold_exceeded() {
        let mut world = World::new();
        world.insert_resource(DetectionRisk {
            current_risk: 105.0,
            threshold: 100.0,
        });
        world.insert_resource(Events::<HostileSpawnEvent>::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(check_hostile_spawn_system);
        schedule.run(&mut world);

        let events = world.resource::<Events<HostileSpawnEvent>>();
        let mut reader = events.get_cursor();
        assert!(
            reader.read(events).next().is_some(),
            "HostileSpawnEvent should have been emitted."
        );

        // Ensure risk resets or threshold increases
        let risk = world.resource::<DetectionRisk>();
        assert!(
            risk.threshold > 100.0,
            "Threshold should increase after a spawn."
        );
    }
}
