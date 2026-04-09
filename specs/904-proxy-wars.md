# Proxy Wars

## 1. Overview
Being a pawn (or a knight) in a galactic game. Large Civilizations on Layer 3 will pay you to attack their enemies without declaring war themselves. By accepting, you gain "Privateer" status (legal piracy) and Credits, but generate "Threat" with the targeted faction. It provides easy money through mercenary work but compromises diplomatic independence. If the backing civilization signs a peace treaty or trade deal with the target, they may disavow you, instantly stripping your Privateer status and leaving you branded as a common pirate.

## 2. Dependencies
- Layer 3 Diplomacy and Civilizations
- Fleet Combat and Piracy Systems
- Economy and Credits

## 3. RED Phase: Tests First
```rust
#[test]
fn test_proxy_war_accepting_contract() {
    let mut app = bevy::app::App::new();

    // Arrange: Spawn Faction A (Sponsor) and Faction B (Target), offer a Proxy War contract
    let faction_a = app.world_mut().spawn(Faction { name: "Faction A".to_string() }).id();
    let faction_b = app.world_mut().spawn(Faction { name: "Faction B".to_string() }).id();
    let player = app.world_mut().spawn((Player, Credits(0), ThreatMap::default())).id();
    let contract = ProxyWarContract { sponsor: faction_a, target: faction_b, reward: 1000 };

    // Act: Player accepts the contract
    app.world_mut().send_event(AcceptContractEvent { player, contract });
    app.update();

    // Assert: Verify Player gains "Privateer" status against Faction B, receives upfront Credits, and Threat increases with Faction B
    assert!(app.world().get::<PrivateerStatus>(player).unwrap().targets.contains(&faction_b));
    assert_eq!(app.world().get::<Credits>(player).unwrap().0, 1000);
    assert!(app.world().get::<ThreatMap>(player).unwrap().get_threat(faction_b) > 0);
}

#[test]
fn test_proxy_war_combat_rewards() {
    let mut app = bevy::app::App::new();

    // Arrange: Player has Privateer status against Faction B
    let faction_a = app.world_mut().spawn(Faction { name: "Faction A".to_string() }).id();
    let faction_b = app.world_mut().spawn(Faction { name: "Faction B".to_string() }).id();
    let player = app.world_mut().spawn((Player, Credits(0), PrivateerStatus::new(faction_a, faction_b), GlobalPirateBounty(0))).id();
    let enemy_ship = app.world_mut().spawn((Ship, BelongsTo(faction_b))).id();

    // Act: Player fleet destroys a Faction B ship
    app.world_mut().send_event(ShipDestroyedEvent { destroyer: player, destroyed: enemy_ship });
    app.update();

    // Assert: Verify Player receives bounty Credits from Faction A without incurring global Pirate penalties
    assert!(app.world().get::<Credits>(player).unwrap().0 > 0);
    assert_eq!(app.world().get::<GlobalPirateBounty>(player).unwrap().0, 0);
}

#[test]
fn test_proxy_war_disavowal() {
    let mut app = bevy::app::App::new();

    // Arrange: Player has Privateer status for Faction A against Faction B
    let faction_a = app.world_mut().spawn(Faction { name: "Faction A".to_string() }).id();
    let faction_b = app.world_mut().spawn(Faction { name: "Faction B".to_string() }).id();
    let player = app.world_mut().spawn((Player, PrivateerStatus::new(faction_a, faction_b))).id();

    // Act: Faction A and Faction B sign a peace treaty
    app.world_mut().send_event(PeaceTreatyEvent { faction1: faction_a, faction2: faction_b });
    app.update();

    // Assert: Verify Player loses Privateer status and is branded as a "Pirate" if they continue hostilities
    assert!(!app.world().get::<PrivateerStatus>(player).unwrap().is_active());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal implementation for Proxy War contracts, Privateer status toggling, and bounty payouts.
```

## 5. REFACTOR Phase: Quality & Design
- Expand the diplomatic standing structs to cleanly track targeted hostilities versus global piracy.
- Ensure the event system properly fires `DisavowalEvent` to notify the player and update their standing UI.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Track "Privateer" as a directional relationship (Player -> Target Faction) validated by a Sponsor Faction. If the Sponsor's stance towards the Target becomes neutral or allied, the Privateer status should automatically expire.

## 8. Questions
*Builder: add questions here if spec is unclear.*
