# The Gossip Economy

## 1. Overview
Secrets are currency, and a well-timed rumor is worth more than gold. Pops can generate "Intel" tokens by participating in the Rumor Web or working in administrative jobs. These tokens can be spent at a black-market "Broker" for rare resources or to instantly improve relations with hostile factions. However, generating Intel requires Pops to spend time gossiping instead of working, and risks spreading massive negative Morale debuffs if they uncover the colony's dark secrets.

## 2. Dependencies
- `055` The Rumor Web
- `031` Pop Morale
- `348` The Black Market

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::social::rumor::{Rumor, GossipEvent};
    use crate::layer1::social::morale::Morale;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<GossipEvent>();
        app.add_systems(Update, (process_gossip, spend_intel));
        app.insert_resource(IntelTokens(0));
        app
    }

    #[test]
    fn test_gossiping_generates_intel_tokens() {
        // Arrange
        let mut app = setup_app();
        let pop = app.world_mut().spawn((Pop::default(),)).id();
        app.world_mut().resource_mut::<Events<GossipEvent>>().send(GossipEvent { pop, rumor: Rumor::General });

        // Act
        app.update(); // process_gossip

        // Assert
        assert_eq!(app.world().resource::<IntelTokens>().0, 1);
    }

    #[test]
    fn test_gossiping_reduces_productivity() {
        // Arrange
        let mut app = setup_app();
        let pop = app.world_mut().spawn((
            Pop::default(),
            WorkTask { progress: 0.0, rate: 1.0 },
        )).id();

        app.world_mut().resource_mut::<Events<GossipEvent>>().send(GossipEvent { pop, rumor: Rumor::General });

        // Act
        app.update(); // process_gossip intercepts work cycle

        // Assert
        let task = app.world().get::<WorkTask>(pop).unwrap();
        assert_eq!(task.progress, 0.0, "Pop should not progress work while gossiping");
    }

    #[test]
    fn test_dark_secret_gossip_lowers_morale() {
        // Arrange
        let mut app = setup_app();
        let pop = app.world_mut().spawn((
            Pop::default(),
            Morale { value: 50.0 },
        )).id();

        app.world_mut().resource_mut::<Events<GossipEvent>>().send(GossipEvent { pop, rumor: Rumor::DarkSecret });

        // Act
        app.update(); // process_gossip

        // Assert
        let morale = app.world().get::<Morale>(pop).unwrap();
        assert!(morale.value < 50.0, "Dark secret rumor should lower morale");
        assert_eq!(app.world().resource::<IntelTokens>().0, 2, "Dark secrets generate more intel");
    }

    #[test]
    fn test_spend_intel_at_broker() {
        // Arrange
        let mut app = setup_app();
        app.world_mut().resource_mut::<IntelTokens>().0 = 10;

        app.world_mut().resource_mut::<Events<BrokerPurchaseEvent>>().send(BrokerPurchaseEvent { item: BrokerItem::RareTech, cost: 5 });

        // Act
        app.update(); // spend_intel

        // Assert
        assert_eq!(app.world().resource::<IntelTokens>().0, 5, "Intel tokens should be deducted");
        // Verify tech unlocked (omitted for brevity)
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::social::rumor::{Rumor, GossipEvent};
use crate::layer1::social::morale::Morale;

#[derive(Resource, Default)]
pub struct IntelTokens(pub u32);

#[derive(Component)]
pub struct WorkTask {
    pub progress: f32,
    pub rate: f32,
}

#[derive(Event)]
pub struct BrokerPurchaseEvent {
    pub item: BrokerItem,
    pub cost: u32,
}

pub enum BrokerItem {
    RareTech,
    RelationsBoost,
}

pub fn process_gossip(
    mut gossip_events: EventReader<GossipEvent>,
    mut intel: ResMut<IntelTokens>,
    mut pops: Query<(&mut Morale, Option<&mut WorkTask>)>,
) {
    for event in gossip_events.read() {
        if let Ok((mut morale, mut task)) = pops.get_mut(event.pop) {
            match event.rumor {
                Rumor::General => {
                    intel.0 += 1;
                }
                Rumor::DarkSecret => {
                    intel.0 += 2;
                    morale.value -= 10.0;
                }
            }

            // Halt work progress for the tick
            if let Some(mut t) = task {
                t.rate = 0.0; // Temporary debuff or skip execution
            }
        }
    }
}

pub fn spend_intel(
    mut purchase_events: EventReader<BrokerPurchaseEvent>,
    mut intel: ResMut<IntelTokens>,
) {
    for event in purchase_events.read() {
        if intel.0 >= event.cost {
            intel.0 -= event.cost;
            // Execute broker purchase logic here
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Action Cycle Integration:** Gossip shouldn't just instantly drop work rate; it should be integrated into the `utility_ai.rs` action cycle as a distinct action. Pops choose "Gossip" instead of "Work" based on traits and social needs.
- **Intel Cap/Decay:** Intel should decay over time if unspent. "Old news is useless."
- **Black Market Spawning:** The Broker shouldn't be a generic UI button; it should be an entity (e.g., a shady merchant at the spaceport) that only appears periodically.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops generate Intel when processing rumors.
- [ ] Pops spreading Dark Secrets suffer morale penalties.
- [ ] Intel can be successfully spent via events.
- [ ] Gossiping disrupts work actions.

## 7. Technical Guidance
- Integrate with `layer1::utility_ai`. If a Pop's social need is high and they have a juicy rumor, the utility of gossiping should outweigh working.
- Ensure `IntelTokens` are serialized alongside `ColonyResources`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
