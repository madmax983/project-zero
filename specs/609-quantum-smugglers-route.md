# The Quantum Smuggler's Route

**1. Overview**
A shortcut through reality that cuts travel time to zero, but occasionally leaves parts of you behind. You discover a "Quantum Fissure" in Layer 2. Trading through it to Layer 3 empires is instantaneous, completely bypassing all blockades and travel times. However, there is a small, compounding "Decoherence" chance. Items, or even Pops, sent through the fissure might arrive fundamentally altered, corrupted, or completely erased from reality. The unparalleled strategic and economic advantage of instantaneous travel comes at the terrifying, unpredictable corruption of matter and life crossing the quantum threshold.

**2. Dependencies**
- `layer2::fissures::QuantumFissure`
- `layer3::trade::TradeFleet`
- `layer1::cargo::Shipment`
- `layer1::biology::Traits`

**3. RED Phase: Tests First**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_instantaneous_travel() {
        let mut app = App::new();
        app.add_systems(Update, quantum_fissure_transit_system);

        // Arrange
        let fleet = app.world_mut().spawn((
            TradeFleet { status: FleetStatus::InTransit, transit_time_remaining: 100 },
            QuantumRoute,
        )).id();

        // Act
        app.update();

        // Assert
        let fleet_state = app.world().get::<TradeFleet>(fleet).unwrap();
        assert_eq!(fleet_state.transit_time_remaining, 0, "Quantum route should reduce transit time to 0 instantly");
        assert_eq!(fleet_state.status, FleetStatus::Arrived, "Quantum route should instantly arrive");
    }

    #[test]
    fn test_cargo_decoherence() {
        let mut app = App::new();
        app.add_systems(Update, apply_decoherence_system);

        // Arrange
        let fleet = app.world_mut().spawn((
            TradeFleet { status: FleetStatus::Arrived, transit_time_remaining: 0 },
            QuantumRoute,
            Cargo { items: vec!["Credits".to_string(), "Iron".to_string()] },
        )).id();

        // Act - Force deterministic decoherence for testing
        app.insert_resource(DecoherenceProbability(1.0)); // 100% chance
        app.update();

        // Assert
        let cargo = app.world().get::<Cargo>(fleet).unwrap();
        assert!(cargo.items.contains(&"Toxic Sludge".to_string()), "Cargo should be corrupted by decoherence");
    }

    #[test]
    fn test_crew_decoherence() {
        let mut app = App::new();
        app.add_systems(Update, apply_crew_decoherence_system);

        // Arrange
        let pop = app.world_mut().spawn((Pop, CrewMember)).id();
        let fleet = app.world_mut().spawn((
            TradeFleet { status: FleetStatus::Arrived, transit_time_remaining: 0 },
            QuantumRoute,
        )).push_children(&[pop]).id();

        // Act
        app.insert_resource(DecoherenceProbability(1.0)); // 100% chance
        app.update();

        // Assert
        let pop_state = app.world().get::<CorruptedBiology>(pop);
        assert!(pop_state.is_some(), "Crew members should gain corrupted biology if affected by decoherence");
    }
}
```

**4. GREEN Phase: Minimal Implementation**

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct TradeFleet {
    pub status: FleetStatus,
    pub transit_time_remaining: u32,
}

#[derive(Component)]
pub struct QuantumRoute;

#[derive(Component)]
pub struct Cargo {
    pub items: Vec<String>,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct CrewMember;

#[derive(Component)]
pub struct CorruptedBiology;

#[derive(Resource)]
pub struct DecoherenceProbability(pub f32);

#[derive(PartialEq, Debug)]
pub enum FleetStatus {
    InTransit,
    Arrived,
}

pub fn quantum_fissure_transit_system(
    mut fleets: Query<&mut TradeFleet, With<QuantumRoute>>,
) {
    for mut fleet in fleets.iter_mut() {
        if fleet.status == FleetStatus::InTransit {
            fleet.transit_time_remaining = 0;
            fleet.status = FleetStatus::Arrived;
        }
    }
}

pub fn apply_decoherence_system(
    prob: Option<Res<DecoherenceProbability>>,
    mut fleets: Query<&mut Cargo, (With<QuantumRoute>, Changed<TradeFleet>)>,
) {
    let probability = prob.map(|p| p.0).unwrap_or(0.1); // Default 10%
    if probability > 0.0 {
        for mut cargo in fleets.iter_mut() {
            // Minimal implementation: replace all cargo with toxic sludge
            cargo.items.clear();
            cargo.items.push("Toxic Sludge".to_string());
        }
    }
}

pub fn apply_crew_decoherence_system(
    mut commands: Commands,
    prob: Option<Res<DecoherenceProbability>>,
    fleets: Query<&Children, (With<QuantumRoute>, Changed<TradeFleet>)>,
    crew: Query<Entity, With<CrewMember>>,
) {
    let probability = prob.map(|p| p.0).unwrap_or(0.1);
    if probability > 0.0 {
        for children in fleets.iter() {
            for child in children.iter() {
                if crew.contains(*child) {
                    commands.entity(*child).insert(CorruptedBiology);
                }
            }
        }
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- Replace deterministic `1.0` probability with a random number generator bounded by `DecoherenceProbability` in non-test scenarios.
- Make the `CorruptedBiology` trait more robust, adding specific negative modifiers (e.g., lower health, strange needs) or a `SimulationEvent` to notify the player.
- Ensure the sludge generates hazard events upon docking in Layer 1.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Trade fleets using `QuantumRoute` arrive instantly.
- [ ] `Cargo` has a chance to turn into `Toxic Sludge`.
- [ ] `CrewMember`s have a chance to gain `CorruptedBiology`.

**7. Technical Guidance**
- Using `Changed<TradeFleet>` inside the decoherence system ensures the corruption logic only runs the frame the fleet arrives, instead of every frame.
- Cargo item strings are fine for minimal implementation, but should integrate with the project's formal enum type for Items/Resources if one exists.

**8. Questions**
*Builder: add questions here if spec is unclear.*
