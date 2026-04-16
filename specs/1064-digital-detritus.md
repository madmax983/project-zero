# Digital Detritus

## 1. Overview
The galactic network is littered with the digital remains of fallen civilizations—spam, malware, and corrupted advertisements. When players engage in "Data Mining" to uncover lost technologies, they must filter through this Junk Data. Failing to do so can trigger System Crashes or unleash Viruses into the colony's infrastructure.

## 2. Dependencies
- Technology / Research System (Layer 3)
- Event / Chronicle System (`src/layer1/chronicle.rs`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[test]
    fn test_data_mining_success_yields_tech() {
        let mut app = App::new();
        app.init_resource::<DataMiningQueue>();
        app.init_resource::<DiscoveredTechs>();

        app.add_systems(Update, process_data_mining_system);

        // Queue a safe data mining operation
        app.world_mut().resource_mut::<DataMiningQueue>().add_job(MiningJob {
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
    fn test_data_mining_failure_triggers_virus() {
        let mut app = App::new();
        app.init_resource::<DataMiningQueue>();
        app.init_resource::<DiscoveredTechs>();
        app.add_event::<VirusEvent>();

        app.add_systems(Update, process_data_mining_system);

        // Queue a high-risk data mining operation that is rigged to fail in test
        app.world_mut().resource_mut::<DataMiningQueue>().add_job(MiningJob {
            risk_level: RiskLevel::Extreme, // Forces a failure in test environment
            data_volume: 1000,
        });

        app.update();

        // Should emit a virus event
        let events = app.world().resource::<Events<VirusEvent>>();
        let mut reader = events.get_reader();
        let virus_events: Vec<_> = reader.read(events).collect();

        assert_eq!(virus_events.len(), 1);
        assert_eq!(virus_events[0].severity, VirusSeverity::Critical);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

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
) {
    for job in queue.jobs.drain(..) {
        match job.risk_level {
            RiskLevel::Low => {
                techs.count += 1;
            }
            RiskLevel::Extreme => {
                virus_events.send(VirusEvent {
                    severity: VirusSeverity::Critical,
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- The `RiskLevel` needs to be replaced with a probabilistic system. "Extreme" risk shouldn't guarantee failure, just increase the chance.
- We need a `JunkDataFilter` resource or component that can mitigate risk based on the colony's processing power or specific filter technologies.
- The `VirusEvent` needs to be caught by a system that actually applies negative effects to the colony (e.g., locking airlocks, draining credits, shutting down factories).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] High-risk data mining has a chance to trigger a Virus Event.

## 7. Technical Guidance
- Integrate this with the existing Chronicle system so that virus outbreaks are recorded in the colony's history.
- The actual effects of the virus (e.g., locking airlocks) should be handled by a separate system that listens for `VirusEvent`s to maintain decoupling.

## 8. Questions
*Builder: add questions here if spec is unclear.*
