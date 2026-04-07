# Specification: 861 Stellar Cartography

## 1. Overview
**Layer:** 3
**Fantasy:** The galaxy is dark and unknown. Information is the first resource you need.
**Mechanic:** Star systems are hidden. You can send "Probe" ships (slow, cheap) or buy "Star Charts" (fast, expensive) from traders. Charts have "Freshness"; old charts might miss a new supernova or pirate base.

## 2. Dependencies
- Layer 3 Galaxy Map Nodes
- LocalKnowledge (from 860 Sub-light Communication)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_stellar_cartography_freshness() {
        let mut app = App::new();
        app.add_plugins(StellarCartographyPlugin);

        let system_a = app.world_mut().spawn(StarSystemNode).id();
        let player = app.world_mut().spawn(LocalKnowledge::default()).id();

        // Add a fresh chart
        let mut knowledge = app.world_mut().get_mut::<LocalKnowledge>(player).unwrap();
        knowledge.known_charts.insert(system_a, StarChart { freshness: 100 });

        // Update loop reduces freshness over time
        app.world_mut().insert_resource(Time::new_with(bevy::utils::Duration::from_secs(10)));
        app.update();

        let knowledge = app.world().get::<LocalKnowledge>(player).unwrap();
        let chart = knowledge.known_charts.get(&system_a).unwrap();
        assert!(chart.freshness < 100, "Chart freshness should decrease over time");
    }

    #[test]
    fn test_blind_jump_risk() {
        let mut app = App::new();
        app.add_plugins(StellarCartographyPlugin);

        let system_unknown = app.world_mut().spawn(StarSystemNode).id();
        let ship = app.world_mut().spawn((Ship, Position(Vec3::ZERO))).id();

        app.world_mut().send_event(JumpEvent { ship, destination: system_unknown });
        app.update();

        // The jump should carry a risk if the system is unknown
        let has_risk_status = app.world().get::<JumpRisk>(ship).is_some();
        assert!(has_risk_status, "Jumping to an unknown system should apply a risk status");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub struct StarSystemNode;

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct Position(pub Vec3);

#[derive(Clone, Debug)]
pub struct StarChart {
    pub freshness: u32,
}

#[derive(Component, Default)]
pub struct LocalKnowledge {
    pub known_charts: HashMap<Entity, StarChart>,
}

#[derive(Event)]
pub struct JumpEvent {
    pub ship: Entity,
    pub destination: Entity,
}

#[derive(Component)]
pub struct JumpRisk;

pub struct StellarCartographyPlugin;

impl Plugin for StellarCartographyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<JumpEvent>()
           .add_systems(Update, (
               decay_chart_freshness_system,
               handle_jump_risk_system,
           ));
    }
}

fn decay_chart_freshness_system(mut queries: Query<&mut LocalKnowledge>) {
    for mut knowledge in queries.iter_mut() {
        for chart in knowledge.known_charts.values_mut() {
            if chart.freshness > 0 {
                chart.freshness = chart.freshness.saturating_sub(1);
            }
        }
    }
}

fn handle_jump_risk_system(
    mut events: EventReader<JumpEvent>,
    knowledge_query: Query<&LocalKnowledge>, // Player/Faction knowledge
    mut commands: Commands,
) {
    for event in events.read() {
        let is_known = knowledge_query.iter().any(|k| k.known_charts.contains_key(&event.destination));
        if !is_known {
            commands.entity(event.ship).insert(JumpRisk);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Currently `decay_chart_freshness_system` decays all charts unconditionally every frame. Refactor to use a timer or delta time to reduce overhead and tie decay to actual simulation time.
- `JumpRisk` is a simple marker; it should be integrated with actual damage or delay mechanics in the `ShipMovementSystem`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the module.
- [ ] Known star charts lose freshness over time.
- [ ] Jumping to an unknown system applies a `JumpRisk` component to the ship.

## 7. Technical Guidance
- Integrate with `LocalKnowledge` from `Sub-light Communication` to allow ships to trade/exchange star charts.
- Older charts should provide less info to the UI and might fail to reveal new hazards.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
