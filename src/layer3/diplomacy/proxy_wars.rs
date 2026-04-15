use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Credits(pub u32);

#[derive(Component)]
pub struct GlobalPirateBounty(pub u32);

#[derive(Component, Default)]
pub struct ThreatMap {
    pub threats: bevy::utils::HashMap<Entity, u32>,
}

impl ThreatMap {
    #[must_use]
    pub fn get_threat(&self, target: Entity) -> u32 {
        self.threats.get(&target).copied().unwrap_or(0)
    }
}

pub struct ProxyWarContract {
    pub sponsor: Entity,
    pub target: Entity,
    pub reward: u32,
}

#[derive(Event)]
pub struct AcceptContractEvent {
    pub player: Entity,
    pub contract: ProxyWarContract,
}

#[derive(Component, Default)]
pub struct PrivateerStatus {
    // Maps Target Faction -> List of Sponsors
    pub contracts: bevy::utils::HashMap<Entity, Vec<Entity>>,
}

impl PrivateerStatus {
    #[must_use]
    pub fn new(sponsor: Entity, target: Entity) -> Self {
        let mut contracts = bevy::utils::HashMap::new();
        contracts.insert(target, vec![sponsor]);
        Self { contracts }
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        !self.contracts.is_empty()
    }

    #[must_use]
    pub fn has_contract_against(&self, target: Entity) -> bool {
        self.contracts.contains_key(&target) && !self.contracts.get(&target).unwrap().is_empty()
    }
}

#[derive(Event)]
pub struct PeaceTreatyEvent {
    pub faction1: Entity,
    pub faction2: Entity,
}

#[allow(clippy::type_complexity)]
pub fn accept_proxy_war_contract_system(
    mut events: EventReader<AcceptContractEvent>,
    mut players: Query<(&mut Credits, &mut ThreatMap, Option<&mut PrivateerStatus>)>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok((mut credits, mut threat_map, privateer_status)) = players.get_mut(event.player) {
            credits.0 += event.contract.reward;
            let current_threat = threat_map.threats.entry(event.contract.target).or_insert(0);
            *current_threat += 10;

            if let Some(mut status) = privateer_status {
                let sponsors = status.contracts.entry(event.contract.target).or_default();
                if !sponsors.contains(&event.contract.sponsor) {
                    sponsors.push(event.contract.sponsor);
                }
            } else {
                commands.entity(event.player).insert(PrivateerStatus::new(
                    event.contract.sponsor,
                    event.contract.target,
                ));
            }
        }
    }
}

#[derive(Event)]
pub struct ProxyWarShipDestroyedEvent {
    pub destroyer: Entity,
    pub destroyed: Entity,
}

#[derive(Component)]
pub struct BelongsTo(pub Entity);

#[derive(Component)]
pub struct Ship;

pub fn proxy_war_combat_rewards_system(
    mut events: EventReader<ProxyWarShipDestroyedEvent>,
    mut players: Query<(&mut Credits, Option<&PrivateerStatus>, &mut GlobalPirateBounty)>,
    targets: Query<&BelongsTo>,
) {
    for event in events.read() {
        if let Ok((mut credits, status_opt, mut bounty)) = players.get_mut(event.destroyer) {
            if let Ok(belongs_to) = targets.get(event.destroyed) {
                let has_contract = status_opt.map_or(false, |s| s.has_contract_against(belongs_to.0));

                if has_contract {
                    credits.0 += 100; // bounty reward
                } else {
                    bounty.0 += 500; // pirate penalty
                }
            }
        }
    }
}

pub fn peace_treaty_disavowal_system(
    mut events: EventReader<PeaceTreatyEvent>,
    mut privateers: Query<&mut PrivateerStatus>,
) {
    for event in events.read() {
        for mut status in privateers.iter_mut() {
            // If Faction1 is target, remove Faction2 as sponsor, and vice versa
            if let Some(sponsors) = status.contracts.get_mut(&event.faction1) {
                sponsors.retain(|&s| s != event.faction2);
            }
            if let Some(sponsors) = status.contracts.get_mut(&event.faction2) {
                sponsors.retain(|&s| s != event.faction1);
            }

            // Clean up empty contracts
            status.contracts.retain(|_, sponsors| !sponsors.is_empty());
        }
    }
}

pub struct ProxyWarsPlugin;

impl Plugin for ProxyWarsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AcceptContractEvent>();
        app.add_event::<ProxyWarShipDestroyedEvent>();
        app.add_event::<PeaceTreatyEvent>();

        app.add_systems(Update, (
            accept_proxy_war_contract_system,
            proxy_war_combat_rewards_system,
            peace_treaty_disavowal_system,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer3::diplomacy::succession::Faction;

    #[test]
    fn test_proxy_war_accepting_contract() {
        let mut app = bevy::app::App::new();
        app.add_plugins(ProxyWarsPlugin);

        // Arrange: Spawn Faction A (Sponsor) and Faction B (Target), offer a Proxy War contract
        let faction_a = app.world_mut().spawn(Faction { name: "Faction A".to_string() }).id();
        let faction_b = app.world_mut().spawn(Faction { name: "Faction B".to_string() }).id();
        let player = app.world_mut().spawn((Player, Credits(0), ThreatMap::default())).id();
        let contract = ProxyWarContract { sponsor: faction_a, target: faction_b, reward: 1000 };

        // Act: Player accepts the contract
        app.world_mut().send_event(AcceptContractEvent { player, contract });
        app.update();

        // Assert: Verify Player gains "Privateer" status against Faction B, receives upfront Credits, and Threat increases with Faction B
        assert!(app.world().get::<PrivateerStatus>(player).unwrap().has_contract_against(faction_b));
        assert_eq!(app.world().get::<Credits>(player).unwrap().0, 1000);
        assert!(app.world().get::<ThreatMap>(player).unwrap().get_threat(faction_b) > 0);
    }

    #[test]
    fn test_proxy_war_combat_rewards() {
        let mut app = bevy::app::App::new();
        app.add_plugins(ProxyWarsPlugin);

        // Arrange: Player has Privateer status against Faction B
        let faction_a = app.world_mut().spawn(Faction { name: "Faction A".to_string() }).id();
        let faction_b = app.world_mut().spawn(Faction { name: "Faction B".to_string() }).id();
        let player = app.world_mut().spawn((Player, Credits(0), PrivateerStatus::new(faction_a, faction_b), GlobalPirateBounty(0))).id();
        let enemy_ship = app.world_mut().spawn((Ship, BelongsTo(faction_b))).id();

        // Act: Player fleet destroys a Faction B ship
        app.world_mut().send_event(ProxyWarShipDestroyedEvent { destroyer: player, destroyed: enemy_ship });
        app.update();

        // Assert: Verify Player receives bounty Credits from Faction A without incurring global Pirate penalties
        assert!(app.world().get::<Credits>(player).unwrap().0 > 0);
        assert_eq!(app.world().get::<GlobalPirateBounty>(player).unwrap().0, 0);
    }

    #[test]
    fn test_proxy_war_combat_without_contract_is_piracy() {
        let mut app = bevy::app::App::new();
        app.add_plugins(ProxyWarsPlugin);

        // Arrange: Player has NO Privateer status against Faction B
        let faction_b = app.world_mut().spawn(Faction { name: "Faction B".to_string() }).id();
        let player = app.world_mut().spawn((Player, Credits(0), GlobalPirateBounty(0))).id();
        let enemy_ship = app.world_mut().spawn((Ship, BelongsTo(faction_b))).id();

        // Act: Player fleet destroys a Faction B ship
        app.world_mut().send_event(ProxyWarShipDestroyedEvent { destroyer: player, destroyed: enemy_ship });
        app.update();

        // Assert: Verify Player receives Pirate penalties and NO credits
        assert_eq!(app.world().get::<Credits>(player).unwrap().0, 0);
        assert!(app.world().get::<GlobalPirateBounty>(player).unwrap().0 > 0);
    }

    #[test]
    fn test_proxy_war_disavowal() {
        let mut app = bevy::app::App::new();
        app.add_plugins(ProxyWarsPlugin);

        // Arrange: Player has Privateer status for Faction A against Faction B
        let faction_a = app.world_mut().spawn(Faction { name: "Faction A".to_string() }).id();
        let faction_b = app.world_mut().spawn(Faction { name: "Faction B".to_string() }).id();
        let player = app.world_mut().spawn((Player, PrivateerStatus::new(faction_a, faction_b), GlobalPirateBounty(0), Credits(0))).id();

        // Act: Faction A and Faction B sign a peace treaty
        app.world_mut().send_event(PeaceTreatyEvent { faction1: faction_a, faction2: faction_b });
        app.update();

        // Assert: Verify Player loses Privateer status
        assert!(!app.world().get::<PrivateerStatus>(player).unwrap().is_active());

        // Act: Player fleet destroys a Faction B ship AFTER disavowal
        let enemy_ship = app.world_mut().spawn((Ship, BelongsTo(faction_b))).id();
        app.world_mut().send_event(ProxyWarShipDestroyedEvent { destroyer: player, destroyed: enemy_ship });
        app.update();

        // Assert: Verify Player receives Pirate penalties (branded as "Pirate")
        assert!(app.world().get::<GlobalPirateBounty>(player).unwrap().0 > 0);
    }
}
