//! Diplomatic Reflection
//!
//! This module acts as the bridge between microscopic colony simulation (Layer 1) and
//! macroscopic galactic diplomacy (Layer 3). It observes local events like killing or
//! planting flora, aggregates them into [`ColonyStats`], and subsequently mutates the
//! global [`DiplomaticTraits`] of the owning civilization. These shifting traits then
//! cascade into changes in [`DiplomaticRelations`] with neighboring factions.

use bevy::prelude::*;

/// Fired when an entity is violently destroyed on a colony.
#[derive(Event)]
pub struct EntityKilledEvent {
    pub colony_entity: Entity,
}

/// Fired when new ecological flora is planted on a colony.
#[derive(Event)]
pub struct FloraPlantedEvent {
    pub colony_entity: Entity,
}

/// Aggregates local behavioral statistics for a specific colony over time.
///
/// These stats are periodically evaluated to determine if they should influence the
/// macro-scale [`DiplomaticTraits`] of the `owner_civ`.
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

/// Represents a macroscopic galactic faction.
#[derive(Component)]
pub struct Civilization {
    pub id: String,
}

/// Fired when a civilization's [`DiplomaticTraits`] have mutated based on their actions.
#[derive(Event)]
pub struct TraitChangedEvent {
    pub civ_entity: Entity,
}

/// High-level behavioral tags defining a civilization's public galactic identity.
///
/// Can be mutated organically through the actions of their constituent colonies.
#[derive(Component, Default)]
pub struct DiplomaticTraits {
    pub is_warlike: bool,
    pub is_barbarian: bool,
    pub is_ecological: bool,
    pub is_pacifist: bool,
}

/// A specific relationship metric towards a target civilization.
#[derive(Clone)]
pub struct DiplomaticStanding {
    pub target_id: String,
    pub standing: f32,
    pub sanctioned: bool,
}

/// A collection of all active diplomatic standings a civilization holds with others.
#[derive(Component)]
pub struct DiplomaticRelations {
    pub relations: Vec<DiplomaticStanding>,
}

/// Listens for local events and aggregates them into the running [`ColonyStats`].
///
/// Translates `EntityKilledEvent` into `kills_last_year` and `FloraPlantedEvent`
/// into `trees_planted_last_year`.
///
/// # Examples
/// ```
/// use bevy::prelude::*;
/// use scale::layer3::diplomacy_reflection::{ColonyStats, EntityKilledEvent, FloraPlantedEvent, aggregate_colony_stats};
///
/// let mut app = App::new();
/// app.add_event::<EntityKilledEvent>();
/// app.add_event::<FloraPlantedEvent>();
/// app.add_systems(Update, aggregate_colony_stats);
///
/// let colony = app.world_mut().spawn(ColonyStats::default()).id();
/// app.world_mut().send_event(EntityKilledEvent { colony_entity: colony });
/// app.update();
///
/// assert_eq!(app.world().get::<ColonyStats>(colony).unwrap().kills_last_year, 1);
/// ```
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

/// Evaluates [`ColonyStats`] and mutates the parent civilization's [`DiplomaticTraits`].
///
/// If a colony exhibits extreme behavior (e.g., mass killing or mass planting), the
/// parent civilization will acquire traits like `is_warlike` or `is_ecological`. When traits
/// change, a [`TraitChangedEvent`] is fired.
///
/// # Examples
/// ```
/// use bevy::prelude::*;
/// use scale::layer3::diplomacy_reflection::{ColonyStats, DiplomaticTraits, TraitChangedEvent, update_diplomatic_traits};
///
/// let mut app = App::new();
/// app.add_event::<TraitChangedEvent>();
/// app.add_systems(Update, update_diplomatic_traits);
///
/// let civ = app.world_mut().spawn(DiplomaticTraits::default()).id();
/// app.world_mut().spawn(ColonyStats { owner_civ: civ, kills_last_year: 5001, trees_planted_last_year: 0 });
/// app.update();
///
/// assert!(app.world().get::<DiplomaticTraits>(civ).unwrap().is_warlike);
/// ```
pub fn update_diplomatic_traits(
    colony_query: Query<&ColonyStats>,
    mut civ_query: Query<(Entity, &mut DiplomaticTraits)>,
    mut trait_events: EventWriter<TraitChangedEvent>,
) {
    for stats in colony_query.iter() {
        if let Ok((entity, mut civ)) = civ_query.get_mut(stats.owner_civ) {
            let mut changed = false;
            if stats.kills_last_year >= 5000 && (!civ.is_warlike || !civ.is_barbarian) {
                civ.is_warlike = true;
                civ.is_barbarian = true; // Simplified
                changed = true;
            }
            if stats.trees_planted_last_year >= 1000 && !civ.is_ecological {
                civ.is_ecological = true;
                changed = true;
            }

            if changed {
                trait_events.send(TraitChangedEvent { civ_entity: entity });
            }
        }
    }
}

/// Applies relational consequences when a civilization's traits change.
///
/// Listens for [`TraitChangedEvent`]. If a civilization becomes `is_barbarian`,
/// neighboring `is_pacifist` civilizations will automatically apply sanctions
/// and reduce their diplomatic standing.
///
/// # Examples
/// ```
/// use bevy::prelude::*;
/// use scale::layer3::diplomacy_reflection::{Civilization, DiplomaticRelations, DiplomaticStanding, DiplomaticTraits, TraitChangedEvent, apply_diplomatic_reactions};
///
/// let mut app = App::new();
/// app.add_systems(Update, apply_diplomatic_reactions);
///
/// let player = app.world_mut().spawn((
///     Civilization { id: "player".to_string() },
///     DiplomaticTraits { is_barbarian: true, ..default() }
/// )).id();
///
/// let neighbor = app.world_mut().spawn((
///     DiplomaticTraits { is_pacifist: true, ..default() },
///     DiplomaticRelations { relations: vec![DiplomaticStanding { target_id: "player".to_string(), standing: 0.0, sanctioned: false }] }
/// )).id();
///
/// // Create the reader event directly or use `Events` resource manually for this test format
/// let mut events = Events::<TraitChangedEvent>::default();
/// events.send(TraitChangedEvent { civ_entity: player });
/// app.insert_resource(events);
///
/// app.update();
///
/// assert!(app.world().get::<DiplomaticRelations>(neighbor).unwrap().relations[0].sanctioned);
/// ```
pub fn apply_diplomatic_reactions(
    mut trait_events: EventReader<TraitChangedEvent>,
    player_query: Query<(&Civilization, &DiplomaticTraits)>,
    mut neighbor_query: Query<(Entity, &DiplomaticTraits, &mut DiplomaticRelations)>,
) {
    for event in trait_events.read() {
        if let Ok((player_civ, player_traits)) = player_query.get(event.civ_entity) {
            if player_traits.is_barbarian {
                for (neighbor_entity, neighbor_traits, mut neighbor_relations) in
                    neighbor_query.iter_mut()
                {
                    // Make sure neighbor is not the player themselves
                    if neighbor_entity != event.civ_entity && neighbor_traits.is_pacifist {
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
        app.add_systems(
            Update,
            update_diplomatic_traits.after(aggregate_colony_stats),
        );

        let civ = app.world_mut().spawn(DiplomaticTraits { ..default() }).id();
        app.world_mut().spawn(ColonyStats {
            owner_civ: civ,
            kills_last_year: 5000,
            ..default()
        });

        app.update();

        let traits = app.world().get::<DiplomaticTraits>(civ).unwrap();
        assert!(
            traits.is_warlike,
            "High kill count should grant the Warlike trait"
        );
    }

    #[test]
    fn test_high_tree_planting_grants_ecological_trait() {
        let mut app = App::new();
        app.add_event::<EntityKilledEvent>();
        app.add_event::<FloraPlantedEvent>();
        app.add_event::<TraitChangedEvent>();
        app.add_systems(Update, aggregate_colony_stats);
        app.add_systems(
            Update,
            update_diplomatic_traits.after(aggregate_colony_stats),
        );

        let civ = app.world_mut().spawn(DiplomaticTraits { ..default() }).id();
        app.world_mut().spawn(ColonyStats {
            owner_civ: civ,
            trees_planted_last_year: 1000,
            ..default()
        });

        app.update();

        let traits = app.world().get::<DiplomaticTraits>(civ).unwrap();
        assert!(
            traits.is_ecological,
            "High tree planting should grant the Ecological trait"
        );
    }

    #[test]
    fn test_barbarian_trait_causes_sanctions_from_pacifist_neighbors() {
        let mut app = App::new();
        app.add_event::<TraitChangedEvent>();
        app.add_systems(Update, apply_diplomatic_reactions);

        let player_civ = app
            .world_mut()
            .spawn((
                Civilization {
                    id: "player".to_string(),
                },
                DiplomaticTraits {
                    is_barbarian: true,
                    ..default()
                },
            ))
            .id();

        app.world_mut().send_event(TraitChangedEvent {
            civ_entity: player_civ,
        });

        let neighbor = app
            .world_mut()
            .spawn((
                Civilization {
                    id: "neighbor".to_string(),
                },
                DiplomaticTraits {
                    is_pacifist: true,
                    ..default()
                },
                DiplomaticRelations {
                    relations: vec![DiplomaticStanding {
                        target_id: "player".to_string(),
                        standing: 0.0,
                        sanctioned: false,
                    }],
                },
            ))
            .id();

        app.update();

        let relations = app.world().get::<DiplomaticRelations>(neighbor).unwrap();
        assert!(
            relations.relations[0].sanctioned,
            "Pacifist neighbor should sanction a Barbarian civilization"
        );
        assert!(
            relations.relations[0].standing < 0.0,
            "Standing should decrease"
        );
    }
}

/// Unity currency gained through diplomatic and religious actions.
#[derive(Resource, Default)]
pub struct FaithCurrency {
    pub amount: f32,
}
