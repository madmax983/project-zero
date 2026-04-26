use crate::layer1::economy::resources::ColonyResources;
use crate::layer3::diplomacy_reflection::Civilization;
use bevy_ecs::prelude::*;

#[derive(PartialEq, Eq)]
pub enum DiplomaticStance {
    War,
    Neutral,
}

#[derive(Component)]
pub struct DiplomacyState {
    pub stance: DiplomaticStance,
    pub opinion: f32,
}

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

pub fn calculate_cultural_pressure_system(
    resources: Res<ColonyResources>,
    mut influence: ResMut<CulturalInfluenceGrid>,
    time: Res<crate::shared::time::SimulationTime>,
) {
    // Calculate current tick pressure
    // Formula: (Art * 1.5) + Luxury = Pressure
    // Using knowledge and clothing as stand-ins
    let current_output = (resources.knowledge * 1.5) + resources.clothing;

    if time.tick.is_multiple_of(10) {
        // Run as a moving average/decaying score
        // Slowly move towards current_output
        let difference = current_output - influence.total_pressure;
        influence.total_pressure += difference * 0.1;
    }
}

pub fn apply_cultural_pressure_system(
    mut influence: ResMut<CulturalInfluenceGrid>,
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
                // Consume pressure so we don't repeatedly trigger defection
                influence.total_pressure =
                    (influence.total_pressure - vulnerability.threshold).max(0.0);

                // Reduce their opinion slightly as they grow resentful of your influence
                diplomacy.opinion -= 1.0;
            } else if diplomacy.stance == DiplomaticStance::Neutral {
                // If neutral, they are drawn closer to alliance
                diplomacy.opinion += 0.5;
            }
        }
    }
}

use crate::layer1::core::chronicle::AddChronicleEvent;
use crate::layer1::core::chronicle::EventImportance;

pub fn process_defections_system(
    mut defection_events: EventReader<DefectionEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for ev in defection_events.read() {
        // Here, logic would spawn incoming refugee/defector ships on Layer 2
        // For minimal implementation, we log it or add pops to Layer 1

        chronicle_events.send(AddChronicleEvent {
            text: format!(
                "Enemy soldiers lay down their arms to defect to our paradise. ({} defectors)",
                ev.amount
            ),
            importance: EventImportance::Major,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(ColonyResources::default());
        app.insert_resource(CulturalInfluenceGrid::default());
        app.add_event::<DefectionEvent>();
        app.insert_resource(crate::shared::time::SimulationTime::default());
        app.add_event::<crate::layer1::core::chronicle::AddChronicleEvent>();
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

        // Workaround since `art` and `luxury` fields don't exist on `ColonyResources`.
        // We'll use `clothing` as a stand-in for `luxury` and `knowledge` as a stand-in for `art`.
        let mut resources = app.world_mut().resource_mut::<ColonyResources>();
        resources.knowledge = 500.0; // Art
        resources.clothing = 300.0; // Luxury

        // Since it's a moving average, we must step until a tick divisible by 10 to see a change.
        for _ in 0..11 {
            app.world_mut()
                .resource_mut::<crate::shared::time::SimulationTime>()
                .tick += 1;
            app.update();
        }

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
        let mut reader = defection_events.get_cursor();
        let ev = reader.read(defection_events).next().unwrap();

        assert_eq!(
            ev.source_civ, enemy_civ,
            "Enemy civ should suffer defections due to overwhelming cultural pressure"
        );
    }
}
