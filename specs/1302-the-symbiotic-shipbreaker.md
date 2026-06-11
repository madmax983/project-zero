# Specification: The Symbiotic Shipbreaker

## 1. Overview
When massive capital ships are destroyed or abandoned in orbit, they form a symbiotic ecosystem over time. Strange zero-G flora and feral maintenance drones create a hostile biome defending the derelict. Players can send specialized "Shipbreaker" Pops to salvage valuable core components, treating the derelict as a localized warzone rather than standard debris.

## 2. Dependencies
- Layer 2 Fleet & Combat (`Fleet`, `CombatEvent`)
- Layer 2 Nodes (`DerelictShip`, `OrbitalNode`)
- Layer 1 Economy (`ColonyResources`, `ItemType::StellarAlloy`)
- Layer 1 Pops (`Pop`, `ShipbreakerTrait`)
- Narrative/Chronicle (`AddChronicleEvent`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::nodes::{OrbitalNode, NodeClass};
    use crate::layer2::fleet::{Fleet, FleetState};
    use crate::layer1::economy::resources::ColonyResources;
    use crate::shared::chronicle::AddChronicleEvent;

    // Components to test
    #[derive(Component)]
    pub struct DerelictEcosystem {
        pub threat_level: f32,
        pub salvage_yield: f32,
    }

    #[derive(Component)]
    pub struct ShipbreakerMission {
        pub target_derelict: Entity,
        pub progress: f32,
    }

    fn simulate_shipbreaker_salvage(
        mut commands: Commands,
        mut q_missions: Query<(Entity, &mut ShipbreakerMission)>,
        mut q_derelicts: Query<&mut DerelictEcosystem>,
        mut resources: ResMut<ColonyResources>,
        mut evt_chronicle: EventWriter<AddChronicleEvent>,
    ) {
        // TDD minimal implementation will go here
    }

    #[test]
    fn test_shipbreaker_salvage_progress_reduces_threat_and_yields_salvage() {
        let mut app = App::new();
        app.add_event::<AddChronicleEvent>();
        app.insert_resource(ColonyResources::default());
        app.add_systems(Update, simulate_shipbreaker_salvage);

        let derelict = app.world_mut().spawn(DerelictEcosystem {
            threat_level: 100.0,
            salvage_yield: 50.0,
        }).id();

        let mission = app.world_mut().spawn(ShipbreakerMission {
            target_derelict: derelict,
            progress: 0.0,
        }).id();

        app.update();

        // After one tick of progress, the threat should decrease and salvage be recovered
        let ecosystem = app.world().get::<DerelictEcosystem>(derelict).unwrap();
        assert!(ecosystem.threat_level < 100.0, "Ecosystem threat should decrease as shipbreakers make progress");

        let resources = app.world().resource::<ColonyResources>();
        assert!(resources.metal > 0.0, "Salvage mission should yield metal resources");

        let mission_data = app.world().get::<ShipbreakerMission>(mission).unwrap();
        assert!(mission_data.progress > 0.0, "Mission progress should increase");
    }

    #[test]
    fn test_shipbreaker_salvage_complete_fires_chronicle() {
        let mut app = App::new();
        app.add_event::<AddChronicleEvent>();
        app.insert_resource(ColonyResources::default());
        app.add_systems(Update, simulate_shipbreaker_salvage);

        // Spawn derelict nearly destroyed
        let derelict = app.world_mut().spawn(DerelictEcosystem {
            threat_level: 5.0, // Almost zero
            salvage_yield: 10.0,
        }).id();

        app.world_mut().spawn(ShipbreakerMission {
            target_derelict: derelict,
            progress: 95.0, // Nearly complete
        });

        app.update();

        // Let's assume the system despawns the derelict and fires the event when threat drops to <= 0
        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let ev_list: Vec<_> = reader.read(events).collect();
        assert_eq!(ev_list.len(), 1, "Should fire chronicle event when derelict salvage completes");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
fn simulate_shipbreaker_salvage(
    mut commands: Commands,
    mut q_missions: Query<(Entity, &mut ShipbreakerMission)>,
    mut q_derelicts: Query<&mut DerelictEcosystem>,
    mut resources: ResMut<ColonyResources>,
    mut evt_chronicle: EventWriter<AddChronicleEvent>,
) {
    for (mission_entity, mut mission) in q_missions.iter_mut() {
        if let Ok(mut ecosystem) = q_derelicts.get_mut(mission.target_derelict) {
            // Apply fixed salvage progress per tick
            let progress_step = 10.0;
            mission.progress += progress_step;
            ecosystem.threat_level -= progress_step;

            // Yield a fraction of the total possible salvage based on progress
            let yield_step = ecosystem.salvage_yield * (progress_step / 100.0);
            resources.metal += yield_step;

            if ecosystem.threat_level <= 0.0 {
                commands.entity(mission.target_derelict).despawn();
                commands.entity(mission_entity).despawn();

                evt_chronicle.send(AddChronicleEvent {
                    template_id: "SYMBIOTIC_SALVAGE".to_string(),
                    args: vec![], // Populate correctly
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Combat Mechanics:** Instead of simple linear progress, tie the `threat_level` reduction to an actual combat resolution system utilizing Shipbreaker Pop stats (e.g., weapon types or specific traits).
- **Hazard Risks:** Add risk of Pop injury or death if the `threat_level` is significantly higher than the Shipbreaker fleet's combat strength. This reinforces the "hostile biome" concept.
- **Visuals/UI:** Expose the `threat_level` and remaining `salvage_yield` on the derelict's UI pane so the player can gauge the risk-reward tradeoff before committing Pops to the mission.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Processing a `ShipbreakerMission` yields resources and reduces `DerelictEcosystem` threat.
- [ ] When the threat is eliminated, the derelict is despawned, and the `SYMBIOTIC_SALVAGE` chronicle event fires.

## 7. Technical Guidance
- Integrate the `simulate_shipbreaker_salvage` system into the `SimulationSchedule`, likely near standard combat or resource gathering steps.
- Ensure that the generated Chronicle event has the correct template IDs as mapped out in `lore/TEMPLATES.md` (`SYMBIOTIC_SALVAGE` or `DERELICT_BREACHED`).

## 8. Questions
*Builder: add questions here if spec is unclear.*
