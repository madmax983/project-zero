//! Proxy Wars and Privateers
//!
//! Handles the acceptance of proxy war contracts, combat rewards for privateers,
//! and the disavowal of privateers when peace treaties are signed.
use bevy::prelude::*;
use bevy::utils::HashSet;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct Credits(pub i32);

/// Represents the perceived threat levels of other factions.
///
/// Threat is typically increased when a faction engages in hostile actions,
/// accepts proxy war contracts, or is caught engaging in unauthorized activities.
#[derive(Resource, Component, Default)]
pub struct ThreatMap {
    /// A map of faction entities to their perceived threat value.
    pub threats: bevy::utils::HashMap<Entity, i32>,
    /// A global modifier applied to all threat interactions.
    pub global_threat_modifier: f32,
}

impl ThreatMap {
    /// Returns the threat value for a specific faction, or `0` if no threat is recorded.
    ///
    /// # Examples
    /// ```
    /// use bevy::prelude::*;
    /// use scale::layer3::diplomacy::proxy_wars::ThreatMap;
    ///
    /// let mut map = ThreatMap::default();
    /// let faction = Entity::from_raw(1);
    ///
    /// // Threat defaults to 0
    /// assert_eq!(map.get_threat(faction), 0);
    ///
    /// // Modifying threat
    /// map.threats.insert(faction, 50);
    /// assert_eq!(map.get_threat(faction), 50);
    /// ```
    pub fn get_threat(&self, faction: Entity) -> i32 {
        *self.threats.get(&faction).unwrap_or(&0)
    }
}

/// A status granted to a player or faction acting as a privateer for a sponsor.
///
/// Privateers earn credits for attacking specific targets designated by their sponsor,
/// rather than accruing a global pirate bounty.
#[derive(Component)]
pub struct PrivateerStatus {
    /// The faction sponsoring the privateer.
    pub sponsor: Entity,
    /// The target factions the privateer is authorized to attack.
    pub targets: HashSet<Entity>,
}

impl PrivateerStatus {
    /// Creates a new `PrivateerStatus` for a specific sponsor and target.
    ///
    /// # Examples
    /// ```
    /// use bevy::prelude::*;
    /// use scale::layer3::diplomacy::proxy_wars::PrivateerStatus;
    ///
    /// let sponsor = Entity::from_raw(1);
    /// let target = Entity::from_raw(2);
    ///
    /// let status = PrivateerStatus::new(sponsor, target);
    /// assert!(status.is_active());
    /// assert!(status.targets.contains(&target));
    /// ```
    pub fn new(sponsor: Entity, target: Entity) -> Self {
        let mut targets = HashSet::new();
        targets.insert(target);
        Self { sponsor, targets }
    }

    /// Returns `true` if the privateer has active targets.
    pub fn is_active(&self) -> bool {
        !self.targets.is_empty()
    }
}

#[derive(Component)]
pub struct GlobalPirateBounty(pub i32);

#[derive(Component)]
pub struct BelongsTo(pub Entity);

pub struct ProxyWarContract {
    pub sponsor: Entity,
    pub target: Entity,
    pub reward: i32,
}

#[derive(Event)]
pub struct AcceptContractEvent {
    pub player: Entity,
    pub contract: ProxyWarContract,
}

#[derive(Event)]
pub struct PeaceTreatyEvent {
    pub faction1: Entity,
    pub faction2: Entity,
}

#[derive(Event)]
pub struct ShipDestroyedEvent {
    pub destroyer: Entity,
    pub destroyed: Entity,
}

#[allow(clippy::type_complexity)]
pub fn accept_proxy_war_contract_system(
    mut commands: Commands,
    mut events: EventReader<AcceptContractEvent>,
    mut query: Query<(
        Entity,
        &mut Credits,
        &mut ThreatMap,
        Option<&mut PrivateerStatus>,
    )>,
) {
    for event in events.read() {
        if let Ok((entity, mut credits, mut threat_map, privateer_status_opt)) =
            query.get_mut(event.player)
        {
            if let Some(mut status) = privateer_status_opt {
                status.targets.insert(event.contract.target);
            } else {
                commands.entity(entity).insert(PrivateerStatus::new(
                    event.contract.sponsor,
                    event.contract.target,
                ));
            }
            credits.0 += event.contract.reward;
            let current_threat = threat_map.get_threat(event.contract.target);
            threat_map
                .threats
                .insert(event.contract.target, current_threat + 50);
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn proxy_war_combat_rewards_system(
    mut events: EventReader<ShipDestroyedEvent>,
    mut player_query: Query<(
        Option<&PrivateerStatus>,
        &mut Credits,
        &mut GlobalPirateBounty,
    )>,
    ship_query: Query<&BelongsTo>,
) {
    for event in events.read() {
        if let Ok((privateer_status_opt, mut credits, mut bounty)) =
            player_query.get_mut(event.destroyer)
        {
            if let Ok(belongs_to) = ship_query.get(event.destroyed) {
                let is_target = privateer_status_opt
                    .is_some_and(|status| status.targets.contains(&belongs_to.0));
                if is_target {
                    credits.0 += 500;
                } else {
                    bounty.0 += 100;
                }
            }
        }
    }
}

pub fn proxy_war_disavowal_system(
    mut commands: Commands,
    mut events: EventReader<PeaceTreatyEvent>,
    mut player_query: Query<(Entity, &mut PrivateerStatus)>,
) {
    for event in events.read() {
        for (entity, mut privateer_status) in player_query.iter_mut() {
            if privateer_status.sponsor == event.faction1
                && privateer_status.targets.contains(&event.faction2)
            {
                privateer_status.targets.remove(&event.faction2);
            } else if privateer_status.sponsor == event.faction2
                && privateer_status.targets.contains(&event.faction1)
            {
                privateer_status.targets.remove(&event.faction1);
            }
            if !privateer_status.is_active() {
                commands.entity(entity).remove::<PrivateerStatus>();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer3::diplomacy::succession::Faction;

    #[test]
    fn test_proxy_war_accepting_contract() {
        let mut app = bevy::app::App::new();
        app.add_event::<AcceptContractEvent>();
        app.add_systems(Update, accept_proxy_war_contract_system);

        // Arrange: Spawn Faction A (Sponsor) and Faction B (Target), offer a Proxy War contract
        let faction_a = app
            .world_mut()
            .spawn(Faction {
                name: "Faction A".to_string(),
            })
            .id();
        let faction_b = app
            .world_mut()
            .spawn(Faction {
                name: "Faction B".to_string(),
            })
            .id();
        let player = app
            .world_mut()
            .spawn((Player, Credits(0), ThreatMap::default()))
            .id();
        let contract = ProxyWarContract {
            sponsor: faction_a,
            target: faction_b,
            reward: 1000,
        };

        // Act: Player accepts the contract
        app.world_mut()
            .send_event(AcceptContractEvent { player, contract });
        app.update();

        // Assert: Verify Player gains "Privateer" status against Faction B, receives upfront Credits, and Threat increases with Faction B
        assert!(app
            .world()
            .get::<PrivateerStatus>(player)
            .unwrap()
            .targets
            .contains(&faction_b));
        assert_eq!(app.world().get::<Credits>(player).unwrap().0, 1000);
        assert!(
            app.world()
                .get::<ThreatMap>(player)
                .unwrap()
                .get_threat(faction_b)
                > 0
        );
    }

    #[test]
    fn test_proxy_war_combat_rewards() {
        let mut app = bevy::app::App::new();
        app.add_event::<ShipDestroyedEvent>();
        app.add_systems(Update, proxy_war_combat_rewards_system);

        // Arrange: Player has Privateer status against Faction B
        let faction_a = app
            .world_mut()
            .spawn(Faction {
                name: "Faction A".to_string(),
            })
            .id();
        let faction_b = app
            .world_mut()
            .spawn(Faction {
                name: "Faction B".to_string(),
            })
            .id();
        let player = app
            .world_mut()
            .spawn((
                Player,
                Credits(0),
                PrivateerStatus::new(faction_a, faction_b),
                GlobalPirateBounty(0),
            ))
            .id();
        let enemy_ship = app.world_mut().spawn((Ship, BelongsTo(faction_b))).id();

        // Act: Player fleet destroys a Faction B ship
        app.world_mut().send_event(ShipDestroyedEvent {
            destroyer: player,
            destroyed: enemy_ship,
        });
        app.update();

        // Assert: Verify Player receives bounty Credits from Faction A without incurring global Pirate penalties
        assert!(app.world().get::<Credits>(player).unwrap().0 > 0);
        assert_eq!(app.world().get::<GlobalPirateBounty>(player).unwrap().0, 0);
    }

    #[test]
    fn test_proxy_war_disavowal() {
        let mut app = bevy::app::App::new();
        app.add_event::<PeaceTreatyEvent>();
        app.add_systems(Update, proxy_war_disavowal_system);

        // Arrange: Player has Privateer status for Faction A against Faction B
        let faction_a = app
            .world_mut()
            .spawn(Faction {
                name: "Faction A".to_string(),
            })
            .id();
        let faction_b = app
            .world_mut()
            .spawn(Faction {
                name: "Faction B".to_string(),
            })
            .id();
        let player = app
            .world_mut()
            .spawn((Player, PrivateerStatus::new(faction_a, faction_b)))
            .id();

        // Act: Faction A and Faction B sign a peace treaty
        app.world_mut().send_event(PeaceTreatyEvent {
            faction1: faction_a,
            faction2: faction_b,
        });
        app.update();

        // Assert: Verify Player loses Privateer status and is branded as a "Pirate" if they continue hostilities
        assert!(
            app.world().get::<PrivateerStatus>(player).is_none()
                || !app
                    .world()
                    .get::<PrivateerStatus>(player)
                    .unwrap()
                    .is_active()
        );
    }
}
