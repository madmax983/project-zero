use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::social::culture::CulturalInfluenceGrid;
use crate::layer3::diplomacy_reflection::{Civilization, DiplomaticRelations, DiplomaticTraits};
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct CulturalVulnerability {
    pub threshold: f32,
}

#[derive(Event)]
pub struct DefectionEvent {
    pub source_civ: Entity,
    pub amount: u32,
}

pub fn apply_cultural_pressure_system(
    influence: Option<Res<CulturalInfluenceGrid>>,
    mut civs: Query<
        (
            Entity,
            &mut DiplomaticRelations,
            &DiplomaticTraits,
            &CulturalVulnerability,
        ),
        With<Civilization>,
    >,
    mut defection_events: EventWriter<DefectionEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    if let Some(inf) = influence {
        for (civ_entity, mut diplomacy, traits, vulnerability) in civs.iter_mut() {
            if inf.total_pressure > vulnerability.threshold {
                // Check if they are hostile/warlike (representing War stance)
                if traits.is_warlike {
                    defection_events.send(DefectionEvent {
                        source_civ: civ_entity,
                        amount: 10,
                    });
                    chronicle_events.send(AddChronicleEvent {
                        text: "Enemy soldiers lay down their arms to defect to our paradise"
                            .to_string(),
                        importance: EventImportance::Major,
            ..Default::default()});

                    // Reduce their opinion of the player (target_id == "player") slightly
                    for relation in diplomacy.relations.iter_mut() {
                        if relation.target_id == "player" {
                            relation.standing -= 1.0;
                        }
                    }
                } else {
                    // If neutral/peaceful, they are drawn closer to alliance
                    for relation in diplomacy.relations.iter_mut() {
                        if relation.target_id == "player" {
                            relation.standing += 0.5;
                        }
                    }
                }
            }
        }
    }
}

pub fn process_defections_system(
    mut defection_events: EventReader<DefectionEvent>,
    _commands: Commands,
) {
    for ev in defection_events.read() {
        println!(
            "Received {} defectors from Civ {:?}",
            ev.amount, ev.source_civ
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::core::chronicle::Chronicle;
    use crate::layer3::diplomacy_reflection::DiplomaticStanding;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(CulturalInfluenceGrid::default());
        app.insert_resource(Chronicle::default());
        app.add_event::<DefectionEvent>();
        app.add_event::<AddChronicleEvent>();
        app.add_systems(
            Update,
            (apply_cultural_pressure_system, process_defections_system),
        );
        app
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
                DiplomaticRelations {
                    relations: vec![DiplomaticStanding {
                        target_id: "player".to_string(),
                        standing: -50.0,
                        sanctioned: false,
                    }],
                },
                DiplomaticTraits {
                    is_warlike: true,
                    ..Default::default()
                }, // Represents war stance
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
