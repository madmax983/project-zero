use crate::layer1::resources::ResourceType;
use crate::layer2::events::ShipDestroyedEvent;
use crate::layer2::fleet::{FleetComposition, FleetFaction, InOrbit};
use crate::layer2::ship::ShipType;
use bevy_ecs::prelude::*;

/// Result of a fleet combat engagement.
#[derive(Debug, Clone, PartialEq)]
pub struct CombatResult {
    /// The faction that won the engagement.
    pub winner: FleetFaction,
    /// The composition of the winning fleet after combat.
    pub winner_survivors: FleetComposition,
    /// The composition of the losing fleet after combat (if any survived).
    pub loser_survivors: FleetComposition,
    /// List of ships destroyed during combat (from both sides).
    pub destroyed_ships: Vec<ShipType>,
    /// Loot generated from the engagement (e.g. from destroyed transport ships).
    pub loot: Option<Vec<(ResourceType, f32)>>,
}

/// Resolves a combat encounter between two fleets.
///
/// Determines the winner based on total attack power.
/// Applies damage to both sides based on the opponent's power.
/// Generates loot if valuable ships are destroyed.
#[must_use]
pub fn resolve_combat(
    att_faction: FleetFaction,
    attacker: &FleetComposition,
    def_faction: FleetFaction,
    defender: &FleetComposition,
) -> CombatResult {
    let att_power: f32 = attacker
        .ships
        .iter()
        .map(|s| s.ship_type.attack_power())
        .sum();
    let def_power: f32 = defender
        .ships
        .iter()
        .map(|s| s.ship_type.attack_power())
        .sum();

    // Attacker wins ties
    let attacker_wins = att_power >= def_power;

    let (winner_faction, winner_comp, loser_comp, winner_power, loser_power) = if attacker_wins {
        (att_faction, attacker, defender, att_power, def_power)
    } else {
        (def_faction, defender, attacker, def_power, att_power)
    };

    // Damage Calculation
    // Winner takes damage = Loser Power * 0.5 (Mitigation)
    // Loser takes damage = Winner Power * 1.5 (Overwhelming force)
    let winner_damage = loser_power * 0.5;
    let loser_damage = winner_power * 1.5;

    let mut winner_survivors = winner_comp.clone();
    let winner_destroyed = winner_survivors.take_damage(winner_damage);

    let mut loser_survivors = loser_comp.clone();
    let loser_destroyed = loser_survivors.take_damage(loser_damage);

    // Loot Generation
    // If any Transport ships were destroyed, generate loot.
    let mut loot = Vec::new();
    let mut scrap_amount = 0.0;

    for ship_type in &loser_destroyed {
        // Scavenge raw materials
        let cost = ship_type.construction_cost();
        for (_, amount) in cost {
            scrap_amount += amount * 0.1; // 10% recovery
        }

        // Bonus for Transports
        if *ship_type == ShipType::Transport {
            scrap_amount += 100.0; // Cargo loot
        }
    }

    if scrap_amount > 0.0 {
        loot.push((ResourceType::Scrap, scrap_amount));
    }

    // Collect all destroyed ships
    let mut destroyed_ships = winner_destroyed;
    destroyed_ships.extend(loser_destroyed);

    CombatResult {
        winner: winner_faction,
        winner_survivors,
        loser_survivors,
        destroyed_ships,
        loot: if loot.is_empty() { None } else { Some(loot) },
    }
}

/// System to resolve combat between fleets at the same orbital location.
pub fn fleet_combat_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &InOrbit,
        &FleetFaction,
        &mut FleetComposition,
        Option<&AvoidCombat>,
    )>,
    mut event_writer: EventWriter<ShipDestroyedEvent>,
) {
    // Naive O(N^2) check for MVP. Optimized: Sort by location or use HashMap.
    // Given low N of fleets, sorting/grouping is fine.

    // Vector of (Entity, Parent, Faction, Composition, AvoidCombat)
    let mut fleets: Vec<_> = query
        .iter_mut()
        .map(|(e, o, f, c, a)| (e, o.parent, *f, c.clone(), a.is_some()))
        .collect();

    // Group by location
    fleets.sort_by_key(|k| k.1);

    // Iterate and find conflicts
    let mut i = 0;
    while i < fleets.len() {
        let current_location = fleets[i].1;
        let mut group_end = i;
        while group_end < fleets.len() && fleets[group_end].1 == current_location {
            group_end += 1;
        }

        // Slice of fleets at this location
        let location_fleets = &mut fleets[i..group_end];

        if location_fleets.len() > 1 {
            // Check for hostile factions (different factions = hostile for now)
            // Pick first two different factions to fight
            let mut combat_pair = None;
            for j in 0..location_fleets.len() {
                for k in j + 1..location_fleets.len() {
                    if location_fleets[j].2 != location_fleets[k].2 {
                        combat_pair = Some((j, k));
                        break;
                    }
                }
                if combat_pair.is_some() {
                    break;
                }
            }

            if let Some((idx1, idx2)) = combat_pair {
                let (e1, _, f1, c1, a1) = &location_fleets[idx1];
                let (e2, _, f2, c2, a2) = &location_fleets[idx2];

                if *a1 || *a2 {
                    // One of the fleets is avoiding combat
                    i = group_end;
                    continue;
                }

                // Resolve Combat
                // Assume first one found is "attacker" (arbitrary)
                let result = resolve_combat(*f1, c1, *f2, c2);

                // Apply results
                let (winner_entity, loser_entity) = if result.winner == *f1 {
                    (*e1, *e2)
                } else {
                    (*e2, *e1)
                };

                // Emit destroyed events
                for ship in &result.destroyed_ships {
                    event_writer.send(ShipDestroyedEvent {
                        planet: current_location,
                        ship_class: format!("{ship:?}"),
                    });
                }

                // Update Winner
                if result.winner_survivors.ships.is_empty() {
                    commands.entity(winner_entity).despawn(); // Mutual destruction?
                } else {
                    commands
                        .entity(winner_entity)
                        .insert(result.winner_survivors);
                }

                // Update Loser
                let loser_survivor_count = result.loser_survivors.ships.len();

                if result.loser_survivors.ships.is_empty() {
                    commands.entity(loser_entity).despawn();
                    // Spawn Debris/Loot
                    if let Some(_loot) = result.loot {
                        // For MVP: Log it or spawn a loot entity.
                        // Since Spec 184 (Orbital Debris) is Backlog, we just log or ignore.
                        // Or we can spawn a generic "Loot" marker if we had one.
                        // I'll just leave a comment/TODO as per spec instructions for MVP.
                        // "Debris: Spawn OrbitalDebris entity (Spec 184) containing the loot."
                    }
                } else {
                    commands.entity(loser_entity).insert(result.loser_survivors);
                }

                println!(
                    "Combat at {:?}! Winner: {:?}, Loser Survivors: {}",
                    current_location, result.winner, loser_survivor_count
                );
            }
        }

        i = group_end;
    }
}

#[cfg(test)]
mod tests {
    use crate::layer2::combat::resolve_combat;
    use crate::layer2::fleet::{FleetComposition, FleetFaction};
    use crate::layer2::ship::{Ship, ShipType};

    // Helper to create a test fleet
    fn create_fleet(_faction: FleetFaction, ships: Vec<ShipType>) -> FleetComposition {
        let mut comp = FleetComposition::default();
        for t in ships {
            comp.add_ship(Ship::new(t));
        }
        comp
    }

    #[test]
    fn test_combat_resolution_stronger_wins() {
        // Fleet A: 10 Frigates (High Combat)
        let fleet_a = create_fleet(FleetFaction::Player, vec![ShipType::Frigate; 10]);

        // Fleet B: 1 Scout (Low Combat)
        let fleet_b = create_fleet(FleetFaction::Pirate, vec![ShipType::Scout; 1]);

        // Resolve
        let result = resolve_combat(
            FleetFaction::Player,
            &fleet_a,
            FleetFaction::Pirate,
            &fleet_b,
        );

        // A should win
        assert_eq!(result.winner, FleetFaction::Player);
        // B should be wiped out (empty list of survivors)
        assert!(
            result.loser_survivors.ships.is_empty(),
            "Loser should have 0 survivors"
        );
        // A should take minimal/no damage (just checking count for now)
        // 10 Frigates vs 1 Scout. Scout power = 2. Frigate power = 50. Total A=500, B=2.
        // Winner damage = 2 * 0.5 = 1.0. Frigate HP = 100. No ships lost.
        assert_eq!(
            result.winner_survivors.ships.len(),
            10,
            "Winner should keep ships"
        );
    }

    #[test]
    fn test_combat_casualties() {
        // Fleet A: 5 Frigates
        let fleet_a = create_fleet(FleetFaction::Player, vec![ShipType::Frigate; 5]);

        // Fleet B: 5 Frigates
        let fleet_b = create_fleet(FleetFaction::Pirate, vec![ShipType::Frigate; 5]);

        // Even fight, both sides should take losses
        let result = resolve_combat(
            FleetFaction::Player,
            &fleet_a,
            FleetFaction::Pirate,
            &fleet_b,
        );

        // Winner Power: 250. Loser Power: 250.
        // Winner Damage: 125. (1.25 Frigates lost) -> 4 survivors (1 damaged).
        // Loser Damage: 375. (3.75 Frigates lost) -> 2 survivors (1 damaged).

        let total_survivors =
            result.winner_survivors.ships.len() + result.loser_survivors.ships.len();
        assert!(
            total_survivors < 10,
            "Casualties should occur in even fight"
        );
    }

    #[test]
    fn test_loot_generation() {
        let fleet_a = create_fleet(FleetFaction::Player, vec![ShipType::Frigate; 10]);
        let fleet_b = create_fleet(FleetFaction::Pirate, vec![ShipType::Transport; 1]); // Transport has high cargo

        let result = resolve_combat(
            FleetFaction::Player,
            &fleet_a,
            FleetFaction::Pirate,
            &fleet_b,
        );

        // If Transport destroyed, loot generated
        assert!(
            result.loot.is_some(),
            "Loot should be generated when transport is destroyed"
        );
    }

    #[test]
    fn test_destroyed_ships_tracking() {
        let fleet_a = create_fleet(FleetFaction::Player, vec![ShipType::Frigate; 10]);
        let fleet_b = create_fleet(FleetFaction::Pirate, vec![ShipType::Scout; 1]);

        let result = resolve_combat(
            FleetFaction::Player,
            &fleet_a,
            FleetFaction::Pirate,
            &fleet_b,
        );

        // B's scout should be in destroyed list
        assert!(
            !result.destroyed_ships.is_empty(),
            "Destroyed ships should be tracked"
        );
        assert!(
            result.destroyed_ships.contains(&ShipType::Scout),
            "Scout should be destroyed"
        );
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct AvoidCombat;
