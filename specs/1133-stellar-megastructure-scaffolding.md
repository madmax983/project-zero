# Spec 1133: Stellar Megastructure Scaffolding

## 1. Overview
The "Stellar Megastructure Scaffolding" feature introduces the first stage of post-scarcity engineering at the systemic level (Layer 2 -> Layer 3). A civilization can begin construction on a massive infrastructure project designed to harvest a star's energy. This scaffolding requires an immense funneling of resources from Layer 1 planetary economies, creating severe local strain. As the scaffolding's mass grows, it begins to alter the host star's output, occasionally triggering solar anomalies that impact the system.

**Fantasy:** The awe and terror of beginning a project so massive it will take generations and redefine your civilization's place in the galaxy.
**Emergence:** You commit to the Scaffolding project. Decades later, your planetary economies are strained to the breaking point. The scaffolding is 40% complete. A neighboring Layer 3 empire notices your star dimming, realizes what you are building, and launches a preemptive war to stop you from achieving post-scarcity godhood.
**Tension:** How much of your present prosperity are you willing to sacrifice for a future you might not live to see, while painting a massive target on your civilization's back?

## 2. Dependencies
- Layer 2 System/Orbital construction logic (`layer2::construction` or similar)
- Layer 1 Economy resource sinking (`layer1::economy::ResourceSink`)
- Layer 3 Civilization threat/diplomacy systems (`layer3::diplomacy::ThreatMap`)

## 3. RED Phase: Tests First

```rust
// tests/layer2_megastructure_scaffolding_tests.rs
use bevy::prelude::*;
use crate::layer2::megastructure::{
    MegastructureScaffolding, MegastructureProgressEvent,
    process_megastructure_construction_system,
    trigger_solar_anomaly_system,
    SolarAnomalyEvent,
};
use crate::layer3::diplomacy::{ThreatMap, FactionId};

#[test]
fn test_megastructure_progress_increases_completion() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, process_megastructure_construction_system);
    app.add_event::<MegastructureProgressEvent>();

    let scaffolding_entity = app.world_mut().spawn(MegastructureScaffolding {
        completion_percent: 10.0,
        resources_invested: 1000,
    }).id();

    // Act
    app.world_mut()
        .resource_mut::<Events<MegastructureProgressEvent>>()
        .send(MegastructureProgressEvent {
            entity: scaffolding_entity,
            resources_added: 500,
            completion_increment: 2.5,
        });

    app.update();

    // Assert
    let scaffolding = app.world().entity(scaffolding_entity).get::<MegastructureScaffolding>().unwrap();
    assert_eq!(scaffolding.completion_percent, 12.5);
    assert_eq!(scaffolding.resources_invested, 1500);
}

#[test]
fn test_high_completion_triggers_solar_anomaly() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, trigger_solar_anomaly_system);
    app.add_event::<SolarAnomalyEvent>();

    // Spawn scaffolding that is sufficiently large to block/alter starlight
    app.world_mut().spawn(MegastructureScaffolding {
        completion_percent: 45.0,
        resources_invested: 50000,
    });

    // Act
    app.update();

    // Assert
    let anomaly_events = app.world().resource::<Events<SolarAnomalyEvent>>();
    // We expect some random chance or threshold to be met, simplified here for testing
    // In a real deterministic test, we'd mock the RNG or force the condition
    // For now, let's assume at >40% it triggers deterministically in the test setup
    let mut reader = anomaly_events.get_reader();
    let events: Vec<_> = reader.read(anomaly_events).collect();
    assert!(!events.is_empty(), "A solar anomaly should have been triggered by the massive scaffolding.");
}

#[test]
fn test_megastructure_generates_threat() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, crate::layer3::diplomacy::update_threat_from_megastructures_system);

    app.world_mut().spawn(MegastructureScaffolding {
        completion_percent: 20.0,
        resources_invested: 20000,
    });

    app.insert_resource(ThreatMap::default());

    // Act
    app.update();

    // Assert
    let threat_map = app.world().resource::<ThreatMap>();
    // Assume we check threat against neighboring factions
    // Megastructure construction should globally raise threat levels
    assert!(threat_map.global_threat_modifier > 0.0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer2/megastructure.rs
use bevy::prelude::*;
use crate::layer3::diplomacy::ThreatMap;

#[derive(Component, Debug, Default)]
pub struct MegastructureScaffolding {
    pub completion_percent: f32,
    pub resources_invested: u64,
}

#[derive(Event, Debug)]
pub struct MegastructureProgressEvent {
    pub entity: Entity,
    pub resources_added: u64,
    pub completion_increment: f32,
}

#[derive(Event, Debug)]
pub struct SolarAnomalyEvent {
    pub severity: f32,
    pub source_entity: Entity,
}

pub fn process_megastructure_construction_system(
    mut events: EventReader<MegastructureProgressEvent>,
    mut query: Query<&mut MegastructureScaffolding>,
) {
    for event in events.read() {
        if let Ok(mut scaffolding) = query.get_mut(event.entity) {
            scaffolding.completion_percent += event.completion_increment;
            scaffolding.resources_invested += event.resources_added;
        }
    }
}

pub fn trigger_solar_anomaly_system(
    query: Query<(Entity, &MegastructureScaffolding)>,
    mut events: EventWriter<SolarAnomalyEvent>,
) {
    for (entity, scaffolding) in query.iter() {
        // Simple threshold check for the test to pass
        if scaffolding.completion_percent > 40.0 {
            events.send(SolarAnomalyEvent {
                severity: scaffolding.completion_percent / 100.0,
                source_entity: entity,
            });
        }
    }
}

// In src/layer3/diplomacy.rs (or appropriately imported)
pub fn update_threat_from_megastructures_system(
    query: Query<&MegastructureScaffolding>,
    mut threat_map: ResMut<ThreatMap>,
) {
    let mut total_threat = 0.0;
    for scaffolding in query.iter() {
        total_threat += scaffolding.completion_percent * 0.5; // Threat scales with completion
    }
    threat_map.global_threat_modifier = total_threat;
}
```

## 5. REFACTOR Phase: Quality & Design
- **RNG for Anomalies:** Instead of a hard threshold triggering an anomaly every tick, implement a probabilistic check using `rand` or Bevy's random utilities. The probability should scale with `completion_percent`.
- **Resource Drain:** Connect `MegastructureProgressEvent` to the Layer 1 economy. Constructing the scaffolding should actively drain alloys, advanced materials, and labor from nearby planets, rather than just abstractly gaining progress.
- **Threat Granularity:** Instead of a `global_threat_modifier`, calculate threat per-faction based on their proximity to the star system and their ideology (e.g., Fallen Empires might be instantly enraged).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Scaffolding correctly tracks progress and invested resources.
- [ ] High completion percentages trigger `SolarAnomalyEvent`s.
- [ ] Existing scaffolding increases threat levels in `ThreatMap`.

## 7. Technical Guidance
- Create a new module `src/layer2/megastructure.rs` to house the core scaffolding logic.
- Ensure that the anomaly triggering system is run periodically (e.g., using a `Timer` or fixed timestep) rather than every frame to avoid spamming events.
- Integrate the threat modification logic into the existing Layer 3 diplomacy update loop so it correctly influences AI behavior.

## 8. Questions
*Builder: add questions here if spec is unclear.*
