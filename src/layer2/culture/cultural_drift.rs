use bevy::prelude::*;


#[derive(Resource)]
pub struct HomeworldLocation {
    pub position: Vec2,
}

#[derive(Component)]
pub struct ColonyMarker;

#[derive(Component)]
pub struct CulturalDrift {
    pub value: f32,
    pub independence_threshold: f32,
}

#[derive(Component)]
pub struct CommsRelay {
    pub is_active: bool,
}

#[derive(Component)]
pub struct Faction {
    pub id: u32,
}

pub fn calculate_cultural_drift_system(
    homeworld: Res<HomeworldLocation>,
    mut colonies: Query<(&Transform, &CommsRelay, &mut CulturalDrift), With<ColonyMarker>>,
) {
    for (transform, comms, mut drift) in colonies.iter_mut() {
        let pos2d = Vec2::new(transform.translation.x, transform.translation.y);
        let distance = pos2d.distance(homeworld.position);

        let mut rate = distance / 1000.0; // Base drift based on distance

        if comms.is_active {
            rate *= 0.1; // 90% reduction in drift if talking to homeworld
        }

        drift.value += rate;
    }
}

pub fn handle_independence_system(
    mut colonies: Query<(&CulturalDrift, &mut Faction), With<ColonyMarker>>,
) {
    for (drift, mut faction) in colonies.iter_mut() {
        if drift.value >= drift.independence_threshold {
            // Assign a new faction ID to symbolize independence
            // In a real implementation, this would trigger diplomacy events
            if faction.id == 0 {
                faction.id = 999;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(
            Update,
            (calculate_cultural_drift_system, handle_independence_system),
        );
        app.insert_resource(HomeworldLocation {
            position: Vec2::ZERO,
        });
        app
    }

    #[test]
    fn test_colony_drift_increases_with_distance_and_time() {
        let mut app = setup_app();

        // Spawn a colony far away
        let colony_id = app
            .world_mut()
            .spawn((
                ColonyMarker,
                Transform::from_xyz(1000.0, 0.0, 0.0),
                CulturalDrift {
                    value: 0.0,
                    independence_threshold: 100.0,
                },
                CommsRelay { is_active: false },
            ))
            .id();

        app.update(); // Tick 1

        let drift = app.world().get::<CulturalDrift>(colony_id).unwrap();
        assert!(drift.value > 0.0); // Drift should increase
    }

    #[test]
    fn test_active_comms_reduce_drift_rate() {
        let mut app = setup_app();

        let far_colony_id = app
            .world_mut()
            .spawn((
                ColonyMarker,
                Transform::from_xyz(1000.0, 0.0, 0.0),
                CulturalDrift {
                    value: 0.0,
                    independence_threshold: 100.0,
                },
                CommsRelay { is_active: false },
            ))
            .id();

        let comms_colony_id = app
            .world_mut()
            .spawn((
                ColonyMarker,
                Transform::from_xyz(1000.0, 0.0, 0.0),
                CulturalDrift {
                    value: 0.0,
                    independence_threshold: 100.0,
                },
                CommsRelay { is_active: true }, // Active comms!
            ))
            .id();

        app.update();

        let drift_no_comms = app
            .world()
            .get::<CulturalDrift>(far_colony_id)
            .unwrap()
            .value;
        let drift_with_comms = app
            .world()
            .get::<CulturalDrift>(comms_colony_id)
            .unwrap()
            .value;

        assert!(drift_with_comms < drift_no_comms);
    }

    #[test]
    fn test_colony_declares_independence() {
        let mut app = setup_app();

        let colony_id = app
            .world_mut()
            .spawn((
                ColonyMarker,
                Transform::from_xyz(1000.0, 0.0, 0.0),
                CulturalDrift {
                    value: 101.0,
                    independence_threshold: 100.0,
                },
                CommsRelay { is_active: false },
                Faction { id: 0 }, // Player faction
            ))
            .id();

        app.update(); // Independence check runs

        let faction = app.world().get::<Faction>(colony_id).unwrap();
        assert_ne!(faction.id, 0); // Faction should have changed
    }
}
