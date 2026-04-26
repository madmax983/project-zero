# 1202: Bureaucratic Black Holes

## 1. Overview
**Layer:** 2 -> 1
**Fantasy:** A planetary administration so complex and opaque that entire shipments, and sometimes people, just vanish into the paperwork.
**Mechanic:** As a Layer 2 node (planet) reaches a certain population density, its "Bureaucratic Complexity" score rises. High complexity increases the chance that resource shipments or assigned Pops simply disappear. These aren't stolen; they are misfiled. Occasionally, an "Audit Event" occurs, suddenly dumping decades worth of missing resources or very confused, chronologically displaced Pops onto a single tile.
**Emergence:** You are starving and desperately need a food shipment from the core worlds. It gets "lost in transit." Fifty years later, during a golden age of surplus, a colossal mountain of rotting grain suddenly materializes in the middle of your capital city's plaza because a clerk finally processed the form.
**Tension:** Do you ruthlessly purge your administration, taking a massive hit to overall efficiency and stability, or accept that a certain percentage of your empire simply ceases to exist on paper?

## 2. Dependencies
- Layer 2 Node Population / Admin stats
- Layer 1 Resource Hauling / Inventory
- Pop Lifecycle management
- Global Event system (for "Audit Event")

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_high_complexity_causes_misfiling() {
        let mut app = App::new();
        app.add_systems(Update, process_shipments_system);

        app.world.insert_resource(BureaucraticComplexity(0.8)); // 80% complexity

        let target = app.world.spawn_empty().id();
        let shipment_id = app.world.spawn(Shipment { target, items: 100, is_misfiled: false }).id();

        app.update();

        // Given high complexity, the shipment should be flagged as misfiled.
        // We use a deterministic mock randomizer in the actual tests, but for the spec,
        // we assume high complexity forces it.
        let shipment = app.world.get::<Shipment>(shipment_id).unwrap();
        assert!(shipment.is_misfiled, "Shipments should be misfiled in highly complex bureaucracies");
    }

    #[test]
    fn test_audit_event_restores_misfiled_entities() {
        let mut app = App::new();
        app.add_systems(Update, audit_event_system);

        let target = app.world.spawn_empty().id();
        let misfiled_shipment = app.world.spawn(Shipment { target, items: 100, is_misfiled: true }).id();

        app.world.insert_resource(Events::<AuditEvent>::default());
        let mut events = app.world.get_resource_mut::<Events<AuditEvent>>().unwrap();
        events.send(AuditEvent);

        app.update();

        // The audit event should clear the misfiled flag and deliver the items
        let shipment = app.world.get::<Shipment>(misfiled_shipment).unwrap();
        assert!(!shipment.is_misfiled, "Audit should clear the misfiled status");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct BureaucraticComplexity(pub f32); // 0.0 to 1.0

#[derive(Component)]
pub struct Shipment {
    pub target: Entity,
    pub items: u32,
    pub is_misfiled: bool,
}

#[derive(Event)]
pub struct AuditEvent;

pub fn process_shipments_system(
    complexity: Res<BureaucraticComplexity>,
    mut shipments: Query<&mut Shipment>,
) {
    for mut shipment in shipments.iter_mut() {
        // Simplified deterministic logic for testing purposes:
        // if complexity is very high (> 0.5), it gets misfiled.
        // In reality, this should use a random roll against complexity.
        if complexity.0 > 0.5 && !shipment.is_misfiled {
            shipment.is_misfiled = true;
        }
    }
}

pub fn audit_event_system(
    mut audit_events: EventReader<AuditEvent>,
    mut misfiled_items: Query<&mut Shipment>,
) {
    for _ in audit_events.read() {
        // When an audit happens, process ALL misfiled paperwork at once
        for mut shipment in misfiled_items.iter_mut() {
            if shipment.is_misfiled {
                shipment.is_misfiled = false;
                // Deliver items to target logic would go here
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Randomization:** The minimal implementation is deterministic. Inject a `ResMut<GlobalEntropy>` or similar RNG to roll probability based on `BureaucraticComplexity`.
- **Misfiled Pops:** Extend the logic to `Pop` entities. A misfiled pop shouldn't render on the map or consume food until the audit, essentially acting as if they are in "administrative stasis."
- **Audit Trigger:** Create a system that randomly triggers `AuditEvent`s, perhaps tied to a "Regime Change" or explicit player "Audit" action that costs high admin points.

## 6. Acceptance Criteria
- [ ] Shipments and Pops have a chance to be hidden/disabled based on `BureaucraticComplexity`.
- [ ] `AuditEvent` restores all hidden/misfiled entities to active status.
- [ ] All RED phase tests pass.
- [ ] Coverage >= 85%.

## 7. Technical Guidance
- **State Management:** When a Pop is misfiled, be careful not to delete the entity. Remove `Visibility` or active processing components, or move them to an "Admin Limbo" `GridPosition` so they aren't processed by standard needs systems.
- Use Bevy's `EventReader::read()` iterator.

## 8. Questions
*Builder: Add any questions here.*
