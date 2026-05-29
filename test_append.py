with open("src/layer3/diplomacy_reflection.rs", "r") as f:
    content = f.read()

new_tests = """
    #[test]
    fn test_aggregate_colony_stats_ignores_invalid_entities() {
        let mut app = App::new();
        app.add_event::<EntityKilledEvent>();
        app.add_event::<FloraPlantedEvent>();
        app.add_systems(Update, aggregate_colony_stats);

        let invalid_entity = Entity::from_raw(999);
        app.world_mut().send_event(EntityKilledEvent { colony_entity: invalid_entity });
        app.world_mut().send_event(FloraPlantedEvent { colony_entity: invalid_entity });

        app.update();
    }

    #[test]
    fn test_update_diplomatic_traits_no_change() {
        let mut app = App::new();
        app.add_event::<TraitChangedEvent>();
        app.add_systems(Update, update_diplomatic_traits);

        let civ = app.world_mut().spawn(DiplomaticTraits { ..default() }).id();
        app.world_mut().spawn(ColonyStats {
            owner_civ: civ,
            kills_last_year: 4999,
            trees_planted_last_year: 999,
        });

        app.update();

        let events = app.world().resource::<Events<TraitChangedEvent>>();
        let mut reader = events.get_cursor();
        assert!(reader.read(events).next().is_none());
    }

    #[test]
    fn test_apply_diplomatic_reactions_ignores_self_and_non_pacifist() {
        let mut app = App::new();
        app.add_event::<TraitChangedEvent>();
        app.add_systems(Update, apply_diplomatic_reactions);

        let player_civ = app.world_mut().spawn((
            Civilization { id: "player".to_string() },
            DiplomaticTraits { is_barbarian: true, is_pacifist: true, ..default() },
            DiplomaticRelations { relations: vec![DiplomaticStanding { target_id: "player".to_string(), standing: 0.0, sanctioned: false }] }
        )).id();

        let non_pacifist_neighbor = app.world_mut().spawn((
            Civilization { id: "neighbor1".to_string() },
            DiplomaticTraits { is_pacifist: false, ..default() },
            DiplomaticRelations { relations: vec![DiplomaticStanding { target_id: "player".to_string(), standing: 0.0, sanctioned: false }] }
        )).id();

        app.world_mut().send_event(TraitChangedEvent { civ_entity: player_civ });
        app.update();

        let player_relations = app.world().get::<DiplomaticRelations>(player_civ).unwrap();
        assert!(!player_relations.relations[0].sanctioned);

        let neighbor_relations = app.world().get::<DiplomaticRelations>(non_pacifist_neighbor).unwrap();
        assert!(!neighbor_relations.relations[0].sanctioned);
    }

    #[test]
    fn test_apply_diplomatic_reactions_ignores_other_relations() {
        let mut app = App::new();
        app.add_event::<TraitChangedEvent>();
        app.add_systems(Update, apply_diplomatic_reactions);

        let player_civ = app.world_mut().spawn((
            Civilization { id: "player".to_string() },
            DiplomaticTraits { is_barbarian: true, ..default() }
        )).id();

        let pacifist_neighbor = app.world_mut().spawn((
            Civilization { id: "neighbor".to_string() },
            DiplomaticTraits { is_pacifist: true, ..default() },
            DiplomaticRelations { relations: vec![
                DiplomaticStanding { target_id: "other".to_string(), standing: 0.0, sanctioned: false },
                DiplomaticStanding { target_id: "player".to_string(), standing: 0.0, sanctioned: false }
            ] }
        )).id();

        app.world_mut().send_event(TraitChangedEvent { civ_entity: player_civ });
        app.update();

        let neighbor_relations = app.world().get::<DiplomaticRelations>(pacifist_neighbor).unwrap();
        assert!(!neighbor_relations.relations[0].sanctioned);
        assert!(neighbor_relations.relations[1].sanctioned);
    }
"""

start_tests = content.find("mod tests {")
end_tests = content.find("}\n\n/// Unity currency", start_tests)

if start_tests != -1 and end_tests != -1:
    old_mod_tests = content[start_tests:end_tests]
    new_content = content[:start_tests] + old_mod_tests + new_tests + "}\n\n/// Unity currency" + content[end_tests + 20:]
    with open("src/layer3/diplomacy_reflection.rs", "w") as f:
        f.write(new_content)
