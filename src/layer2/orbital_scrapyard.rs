use crate::layer1::orbital_crossfire::OrbitalEvent;
use bevy::prelude::*;

#[derive(Component)]
pub struct OrbitalDebrisField {
    pub stability: f32, // 1.0 down to 0.0
}

#[derive(Event)]
pub struct DeorbitEvent {
    pub target: Vec2,
    pub mass: f32,
}

#[derive(Component)]
pub struct SalvageMission {
    pub progress: f32,
}

// System to complete salvage mission and lower stability
pub fn process_salvage_missions(
    mut mission_query: Query<(Entity, &mut SalvageMission, &crate::layer1::Parent)>,
    mut debris_query: Query<&mut OrbitalDebrisField>,
    mut resources: Option<ResMut<crate::layer1::resources::ColonyResources>>,
    mut commands: Commands,
) {
    for (mission_entity, mut mission, parent) in mission_query.iter_mut() {
        mission.progress += 0.1;
        if mission.progress >= 1.0 {
            // Give rewards (Tech/Resources)
            if let Some(ref mut res) = resources {
                res.add_credits(100.0); // Reward credits, rare_metals, etc
            }

            // Lower stability
            for mut debris in debris_query.iter_mut() {
                debris.stability -= 0.05;
            }
            commands.entity(mission_entity).despawn();

            // Parent is an entity directly
            commands.entity(parent.0).remove::<SalvageMission>();
        }
    }
}

// System to trigger deorbit events
pub fn check_deorbit_trigger(
    debris_query: Query<&OrbitalDebrisField>,
    mut deorbit_events: EventWriter<DeorbitEvent>,
) {
    for debris in debris_query.iter() {
        if debris.stability < 0.2 {
            deorbit_events.send(DeorbitEvent {
                target: Vec2::new(50.0, 50.0),
                mass: 1000.0,
            });
        }
    }
}

pub fn convert_deorbit_to_orbital(
    mut deorbit_events: EventReader<DeorbitEvent>,
    mut commands: Commands,
) {
    for event in deorbit_events.read() {
        // We will spawn the OrbitalEvent as a component since impact_system expects a component!
        commands.spawn(OrbitalEvent {
            target: crate::layer1::map::GridPosition {
                x: event.target.x as i32,
                y: event.target.y as i32,
            },
            damage: event.mass,
            heat: event.mass / 2.0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::resources::ColonyResources;

    // 1. Salvage Missions Yield Tech
    #[test]
    fn test_salvage_mission_yields_tech_blueprints() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let debris_field = app
            .world_mut()
            .spawn(OrbitalDebrisField { stability: 1.0 })
            .id();
        let fleet = app.world_mut().spawn_empty().id();
        let mission = app
            .world_mut()
            .spawn((
                SalvageMission { progress: 0.9 },
                crate::layer1::Parent(fleet),
            ))
            .id();

        app.world_mut().insert_resource(ColonyResources::default());

        app.add_systems(
            Update,
            crate::layer2::orbital_scrapyard::process_salvage_missions,
        );
        app.update();

        let resources = app.world_mut().resource::<ColonyResources>();
        assert!(resources.credits > 0.0, "Salvage should grant rewards");

        let debris = app
            .world_mut()
            .get::<OrbitalDebrisField>(debris_field)
            .unwrap();
        assert!(debris.stability < 1.0, "Stability should decrease");

        assert!(
            app.world_mut().get::<SalvageMission>(mission).is_none(),
            "Mission should be removed on completion"
        );
    }

    // 2. Deorbit Event Trigger
    #[test]
    fn test_salvage_operation_increases_deorbit_chance() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.world_mut().spawn(OrbitalDebrisField { stability: 0.1 });
        app.world_mut()
            .insert_resource(Events::<DeorbitEvent>::default());

        app.add_systems(
            Update,
            crate::layer2::orbital_scrapyard::check_deorbit_trigger,
        );
        app.update();

        let events = app.world_mut().resource::<Events<DeorbitEvent>>();
        assert!(
            !events.is_empty(),
            "DeorbitEvent should be triggered when stability is low"
        );
    }

    // 3. Debris Impact Consequences
    #[test]
    fn test_debris_impact_destroys_buildings_and_spawns_scrap() {
        use crate::layer1::building::{Building, BuildingType};
        use crate::layer1::map::GridPosition;
        use crate::layer1::orbital_crossfire::{impact_system, ImpactSite};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let target_pos = GridPosition { x: 50, y: 50 };
        let building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                target_pos,
                crate::layer1::structure::Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        app.world_mut()
            .insert_resource(Events::<DeorbitEvent>::default());
        app.world_mut()
            .resource_mut::<Events<DeorbitEvent>>()
            .send(DeorbitEvent {
                target: Vec2::new(50.0, 50.0),
                mass: 1000.0,
            });

        app.add_systems(Update, (convert_deorbit_to_orbital, impact_system).chain());
        app.update();

        assert!(
            app.world_mut().get::<Building>(building).is_none(),
            "Impact should destroy building"
        );

        let mut found_scrap = false;
        let mut query = app.world_mut().query::<(&ImpactSite, &GridPosition)>();
        for (_, pos) in query.iter(app.world_mut()) {
            if pos.x == target_pos.x && pos.y == target_pos.y {
                found_scrap = true;
            }
        }
        assert!(found_scrap, "Impact should spawn harvestable scrap");
    }
}
