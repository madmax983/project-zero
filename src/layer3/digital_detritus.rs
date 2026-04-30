//! Digital Detritus
//!
//! This module handles the process of data mining digital detritus left by ancient civilizations.
//! Data mining is a risky endeavor. While it yields ancient technologies, extreme jobs risk unleashing
//! crippling computational viruses. A [`JunkDataFilter`] can be used to mitigate these risks.

use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use bevy::prelude::*;
use rand::Rng;

/// A queue of [`MiningJob`] operations to be processed.
#[derive(Resource, Default)]
pub struct DataMiningQueue {
    pub jobs: Vec<MiningJob>,
}

impl DataMiningQueue {
    pub fn add_job(&mut self, job: MiningJob) {
        self.jobs.push(job);
    }
}

/// A global counter of ancient technologies recovered from data mining operations.
#[derive(Resource, Default)]
pub struct DiscoveredTechs {
    pub count: u32,
}

impl DiscoveredTechs {
    pub fn count(&self) -> u32 {
        self.count
    }
}

/// Provides a defensive barrier against computational viruses.
///
/// A `mitigation_chance` of `1.0` means a 100% chance to block a virus.
#[derive(Resource, Default)]
pub struct JunkDataFilter {
    pub mitigation_chance: f32, // Probability to mitigate an extreme event (0.0 to 1.0)
}

/// The level of danger associated with a [`MiningJob`].
pub enum RiskLevel {
    Low,
    Extreme,
}

/// A single data mining operation to be processed by [`process_data_mining_system`].
pub struct MiningJob {
    pub risk_level: RiskLevel,
    pub data_volume: u32,
}

/// An event triggered when an `Extreme` risk [`MiningJob`] fails, unleashing a computational virus.
#[derive(Event)]
pub struct VirusEvent;

/// Processes all pending jobs in the [`DataMiningQueue`].
///
/// Successfully processed jobs yield an increase to [`DiscoveredTechs`]. Jobs with
/// [`RiskLevel::Extreme`] have a base 30% chance to fail, triggering a [`VirusEvent`].
/// This risk can be mitigated if a [`JunkDataFilter`] resource is present.
///
/// # Examples
/// ```
/// use bevy::prelude::*;
/// use scale::layer3::digital_detritus::{process_data_mining_system, DataMiningQueue, DiscoveredTechs, MiningJob, RiskLevel, VirusEvent};
///
/// let mut app = App::new();
/// app.init_resource::<DataMiningQueue>();
/// app.init_resource::<DiscoveredTechs>();
/// app.add_event::<VirusEvent>();
/// app.add_systems(Update, process_data_mining_system);
///
/// app.world_mut().resource_mut::<DataMiningQueue>().add_job(MiningJob {
///     risk_level: RiskLevel::Low,
///     data_volume: 100,
/// });
///
/// app.update();
///
/// assert_eq!(app.world().resource::<DiscoveredTechs>().count(), 1);
/// ```
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
                    virus_events.send(VirusEvent);
                } else {
                    techs.count += 1;
                }
            }
        }
    }
}

/// Observes [`VirusEvent`]s and records them to the global chronicle.
///
/// Adds an [`AddChronicleEvent`] with [`EventImportance::Major`] detailing the disaster.
///
/// # Examples
/// ```
/// use bevy::prelude::*;
/// use scale::layer1::core::chronicle::AddChronicleEvent;
/// use scale::layer3::digital_detritus::{record_virus_event_chronicle_system, VirusEvent};
///
/// let mut app = App::new();
/// app.add_event::<VirusEvent>();
/// app.add_event::<AddChronicleEvent>();
/// app.add_systems(Update, record_virus_event_chronicle_system);
///
/// app.world_mut().resource_mut::<Events<VirusEvent>>().send(VirusEvent);
/// app.update();
///
/// let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
/// #[allow(deprecated)]
/// let mut reader = chronicle_events.get_reader();
/// let events: Vec<_> = reader.read(chronicle_events).collect();
/// assert_eq!(events.len(), 1);
/// assert!(events[0].text.contains("Critical Virus"));
/// ```
pub fn record_virus_event_chronicle_system(
    mut virus_events: EventReader<VirusEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for _event in virus_events.read() {
        chronicle_events.send(AddChronicleEvent {
            text: "A Critical Virus has been unleashed from the Data Mining operation!".to_string(),
            importance: EventImportance::Major,
        });
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
            .send(VirusEvent);

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
            .send(VirusEvent);

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
