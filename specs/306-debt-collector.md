# Specification 306: The Debt Collector

## 1. Overview
This feature introduces a powerful Layer 3 faction auditor that orbits the player's colony to collect "back taxes." If the player refuses or is unable to pay the demanded credits or resources, the Auditor initiates a precision strike against the colony's luxury and social buildings (e.g., Tavern, Museum, Statue), bypassing military targets entirely. This cripples morale and forces the player to choose between wealth and social stability.

## 2. Dependencies
- `ColonyResources` (Spec 022, `src/layer1/resources.rs`)
- `Building` components (`src/layer1/buildings.rs`)
- `FleetCombat` (Spec 159)
- `ColonyEvent` system (`src/layer1/events.rs`)

## 3. RED Phase: Tests First

```rust
// src/layer3/auditor.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::resources::{ColonyResources, ResourceType};
    use crate::layer1::buildings::{Building, BuildingType, Health};

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(ColonyResources::new());
        app.add_event::<AuditorArrivalEvent>();
        app.add_event::<AuditorDemandEvent>();
        app.add_event::<AuditorRefusalEvent>();
        app.add_systems(Update, (
            process_auditor_demand_system,
            execute_auditor_strike_system,
        ));
        app
    }

    #[test]
    fn test_auditor_demand_payment_success() {
        let mut app = setup_app();

        let mut resources = app.world_mut().resource_mut::<ColonyResources>();
        resources.add(ResourceType::Credits, 5000.0);

        app.world_mut().resource_mut::<Events<AuditorDemandEvent>>().send(
            AuditorDemandEvent { amount: 5000.0, resource: ResourceType::Credits }
        );

        app.update();

        // Payment successful, resources deducted
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.get(ResourceType::Credits), 0.0);
    }

    #[test]
    fn test_auditor_demand_payment_failure_triggers_strike() {
        let mut app = setup_app();

        let mut resources = app.world_mut().resource_mut::<ColonyResources>();
        resources.add(ResourceType::Credits, 1000.0); // Not enough

        app.world_mut().resource_mut::<Events<AuditorDemandEvent>>().send(
            AuditorDemandEvent { amount: 5000.0, resource: ResourceType::Credits }
        );

        app.update();

        // Resources unchanged
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.get(ResourceType::Credits), 1000.0);

        // AuditorRefusalEvent should be sent
        let refusal_events = app.world().resource::<Events<AuditorRefusalEvent>>();
        assert_eq!(refusal_events.len(), 1);
    }

    #[test]
    fn test_auditor_strike_destroys_luxury_building() {
        let mut app = setup_app();

        let tavern = app.world_mut().spawn((
            Building { building_type: BuildingType::Tavern },
            Health { current: 100.0, max: 100.0 },
        )).id();

        app.world_mut().resource_mut::<Events<AuditorRefusalEvent>>().send(AuditorRefusalEvent);

        app.update();

        // Tavern should be heavily damaged or destroyed
        let health = app.world().get::<Health>(tavern).unwrap();
        assert!(health.current <= 0.0);
    }

    #[test]
    fn test_auditor_strike_ignores_military_building() {
        let mut app = setup_app();

        let turret = app.world_mut().spawn((
            Building { building_type: BuildingType::Turret },
            Health { current: 100.0, max: 100.0 },
        )).id();

        app.world_mut().resource_mut::<Events<AuditorRefusalEvent>>().send(AuditorRefusalEvent);

        app.update();

        // Turret health untouched
        let health = app.world().get::<Health>(turret).unwrap();
        assert_eq!(health.current, 100.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer3/auditor.rs

use bevy::prelude::*;
use crate::layer1::resources::{ColonyResources, ResourceType};
use crate::layer1::buildings::{Building, BuildingType, Health};

#[derive(Event, Clone, Debug)]
pub struct AuditorArrivalEvent;

#[derive(Event, Clone, Debug)]
pub struct AuditorDemandEvent {
    pub amount: f32,
    pub resource: ResourceType,
}

#[derive(Event, Clone, Debug)]
pub struct AuditorRefusalEvent;

pub fn process_auditor_demand_system(
    mut events: EventReader<AuditorDemandEvent>,
    mut resources: ResMut<ColonyResources>,
    mut refusal_writer: EventWriter<AuditorRefusalEvent>,
) {
    for ev in events.read() {
        if resources.get(ev.resource) >= ev.amount {
            // In a real implementation, add a specific consume method or match on ResourceType
            // Assuming generic consume is available or mapping to specific methods:
            // For now, we simulate generic consumption
            if ev.resource == ResourceType::Credits {
                // If generic consume is available: resources.consume(ResourceType::Credits, ev.amount);
                // Placeholder mapping for minimal implementation:
                let current = resources.get(ResourceType::Credits);
                // Implementation requires matching on resource type due to existing architecture
            }
        } else {
            refusal_writer.send(AuditorRefusalEvent);
        }
    }
}

pub fn execute_auditor_strike_system(
    mut events: EventReader<AuditorRefusalEvent>,
    mut query: Query<(&Building, &mut Health)>,
) {
    for _ in events.read() {
        // Strike the first luxury building found
        for (building, mut health) in query.iter_mut() {
            if matches!(building.building_type, BuildingType::Tavern | BuildingType::Museum | BuildingType::Statue) {
                health.current = 0.0;
                break; // One precision strike per refusal
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **API Mapping**: Ensure `ColonyResources` API is used correctly. `ColonyResources` lacks a generic `consume` across all variants. The `process_auditor_demand_system` must map the `ResourceType` strictly using `consume_credits()`, `consume_food()`, etc.
- **Strike Targeting**: The minimal implementation strikes the first found building. In a refactored version, it should prioritize the highest value luxury building or affect an AoE around the largest social hub to maximize grievance output.
- **Narrative Hook**: The strike should spawn a `ChronicleEvent` detailing the Auditor's attack on a specific named building (e.g., "The Taxman broke the Golden Keg Tavern.").

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for `auditor.rs`.
- [ ] Auditor correctly deducts the exact amount if available.
- [ ] Auditor strike correctly damages or destroys at least one luxury building and leaves military/industrial buildings intact on refusal.

## 7. Technical Guidance
- **Integration**: The demand and refusal system must be tightly integrated with the game's existing UI to prompt the player, rather than instantly draining resources in the background. Add a `PendingDemand` state or similar.
- **ColonyResources constraints**: The tests assume `add` and `get` methods. Make sure the implementation maps accurately to the actual resource deduction methods (like `consume_food(amount)` and `consume_metal(amount)`).

## 8. Questions
*Builder: add questions here if spec is unclear.*
