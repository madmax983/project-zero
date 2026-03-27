use bevy::prelude::*;

#[derive(Event)]
pub struct EntityKilledEvent {
    pub colony_entity: Entity,
}

#[derive(Event)]
pub struct FloraPlantedEvent {
    pub colony_entity: Entity,
}

#[derive(Component)]
pub struct ColonyStats {
    pub owner_civ: Entity,
    pub kills_last_year: u32,
    pub trees_planted_last_year: u32,
}

impl Default for ColonyStats {
    fn default() -> Self {
        Self {
            owner_civ: Entity::PLACEHOLDER,
            kills_last_year: 0,
            trees_planted_last_year: 0,
        }
    }
}

#[derive(Component)]
pub struct Civilization {
    pub id: String,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum DiplomaticTrait {
    Warlike,
    Barbarian,
    Ecological,
    Pacifist,
}

#[derive(Event)]
pub struct TraitChangedEvent {
    pub civ_entity: Entity,
}

#[derive(Component, Default)]
pub struct DiplomaticTraits {
    pub traits: Vec<DiplomaticTrait>,
}

#[derive(Clone)]
pub struct DiplomaticStanding {
    pub target_id: String,
    pub standing: f32,
    pub sanctioned: bool,
}

#[derive(Component)]
pub struct DiplomaticRelations {
    pub relations: Vec<DiplomaticStanding>,
}

pub fn aggregate_colony_stats(
    mut kill_events: EventReader<EntityKilledEvent>,
    mut plant_events: EventReader<FloraPlantedEvent>,
    mut stats_query: Query<(Entity, &mut ColonyStats)>,
) {
    for event in kill_events.read() {
        if let Ok((_, mut stats)) = stats_query.get_mut(event.colony_entity) {
            stats.kills_last_year += 1;
        }
    }

    for event in plant_events.read() {
        if let Ok((_, mut stats)) = stats_query.get_mut(event.colony_entity) {
            stats.trees_planted_last_year += 1;
        }
    }
}

pub fn update_diplomatic_traits(
    colony_query: Query<&ColonyStats>,
    mut civ_query: Query<(Entity, &mut DiplomaticTraits)>,
    mut trait_events: EventWriter<TraitChangedEvent>,
) {
    for stats in colony_query.iter() {
        if let Ok((entity, mut civ)) = civ_query.get_mut(stats.owner_civ) {
            let mut changed = false;
            if stats.kills_last_year >= 5000 && !civ.traits.contains(&DiplomaticTrait::Warlike) {
                civ.traits.push(DiplomaticTrait::Warlike);
                civ.traits.push(DiplomaticTrait::Barbarian); // Simplified
                changed = true;
            }
            if stats.trees_planted_last_year >= 1000 && !civ.traits.contains(&DiplomaticTrait::Ecological) {
                civ.traits.push(DiplomaticTrait::Ecological);
                changed = true;
            }

            if changed {
                trait_events.send(TraitChangedEvent { civ_entity: entity });
            }
        }
    }
}

pub fn apply_diplomatic_reactions(
    mut trait_events: EventReader<TraitChangedEvent>,
    player_query: Query<(&Civilization, &DiplomaticTraits)>,
    mut neighbor_query: Query<(Entity, &DiplomaticTraits, &mut DiplomaticRelations)>,
) {
    for event in trait_events.read() {
        if let Ok((player_civ, player_traits)) = player_query.get(event.civ_entity) {
            if player_traits.traits.contains(&DiplomaticTrait::Barbarian) {
                for (neighbor_entity, neighbor_traits, mut neighbor_relations) in neighbor_query.iter_mut() {
                    // Make sure neighbor is not the player themselves
                    if neighbor_entity != event.civ_entity && neighbor_traits.traits.contains(&DiplomaticTrait::Pacifist) {
                        for relation in neighbor_relations.relations.iter_mut() {
                            if relation.target_id == player_civ.id {
                                relation.sanctioned = true;
                                relation.standing -= 50.0;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_colony_kills_grants_warlike_trait() {
        let mut app = App::new();
        app.add_event::<EntityKilledEvent>();
        app.add_event::<FloraPlantedEvent>();
        app.add_event::<TraitChangedEvent>();
        app.add_systems(Update, aggregate_colony_stats);
        app.add_systems(Update, update_diplomatic_traits.after(aggregate_colony_stats));

        let civ = app.world_mut().spawn(DiplomaticTraits { traits: vec![] }).id();
        app.world_mut().spawn(ColonyStats { owner_civ: civ, kills_last_year: 5000, ..default() });

        app.update();

        let traits = app.world().get::<DiplomaticTraits>(civ).unwrap();
        assert!(traits.traits.contains(&DiplomaticTrait::Warlike), "High kill count should grant the Warlike trait");
    }

    #[test]
    fn test_high_tree_planting_grants_ecological_trait() {
        let mut app = App::new();
        app.add_event::<EntityKilledEvent>();
        app.add_event::<FloraPlantedEvent>();
        app.add_event::<TraitChangedEvent>();
        app.add_systems(Update, aggregate_colony_stats);
        app.add_systems(Update, update_diplomatic_traits.after(aggregate_colony_stats));

        let civ = app.world_mut().spawn(DiplomaticTraits { traits: vec![] }).id();
        app.world_mut().spawn(ColonyStats { owner_civ: civ, trees_planted_last_year: 1000, ..default() });

        app.update();

        let traits = app.world().get::<DiplomaticTraits>(civ).unwrap();
        assert!(traits.traits.contains(&DiplomaticTrait::Ecological), "High tree planting should grant the Ecological trait");
    }

    #[test]
    fn test_barbarian_trait_causes_sanctions_from_pacifist_neighbors() {
        let mut app = App::new();
        app.add_event::<TraitChangedEvent>();
        app.add_systems(Update, apply_diplomatic_reactions);

        let player_civ = app.world_mut().spawn((
            Civilization { id: "player".to_string() },
            DiplomaticTraits { traits: vec![DiplomaticTrait::Barbarian] }
        )).id();

        app.world_mut().send_event(TraitChangedEvent { civ_entity: player_civ });

        let neighbor = app.world_mut().spawn((
             Civilization { id: "neighbor".to_string() },
             DiplomaticTraits { traits: vec![DiplomaticTrait::Pacifist] },
             DiplomaticRelations { relations: vec![DiplomaticStanding { target_id: "player".to_string(), standing: 0.0, sanctioned: false }] }
        )).id();

        app.update();

        let relations = app.world().get::<DiplomaticRelations>(neighbor).unwrap();
        assert!(relations.relations[0].sanctioned, "Pacifist neighbor should sanction a Barbarian civilization");
        assert!(relations.relations[0].standing < 0.0, "Standing should decrease");
    }
}
