use crate::layer1::economy::resources::ColonyResources;
use crate::layer3::diplomacy_reflection::Civilization;
use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct CulturalInfluenceGrid {
    pub total_pressure: f32,
}

#[derive(Component)]
pub struct CulturalVulnerability {
    pub threshold: f32,
}

#[derive(Event)]
pub struct DefectionEvent {
    pub source_civ: Entity,
    pub amount: u32,
}

#[derive(Component, Default)]
pub struct DiplomacyState {
    pub stance: DiplomaticStance,
    pub opinion: f32,
}

#[derive(PartialEq, Eq, Debug, Default)]
pub enum DiplomaticStance {
    #[default]
    Neutral,
    War,
}

pub fn calculate_cultural_pressure_system(
    resources: Res<ColonyResources>,
    mut influence: ResMut<CulturalInfluenceGrid>,
) {
    // Formula: (Art * 1.5) + Luxury = Pressure
    influence.total_pressure = (resources.clothing * 1.5) + resources.alcohol;
}

pub fn apply_cultural_pressure_system(
    influence: Res<CulturalInfluenceGrid>,
    mut civs: Query<(Entity, &mut DiplomacyState, &CulturalVulnerability), With<Civilization>>,
    mut defection_events: EventWriter<DefectionEvent>,
) {
    for (civ_entity, mut diplomacy, vulnerability) in civs.iter_mut() {
        if influence.total_pressure > vulnerability.threshold {
            // Apply diplomatic penalty if they are hostile
            if diplomacy.stance == DiplomaticStance::War {
                // Defection logic
                defection_events.send(DefectionEvent {
                    source_civ: civ_entity,
                    amount: 10, // Base defection rate
                });

                // Reduce their opinion slightly as they grow resentful of your influence
                diplomacy.opinion -= 1.0;
            } else if diplomacy.stance == DiplomaticStance::Neutral {
                // If neutral, they are drawn closer to alliance
                diplomacy.opinion += 0.5;
            }
        }
    }
}

pub fn process_defections_system(mut defection_events: EventReader<DefectionEvent>) {
    for _ev in defection_events.read() {
        // Just consume the event for the minimal implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{App, Update};

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(ColonyResources::default());
        app.insert_resource(CulturalInfluenceGrid::default());
        app.add_event::<DefectionEvent>();
        app.add_systems(
            Update,
            (
                calculate_cultural_pressure_system,
                apply_cultural_pressure_system,
                process_defections_system,
            ),
        );
        app
    }

    #[test]
    fn test_high_art_luxury_generates_cultural_pressure() {
        let mut app = setup_app();

        let mut resources = app.world_mut().resource_mut::<ColonyResources>();
        // Using existing resources as proxies for art and luxury if none exist
        resources.clothing = 500.0;
        resources.alcohol = 300.0;

        app.update();

        // Assert the cultural influence grid or score has increased
        let influence = app.world().get_resource::<CulturalInfluenceGrid>().unwrap();
        assert!(
            influence.total_pressure > 100.0,
            "High art and luxury should generate significant cultural pressure"
        );
    }

    #[test]
    fn test_cultural_pressure_causes_enemy_defections_during_war() {
        let mut app = setup_app();

        // Setup high cultural pressure
        app.world_mut()
            .resource_mut::<CulturalInfluenceGrid>()
            .total_pressure = 1000.0;

        let enemy_civ = app
            .world_mut()
            .spawn((
                Civilization {
                    id: "The Hegemony".to_string(),
                },
                DiplomacyState {
                    stance: DiplomaticStance::War,
                    opinion: -50.0,
                },
                CulturalVulnerability { threshold: 500.0 }, // Defect if pressure > threshold
            ))
            .id();

        app.update();

        // Assert defection event was fired
        let defection_events = app
            .world()
            .get_resource::<Events<DefectionEvent>>()
            .unwrap();
        let mut cursor = defection_events.get_cursor();
        let ev = cursor.read(defection_events).next().unwrap();

        assert_eq!(
            ev.source_civ, enemy_civ,
            "Enemy civ should suffer defections due to overwhelming cultural pressure"
        );
    }
}
