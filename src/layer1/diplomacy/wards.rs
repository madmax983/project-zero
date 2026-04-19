use crate::layer1::health::Dead;
use crate::layer1::social::factions::FactionId;
use crate::layer1::Morale;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource)]
pub struct DiplomaticStanding {
    pub faction_relations: HashMap<FactionId, f32>,
}

#[derive(Component)]
pub struct DiplomaticWard {
    pub home_faction: FactionId,
}

#[derive(Event)]
pub struct WarDeclaredEvent {
    pub target_faction: FactionId,
}

pub fn process_diplomatic_wards_system(
    ward_query: Query<(&DiplomaticWard, &Morale)>,
    mut standing: ResMut<DiplomaticStanding>,
) {
    for (ward, morale) in ward_query.iter() {
        if let Some(relation) = standing.faction_relations.get_mut(&ward.home_faction) {
            if morale.value >= 80.0 {
                // High morale improves relations slowly
                *relation = (*relation + 0.5).clamp(-100.0, 100.0);
            } else if morale.value <= 20.0 {
                // Low morale damages relations
                *relation = (*relation - 1.0).clamp(-100.0, 100.0);
            }
        }
    }
}

pub fn process_ward_deaths_system(
    dead_wards: Query<&DiplomaticWard, With<Dead>>,
    mut standing: ResMut<DiplomaticStanding>,
    mut war_events: EventWriter<WarDeclaredEvent>,
) {
    for ward in dead_wards.iter() {
        if let Some(relation) = standing.faction_relations.get_mut(&ward.home_faction) {
            *relation = -100.0; // Instant bottom relations
        }
        war_events.send(WarDeclaredEvent {
            target_faction: ward.home_faction,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_diplomatic_ward_happiness_improves_relations() {
        let mut app = App::new();

        let home_faction = FactionId::FarmersGuild;
        app.world_mut().insert_resource(DiplomaticStanding {
            faction_relations: vec![(home_faction, 0.0)].into_iter().collect(),
        });

        // Spawn a ward with high morale
        let _ward = app
            .world_mut()
            .spawn((
                Pop,
                DiplomaticWard { home_faction },
                Morale {
                    value: 90.0,
                    ..Default::default()
                },
            ))
            .id();

        app.add_systems(Update, process_diplomatic_wards_system);

        // Run system to process ward morale
        app.update();

        // Relations should improve
        let standing = app.world().get_resource::<DiplomaticStanding>().expect("Component should exist or System should run");
        assert!(standing.faction_relations.get(&home_faction).expect("Component should exist or System should run") > &0.0);
    }

    #[test]
    fn test_diplomatic_ward_death_triggers_war() {
        let mut app = App::new();

        let home_faction = FactionId::FarmersGuild;
        app.world_mut().insert_resource(DiplomaticStanding {
            faction_relations: vec![(home_faction, 50.0)].into_iter().collect(),
        });

        app.add_event::<WarDeclaredEvent>();

        // Spawn a ward and then kill them
        let _ward = app
            .world_mut()
            .spawn((
                Pop,
                DiplomaticWard { home_faction },
                // Represent dead or dying pop (e.g. low health/dead component)
                Dead,
            ))
            .id();

        app.add_systems(Update, process_ward_deaths_system);

        // Run system
        app.update();

        // War event should be emitted
        let war_events = app.world().resource::<Events<WarDeclaredEvent>>();
        assert_eq!(war_events.len(), 1);

        let mut reader = war_events.get_cursor();
        let event = reader.read(war_events).next().expect("Component should exist or System should run");
        assert_eq!(event.target_faction, home_faction);

        // Relations should tank
        let standing = app.world().get_resource::<DiplomaticStanding>().expect("Component should exist or System should run");
        assert_eq!(
            *standing.faction_relations.get(&home_faction).expect("Component should exist or System should run"),
            -100.0
        );
    }

    #[test]
    fn test_diplomatic_ward_mistreatment_tanks_relations() {
        let mut app = App::new();

        let home_faction = FactionId::FarmersGuild;
        app.world_mut().insert_resource(DiplomaticStanding {
            faction_relations: vec![(home_faction, 50.0)].into_iter().collect(),
        });

        // Spawn a ward with terrible morale
        let _ward = app
            .world_mut()
            .spawn((
                Pop,
                DiplomaticWard { home_faction },
                Morale {
                    value: 10.0,
                    ..Default::default()
                },
            ))
            .id();

        app.add_systems(Update, process_diplomatic_wards_system);

        app.update();

        // Relations should degrade
        let standing = app.world().get_resource::<DiplomaticStanding>().expect("Component should exist or System should run");
        assert!(*standing.faction_relations.get(&home_faction).expect("Component should exist or System should run") < 50.0);
    }
}
