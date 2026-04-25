# 1161: The Blind Auction

## Overview
An enigmatic merchant fleet arrives offering a "Sealed Precursor Vault" to the highest bidder in the sector. You must bid large amounts of Layer 1 resources (alloys, food, energy) without knowing the contents. It could contain game-winning endgame tech, or it could unleash a devastating memetic virus or host of hostile nano-drones directly into your capital. The tension is the immense FOMO of letting a rival empire potentially win a massive technological leap, versus the catastrophic risk of buying a trojan horse with your own survival resources.

## Dependencies
- Layer 1 Resource Management
- Layer 2/3 Event System
- Trade/Bidding UI mechanisms

## RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_blind_auction_event_generation() {
    // Arrange: Setup simulation state
    // Act: Trigger blind auction event
    // Assert: A `BlindAuction` entity is spawned with a sealed vault target
}

#[test]
fn test_blind_auction_bidding() {
    // Arrange: A `BlindAuction` exists, colony has enough resources
    // Act: Submit a bid for the vault
    // Assert: The resources are deducted and the bid is recorded
}

#[test]
fn test_blind_auction_resolution_positive() {
    // Arrange: The player wins the auction containing positive tech
    // Act: Resolve the auction
    // Assert: The player gains the tech and a chronicle event is logged
}

#[test]
fn test_blind_auction_resolution_negative() {
    // Arrange: The player wins the auction containing a disaster
    // Act: Resolve the auction
    // Assert: The disaster (e.g. memetic virus/drone swarm) is spawned at the colony
}
```

## GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN

pub fn generate_blind_auction_system(
    mut commands: Commands,
    mut event_reader: EventReader<TriggerBlindAuction>,
) {
    for _ in event_reader.read() {
        commands.spawn(BlindAuction {
            vault_contents: VaultContents::random(),
            current_highest_bid: 0,
            winning_faction: None,
            time_remaining: 10.0,
        });
    }
}

pub fn process_bids_system(
    mut auction_query: Query<&mut BlindAuction>,
    mut bid_reader: EventReader<SubmitBid>,
    mut resources: Query<&mut ColonyResources>,
) {
    for mut auction in auction_query.iter_mut() {
        for bid in bid_reader.read() {
            if bid.amount > auction.current_highest_bid {
                if let Ok(mut res) = resources.get_mut(bid.colony) {
                    if res.deduct(bid.resource_type, bid.amount) {
                        auction.current_highest_bid = bid.amount;
                        auction.winning_faction = Some(bid.faction);
                    }
                }
            }
        }
    }
}

pub fn resolve_auction_system(
    mut commands: Commands,
    mut auction_query: Query<(Entity, &BlindAuction)>,
    time: Res<Time>,
) {
    for (entity, auction) in auction_query.iter_mut() {
        if auction.time_remaining <= 0.0 {
            if let Some(winner) = auction.winning_faction {
                // Assign reward or disaster to winner
                commands.spawn(VaultOpened {
                    faction: winner,
                    contents: auction.vault_contents.clone(),
                });
            }
            commands.entity(entity).despawn();
        }
    }
}
```

## REFACTOR Phase: Quality & Design
- **Code Smell:** Randomizing vault contents directly in the spawn function makes it hard to test specific outcomes. Abstract this behind a trait or inject a seeded RNG for tests.
- **Optimization:** Avoid checking all auctions every frame if they aren't active.
- **Integration:** Ensure the vault contents tie into the existing tech tree and disaster systems seamlessly. Add a Chronicle event when the vault is opened.
- **Refactoring:** The bidding logic should be more robust, refunding previous bidders when they are outbid.

## Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code in `src/layer2/events/blind_auction.rs`
- [ ] Vault opening generates appropriate Chronicle logs depending on the outcome.

## Technical Guidance
- **Components:** Create `BlindAuction`, `SubmitBid` (Event), and `VaultOpened` (Event).
- **Enums:** `VaultContents` should have variants like `Tech(TechId)`, `Disaster(DisasterType)`, `Resources(Amount)`.
- **System Placement:** Ensure `process_bids_system` runs before `resolve_auction_system` in the schedule.

## Questions
*Builder: add questions here if spec is unclear. Architect will address.*
