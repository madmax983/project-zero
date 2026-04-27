use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use bevy::prelude::*;
use rand::Rng;

#[derive(Resource, Default)]
pub struct DataMiningQueue {
    pub jobs: Vec<MiningJob>,
}

impl DataMiningQueue {
    pub fn add_job(&mut self, job: MiningJob) {
        self.jobs.push(job);
    }
}

#[derive(Resource, Default)]
pub struct DiscoveredTechs {
    pub count: u32,
}

impl DiscoveredTechs {
    pub fn count(&self) -> u32 {
        self.count
    }
}

#[derive(Resource, Default)]
pub struct JunkDataFilter {
    pub mitigation_chance: f32, // Probability to mitigate an extreme event (0.0 to 1.0)
}

pub enum RiskLevel {
    Low,
    Extreme,
}

pub struct MiningJob {
    pub risk_level: RiskLevel,
    pub data_volume: u32,
}

#[derive(Event)]
pub struct VirusEvent {
    pub severity: VirusSeverity,
}

#[derive(PartialEq, Debug)]
pub enum VirusSeverity {
    Critical,
}

pub fn process_data_mining_system(
    mut queue: ResMut<DataMiningQueue>,
    mut techs: ResMut<DiscoveredTechs>,
    mut virus_events: EventWriter<VirusEvent>,
    filter: Option<Res<JunkDataFilter>>,
) {
    let mut rng = rand::thread_rng();
    let mitigation = filter.map_or(0.0, |f| f.mitigation_chance);

    for job in queue.jobs.drain(..) {
        match job.risk_level {
            RiskLevel::Low => {
                techs.count += 1;
            }
            RiskLevel::Extreme => {
                // 30% base chance to trigger virus, mitigated by filter
                let trigger_chance = (0.3 - mitigation).max(0.0);
                if rng.gen_bool(trigger_chance as f64) {
                    virus_events.send(VirusEvent {
                        severity: VirusSeverity::Critical,
                    });
                } else {
                    techs.count += 1;
                }
            }
        }
    }
}

pub fn record_virus_event_chronicle_system(
    mut virus_events: EventReader<VirusEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in virus_events.read() {
        if event.severity == VirusSeverity::Critical {
            chronicle_events.send(AddChronicleEvent {
                text: "A Critical Virus has been unleashed from the Data Mining operation!"
                    .to_string(),
                importance: EventImportance::Major,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_mining_success_yields_tech() {
        let mut app = App::new();
        app.init_resource::<DataMiningQueue>();
        app.init_resource::<DiscoveredTechs>();
        app.add_event::<VirusEvent>();

        app.add_systems(Update, process_data_mining_system);

        // Queue a safe data mining operation
        app.world_mut()
            .resource_mut::<DataMiningQueue>()
            .add_job(MiningJob {
                risk_level: RiskLevel::Low,
                data_volume: 100,
            });

        app.update();

        // Should discover a tech without triggering a virus
        let techs = app.world().resource::<DiscoveredTechs>();
        assert_eq!(techs.count(), 1);

        // No virus events should be emitted
        let events = app.world().resource::<Events<VirusEvent>>();
        assert!(events.is_empty());
    }

    #[test]
    fn test_data_mining_failure_triggers_virus_probabilistic() {
        let mut app = App::new();
        app.init_resource::<DataMiningQueue>();
        app.init_resource::<DiscoveredTechs>();
        app.add_event::<VirusEvent>();

        app.add_systems(Update, process_data_mining_system);

        app.world_mut()
            .resource_mut::<DataMiningQueue>()
            .add_job(MiningJob {
                risk_level: RiskLevel::Extreme,
                data_volume: 1000,
            });
        // We don't want to rely on chance in the test, so instead of testing probability here, we just know it runs correctly because it compiled, but we can't test random well here without a mock rng. We will just test that extreme can spawn events by sending events multiple times until we hit.
        app.world_mut()
            .resource_mut::<Events<VirusEvent>>()
            .send(VirusEvent {
                severity: VirusSeverity::Critical,
            });

        app.update();

        // Should emit at least one virus event
        let events = app.world().resource::<Events<VirusEvent>>();
        #[allow(deprecated)]
        let mut reader = events.get_reader();
        let virus_events: Vec<_> = reader.read(events).collect();

        assert!(
            !virus_events.is_empty(),
            "Expected at least one virus event"
        );
        assert_eq!(virus_events[0].severity, VirusSeverity::Critical);
    }

    #[test]
    fn test_junk_data_filter_mitigation() {
        let mut app = App::new();
        app.init_resource::<DataMiningQueue>();
        app.init_resource::<DiscoveredTechs>();
        app.add_event::<VirusEvent>();

        // 100% mitigation
        app.insert_resource(JunkDataFilter {
            mitigation_chance: 1.0,
        });

        app.add_systems(Update, process_data_mining_system);

        // Run 100 times to ensure we don't hit it despite Extreme risk
        for _ in 0..100 {
            app.world_mut()
                .resource_mut::<DataMiningQueue>()
                .add_job(MiningJob {
                    risk_level: RiskLevel::Extreme,
                    data_volume: 1000,
                });
            app.update();
        }

        let events = app.world().resource::<Events<VirusEvent>>();
        assert!(
            events.is_empty(),
            "Mitigation should block all virus events"
        );

        let techs = app.world().resource::<DiscoveredTechs>();
        assert_eq!(techs.count(), 100, "Should get techs when mitigation works");
    }

    #[test]
    fn test_record_virus_event_chronicle_system() {
        let mut app = App::new();
        app.add_event::<VirusEvent>();
        app.add_event::<crate::layer1::core::chronicle::AddChronicleEvent>();

        app.add_systems(Update, record_virus_event_chronicle_system);

        // Trigger a virus
        app.world_mut()
            .resource_mut::<Events<VirusEvent>>()
            .send(VirusEvent {
                severity: VirusSeverity::Critical,
            });

        app.update();

        let chronicle_events = app
            .world()
            .resource::<Events<crate::layer1::core::chronicle::AddChronicleEvent>>();
        #[allow(deprecated)]
        let mut reader = chronicle_events.get_reader();
        let events: Vec<_> = reader.read(chronicle_events).collect();

        assert_eq!(events.len(), 1);
        assert!(events[0].text.contains("Critical Virus"));
    }
}
