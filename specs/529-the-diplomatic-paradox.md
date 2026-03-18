# 529 The Diplomatic Paradox

## 1. Overview
**Layer:** 3
**Fantasy:** You receive an alliance proposal from an empire you haven't met yet, from the future.
**Mechanic:** A temporal anomaly allows a Layer 3 empire to interact with you *before* the formal first contact. They might offer a trade deal or demand tribute based on events that haven't happened yet in your timeline.
**Emergence:** They declare war on you for an atrocity you haven't committed yet. To survive the war, you are forced to commit the atrocity to gain an advantage, fulfilling the temporal loop.
**Tension:** Trying to decipher the future from erratic diplomatic messages vs. ignoring them and flying blind.

## 2. Dependencies
- `039 Trade System` (Layer 1/2/3 economy base)
- `469 The Galactic Council` (Layer 3 diplomacy)
- `292 The Chrono-Stutter` (Temporal mechanic basics - optional but thematic)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_temporal_contact_event_triggers() {
    // Arrange: Setup world with no known Layer 3 contacts
    let mut app = setup_world();

    // Act: Force trigger a TemporalDiplomaticEvent
    app.world_mut().send_event(TemporalDiplomaticEvent {
        faction_id: Entity::PLACEHOLDER, // An unknown faction
        message_type: TemporalMessage::DemandTribute,
        referenced_future_event: FutureEvent::Atrocity(Entity::PLACEHOLDER),
    });
    app.update();

    // Assert: The player receives a diplomatic notification from an "Unknown Future Entity"
    let notifications = app.world().resource::<Notifications>();
    assert!(notifications.iter().any(|n| n.title.contains("Temporal Transmission")));
}

#[test]
fn test_fulfilling_temporal_paradox_grants_bonus() {
    // Arrange: Player has an active TemporalDemand (e.g., "Commit Atrocity X to survive War Y")
    let mut app = setup_world();
    let player_faction = spawn_player_faction(&mut app);
    let paradox_id = app.world_mut().spawn(TemporalParadoxTracker {
        condition: ParadoxCondition::CommitAtrocity(AtrocityType::OrbitalBombardment),
        reward: ParadoxReward::AvoidWar,
        status: ParadoxStatus::Active,
    }).id();

    // Act: Player commits the required atrocity
    app.world_mut().send_event(AtrocityCommittedEvent {
        faction: player_faction,
        atrocity_type: AtrocityType::OrbitalBombardment,
    });
    app.update();

    // Assert: The paradox is fulfilled and the reward (avoiding war) is applied
    let tracker = app.world().get::<TemporalParadoxTracker>(paradox_id).unwrap();
    assert_eq!(tracker.status, ParadoxStatus::Fulfilled);

    // Check that the war declaration event was canceled/prevented
    let war_status = app.world().resource::<DiplomaticStatus>();
    assert!(!war_status.is_at_war_with(Entity::PLACEHOLDER));
}

#[test]
fn test_ignoring_temporal_paradox_causes_consequences() {
    // Arrange: Player has an active TemporalDemand
    let mut app = setup_world();
    let paradox_id = app.world_mut().spawn(TemporalParadoxTracker {
        condition: ParadoxCondition::ProvideTribute(ResourceType::Energy, 5000),
        penalty: ParadoxPenalty::ImmediateWar,
        status: ParadoxStatus::Active,
        time_limit: 100, // ticks
    }).id();

    // Act: Time passes without fulfilling the condition
    for _ in 0..101 {
        app.update(); // Advance time
    }

    // Assert: The paradox fails and the penalty is applied
    let tracker = app.world().get::<TemporalParadoxTracker>(paradox_id).unwrap();
    assert_eq!(tracker.status, ParadoxStatus::Failed);

    // Check that the penalty (ImmediateWar) triggered
    let war_events = app.world().resource::<Events<WarDeclarationEvent>>();
    assert!(war_events.get_reader().len() > 0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// In src/layer3/diplomacy/temporal.rs

#[derive(Event)]
pub struct TemporalDiplomaticEvent {
    pub faction_id: Entity,
    pub message_type: TemporalMessage,
    pub referenced_future_event: FutureEvent,
}

#[derive(Component)]
pub struct TemporalParadoxTracker {
    pub condition: ParadoxCondition,
    pub reward: ParadoxReward,
    pub penalty: ParadoxPenalty,
    pub status: ParadoxStatus,
    pub time_limit: u32,
}

pub fn process_temporal_diplomacy_system(
    mut events: EventReader<TemporalDiplomaticEvent>,
    mut commands: Commands,
    mut notifications: ResMut<Notifications>,
) {
    for event in events.read() {
        notifications.push(Notification::new("Temporal Transmission", "We received a message from the future..."));

        commands.spawn(TemporalParadoxTracker {
            condition: ParadoxCondition::None, // Placeholder
            reward: ParadoxReward::None,
            penalty: ParadoxPenalty::None,
            status: ParadoxStatus::Active,
            time_limit: 5000,
        });
    }
}

pub fn evaluate_temporal_paradox_system(
    mut query: Query<(Entity, &mut TemporalParadoxTracker)>,
    // mut atrocity_events: EventReader<AtrocityCommittedEvent>, // Listen for triggers
    mut commands: Commands,
) {
    for (entity, mut tracker) in query.iter_mut() {
        if tracker.time_limit == 0 {
            tracker.status = ParadoxStatus::Failed;
            // Apply penalty...
        } else {
            tracker.time_limit -= 1;
            // Check if condition met...
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration:** Hook the `TemporalDiplomaticEvent` into the `ChronicleSystem` to generate lore entries about cryptic messages from the void.
- **UI:** The Tech Tree or Diplomacy UI needs a way to display "Unknown Factions" and their paradoxical demands clearly so the player understands the stakes.
- **Performance:** Tracking multiple active paradoxes should be cheap, but checking conditions (like "did we commit an atrocity?") should rely on event listeners rather than polling the whole world state every tick.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer3/diplomacy/temporal.rs`.
- [ ] A temporal message generates a notification.
- [ ] Fulfilling or failing a paradox condition applies the correct reward or penalty.

## 7. Technical Guidance
- Ensure that the "Unknown Faction" placeholder in the UI doesn't break existing diplomacy screens that expect a fully realized `Civilization` entity with a name and flag.
- Use the `EventImportance::Major` flag when recording these events in the chronicle.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
