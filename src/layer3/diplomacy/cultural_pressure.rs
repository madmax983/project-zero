use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::social::culture::CulturalInfluenceGrid;
use crate::layer3::diplomacy_reflection::Civilization;
use bevy::prelude::*;

#[derive(PartialEq, Eq, Debug)]
pub enum DiplomaticStance {
    War,
    Neutral,
    Ally,
}

#[derive(Component)]
pub struct DiplomacyState {
    pub stance: DiplomaticStance,
    pub opinion: f32,
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

pub fn process_defections_system(
    mut defection_events: EventReader<DefectionEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for ev in defection_events.read() {
        // Here, logic would spawn incoming refugee/defector ships on Layer 2
        // For minimal implementation, we log it or add pops to Layer 1
        println!(
            "Received {} defectors from Civ {:?}",
            ev.amount, ev.source_civ
        );

        // REFACTOR Phase: Send an AddChronicleEvent documenting the defection
        chronicle_events.send(AddChronicleEvent {
            text: "Enemy soldiers lay down their arms to defect to our paradise".to_string(),
            importance: EventImportance::Major,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::resources::ColonyResources;
    use crate::layer1::social::culture::calculate_cultural_pressure_system;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(ColonyResources::default());
        app.insert_resource(CulturalInfluenceGrid::default());
        app.add_event::<DefectionEvent>();
        app.add_event::<AddChronicleEvent>();
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
        resources.art = 500.0;
        resources.luxury = 300.0;

        app.update();

        // Assert the cultural influence grid or score has increased
        // Since we added decay, after 1 tick at dt=1, decay_rate=0.05
        // target_pressure = 500 * 1.5 + 300 = 1050
        // new_pressure = 0 * 0.95 + 1050 * 0.05 = 52.5
        // Needs a few ticks to go > 100.0
        app.update();
        app.update();

        let _influence = app.world().get_resource::<CulturalInfluenceGrid>().unwrap();
        assert!(
            _influence.total_pressure > 100.0,
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

        // Assert chronicle event was fired
        let chronicle_events = app
            .world()
            .get_resource::<Events<AddChronicleEvent>>()
            .unwrap();
        let mut chron_reader = chronicle_events.get_cursor();
        let chron_ev = chron_reader.read(chronicle_events).next().unwrap();
        assert_eq!(
            chron_ev.text,
            "Enemy soldiers lay down their arms to defect to our paradise"
        );
    }
}
