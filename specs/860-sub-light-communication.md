# Specification: 860 Sub-light Communication

## 1. Overview
**Layer:** 2
**Fantasy:** The mail packet. News travels at the speed of the ship.
**Mechanic:** In the absence of FTL comms infrastructure, critical game state updates ("News", such as price changes or war declarations) must physically travel via courier ships. This creates information latency where remote colonies act on outdated information.

## 2. Dependencies
- Layer 2 Ship Systems (`src/layer2/ship.rs`)
- Faction/Diplomacy System (for war declarations)
- Economy System (for price data)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_information_packet_travels_with_ship() {
        let mut app = App::new();
        app.add_plugins(SublightCommsPlugin);

        let node_a = app.world_mut().spawn(ColonyNode).id();
        let node_b = app.world_mut().spawn(ColonyNode).id();

        // Node A declares war on a third party
        let news = NewsItem::WarDeclaration {
            aggressor: node_a,
            target: Entity::PLACEHOLDER,
        };

        // Spawn a courier ship carrying the news from A to B
        let ship = app.world_mut().spawn((
            CourierShip,
            NewsPayload(vec![news.clone()]),
        )).id();

        // Node B does not know about the war yet
        let knowledge_b = app.world().get::<LocalKnowledge>(node_b);
        assert!(knowledge_b.is_none() || !knowledge_b.unwrap().known_news.contains(&news));

        // Simulate ship arriving at B
        app.world_mut().send_event(ShipArrivalEvent {
            ship_entity: ship,
            destination: node_b,
        });
        app.update();

        // Node B should now have the news
        let knowledge_b = app.world().get::<LocalKnowledge>(node_b).unwrap();
        assert!(knowledge_b.known_news.contains(&news), "Node B should receive news upon ship arrival");

        // Ship payload might be cleared or delivered depending on implementation
    }

    #[test]
    fn test_ftl_comms_bypasses_courier() {
        let mut app = App::new();
        app.add_plugins(SublightCommsPlugin);

        let node_a = app.world_mut().spawn(ColonyNode).id();
        let node_b = app.world_mut().spawn((ColonyNode, FtlReceiver)).id(); // Has FTL

        // Provide global FTL capability resource
        app.world_mut().insert_resource(FtlNetworkState { active: true });

        let news = NewsItem::PriceChange { resource: "Gold".into(), new_price: 500 };

        app.world_mut().send_event(BroadcastNewsEvent {
            source: node_a,
            news: news.clone(),
        });
        app.update();

        // Because FTL is active and B has a receiver, it should know instantly
        let knowledge_b = app.world().get::<LocalKnowledge>(node_b).unwrap();
        assert!(knowledge_b.known_news.contains(&news), "FTL node should receive news instantly");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct ColonyNode;

#[derive(Component)]
pub struct CourierShip;

#[derive(Clone, PartialEq, Debug)]
pub enum NewsItem {
    WarDeclaration { aggressor: Entity, target: Entity },
    PriceChange { resource: String, new_price: i32 },
}

#[derive(Component)]
pub struct NewsPayload(pub Vec<NewsItem>);

#[derive(Component, Default)]
pub struct LocalKnowledge {
    pub known_news: Vec<NewsItem>,
}

#[derive(Event)]
pub struct ShipArrivalEvent {
    pub ship_entity: Entity,
    pub destination: Entity,
}

#[derive(Component)]
pub struct FtlReceiver;

#[derive(Resource, Default)]
pub struct FtlNetworkState {
    pub active: bool,
}

#[derive(Event)]
pub struct BroadcastNewsEvent {
    pub source: Entity,
    pub news: NewsItem,
}

pub struct SublightCommsPlugin;

impl Plugin for SublightCommsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShipArrivalEvent>()
           .add_event::<BroadcastNewsEvent>()
           .init_resource::<FtlNetworkState>()
           .add_systems(Update, (
               handle_courier_delivery_system,
               handle_ftl_broadcast_system,
           ));
    }
}

fn handle_courier_delivery_system(
    mut events: EventReader<ShipArrivalEvent>,
    mut ships: Query<&mut NewsPayload>,
    mut nodes: Query<&mut LocalKnowledge>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok(mut payload) = ships.get_mut(event.ship_entity) {

            // Ensure destination has a LocalKnowledge component
            if nodes.get(event.destination).is_err() {
                commands.entity(event.destination).insert(LocalKnowledge::default());
            }

            // In Bevy 0.15+, queries inside the loop might need to fetch again to get the mutated state,
            // but since we insert above, we can do a second lookup safely or restructure.
            // For minimal green:
            if let Ok(mut knowledge) = nodes.get_mut(event.destination) {
                for news in payload.0.drain(..) {
                    if !knowledge.known_news.contains(&news) {
                        knowledge.known_news.push(news);
                    }
                }
            }
        }
    }
}

fn handle_ftl_broadcast_system(
    mut events: EventReader<BroadcastNewsEvent>,
    ftl_state: Res<FtlNetworkState>,
    mut nodes: Query<(Entity, Option<&mut LocalKnowledge>), With<FtlReceiver>>,
    mut commands: Commands,
) {
    if !ftl_state.active {
        return;
    }

    for event in events.read() {
        for (node_entity, knowledge_opt) in nodes.iter_mut() {
            if let Some(mut knowledge) = knowledge_opt {
                if !knowledge.known_news.contains(&event.news) {
                    knowledge.known_news.push(event.news.clone());
                }
            } else {
                commands.entity(node_entity).insert(LocalKnowledge {
                    known_news: vec![event.news.clone()],
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Currently `LocalKnowledge` is a flat `Vec` which requires O(N) search and unbounded memory. Refactor to a `HashMap` or rolling buffer using timestamped events.
- Ensure that obsolete news (e.g. Price of Gold changed 5 times) is consolidated so colonies only care about the most recent state.
- Create a clear distinction between the "True Global State" and "Local Assumed State" for the economy and diplomacy systems to query against.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the comms module.
- [ ] Ships arriving at destinations transfer their `NewsPayload` into the destination's `LocalKnowledge`.
- [ ] Global FTL broadcasts update `LocalKnowledge` instantly on nodes with receivers.

## 7. Technical Guidance
- The diplomacy systems (which calculate "Can I trade with X?" or "Are they hostile?") MUST query the `LocalKnowledge` of the current active node, NOT the global true state. This is the core of the feature.
- Couriers should pick up local news when departing a node to spread it further. This behavior isn't explicitly tested in the RED phase but is required for the rumor network to function.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
