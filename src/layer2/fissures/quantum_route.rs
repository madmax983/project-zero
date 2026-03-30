use bevy::prelude::*;
use rand::Rng;

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

#[derive(Component)]
pub struct CargoDecoherenceChecked;

#[derive(Component)]
pub struct CrewDecoherenceChecked;

#[derive(Component)]
pub struct DecoherenceChecked;

#[derive(Resource, Default)]
pub struct DecoherenceProbability(pub f32);

#[derive(PartialEq, Debug)]
pub enum FleetStatus {
    InTransit,
    Arrived,
}

pub fn quantum_fissure_transit_system(mut fleets: Query<&mut TradeFleet, With<QuantumRoute>>) {
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
    let mut rng = rand::thread_rng();

    for mut cargo in fleets.iter_mut() {
        if rng.gen::<f32>() < probability {
            cargo.items.clear();
            cargo.items.push("Toxic Sludge".to_string());
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn apply_crew_decoherence_system(
    mut commands: Commands,
    prob: Option<Res<DecoherenceProbability>>,
    fleets: Query<(Entity, &TradeFleet, Option<&Children>), (With<QuantumRoute>, Without<CrewDecoherenceChecked>)>,
    crew: Query<Entity, With<CrewMember>>,
) {
    let probability = prob.map(|p| p.0).unwrap_or(0.1);
    let mut rng = rand::thread_rng();

    for (entity, fleet, children_opt) in fleets.iter() {
        if fleet.status == FleetStatus::Arrived {
            if let Some(children) = children_opt {
                for child in children.iter() {
                    if crew.contains(*child) && rng.gen::<f32>() < probability {
                        commands.entity(*child).insert(CorruptedBiology);
                    }
                }
            }
            commands.entity(entity).insert(CrewDecoherenceChecked);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instantaneous_travel() {
        let mut app = App::new();
        app.add_systems(Update, quantum_fissure_transit_system);

        // Arrange
        let fleet = app
            .world_mut()
            .spawn((
                TradeFleet {
                    status: FleetStatus::InTransit,
                    transit_time_remaining: 100,
                },
                QuantumRoute,
            ))
            .id();

        // Act
        app.update();

        // Assert
        let fleet_state = app.world().get::<TradeFleet>(fleet).unwrap();
        assert_eq!(
            fleet_state.transit_time_remaining, 0,
            "Quantum route should reduce transit time to 0 instantly"
        );
        assert_eq!(
            fleet_state.status,
            FleetStatus::Arrived,
            "Quantum route should instantly arrive"
        );
    }

    #[test]
    fn test_cargo_decoherence() {
        let mut app = App::new();
        app.add_systems(Update, apply_decoherence_system);

        // Arrange
        let fleet = app
            .world_mut()
            .spawn((
                TradeFleet {
                    status: FleetStatus::Arrived,
                    transit_time_remaining: 0,
                },
                QuantumRoute,
                Cargo {
                    items: vec!["Credits".to_string(), "Iron".to_string()],
                },
            ))
            .id();

        // Act - Force deterministic decoherence for testing
        app.insert_resource(DecoherenceProbability(1.0)); // 100% chance
        app.update();

        // Assert
        let cargo = app.world().get::<Cargo>(fleet).unwrap();
        assert!(
            cargo.items.contains(&"Toxic Sludge".to_string()),
            "Cargo should be corrupted by decoherence"
        );
    }

    #[test]
    fn test_crew_decoherence() {
        let mut app = App::new();
        app.add_systems(Update, apply_crew_decoherence_system);

        // Arrange
        let pop = app.world_mut().spawn((Pop, CrewMember)).id();
        let _fleet = app
            .world_mut()
            .spawn((
                TradeFleet {
                    status: FleetStatus::Arrived,
                    transit_time_remaining: 0,
                },
                QuantumRoute,
            ))
            .add_child(pop)
            .id();

        // Act
        app.insert_resource(DecoherenceProbability(1.0)); // 100% chance
        app.update();

        // Assert
        let pop_state = app.world().get::<CorruptedBiology>(pop);
        assert!(
            pop_state.is_some(),
            "Crew members should gain corrupted biology if affected by decoherence"
        );
    }
}
