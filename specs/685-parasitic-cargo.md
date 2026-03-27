# The Parasitic Cargo

## 1. Overview
The Trojan Horse, but the horse is a crate of nutrient paste and the Greeks are microscopic, hyper-aggressive spores. High-value, deeply discounted resource shipments from unknown Layer 2 traders have a chance to carry a "Parasitic Cargo." Once opened on Layer 1 by haulers, they release a fast-spreading contagion or an invasive, rapidly reproducing vermin species that immediately attacks food stores or infects Pops.

## 2. Dependencies
- `022` Resource Stockpiles
- `073` Vermin Infestation
- `039` Trade System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::trade::{TradeShip, Shipment};
    use crate::layer1::resources::{Resource, Storage};
    use crate::layer1::fauna::Vermin;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<UnpackShipmentEvent>();
        app.add_systems(Update, process_suspicious_cargo);
        app
    }

    #[test]
    fn test_unpacking_parasitic_cargo_spawns_vermin() {
        // Arrange
        let mut app = setup_app();

        let storage_entity = app.world_mut().spawn((
            Storage { resources: vec![], capacity: 100 },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let shipment = Shipment {
            contents: Resource::Food(50),
            is_parasitic: true,
        };

        // Act - Simulate a Pop unpacking the crate at storage
        app.world_mut().resource_mut::<Events<UnpackShipmentEvent>>().send(
            UnpackShipmentEvent { shipment, destination: storage_entity }
        );
        app.update();

        // Assert - The food was destroyed, and vermin spawned instead
        let storage = app.world().get::<Storage>(storage_entity).unwrap();
        assert!(storage.resources.is_empty(), "Food should be consumed immediately");

        let mut vermin_count = 0;
        for (_, transform) in app.world_mut().query::<(&Vermin, &Transform)>().iter(app.world()) {
            if transform.translation.distance(Vec3::ZERO) < 1.0 {
                vermin_count += 1;
            }
        }
        assert!(vermin_count >= 5, "At least 5 vermin should have spawned from the cargo");
    }

    #[test]
    fn test_unpacking_normal_cargo_adds_to_storage() {
        // Arrange
        let mut app = setup_app();

        let storage_entity = app.world_mut().spawn((
            Storage { resources: vec![], capacity: 100 },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let shipment = Shipment {
            contents: Resource::Food(50),
            is_parasitic: false,
        };

        // Act
        app.world_mut().resource_mut::<Events<UnpackShipmentEvent>>().send(
            UnpackShipmentEvent { shipment, destination: storage_entity }
        );
        app.update();

        // Assert - Normal cargo behaves normally
        let storage = app.world().get::<Storage>(storage_entity).unwrap();
        assert_eq!(storage.resources.len(), 1);
        if let Resource::Food(amount) = storage.resources[0] {
            assert_eq!(amount, 50);
        } else {
            panic!("Expected food resource");
        }
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::resources::{Resource, Storage};
use crate::layer1::fauna::Vermin;

#[derive(Clone)]
pub struct Shipment {
    pub contents: Resource,
    pub is_parasitic: bool,
}

#[derive(Event)]
pub struct UnpackShipmentEvent {
    pub shipment: Shipment,
    pub destination: Entity,
}

pub fn process_suspicious_cargo(
    mut commands: Commands,
    mut unpack_events: EventReader<UnpackShipmentEvent>,
    mut storages: Query<(&mut Storage, &Transform)>,
) {
    for event in unpack_events.read() {
        if let Ok((mut storage, transform)) = storages.get_mut(event.destination) {
            if event.shipment.is_parasitic {
                // Spawn a swarm of vermin instead of adding resources
                for _ in 0..5 {
                    commands.spawn((
                        Vermin::default(),
                        Transform::from_translation(transform.translation),
                    ));
                }
                // Optional: Notify the player they got scammed
                println!("WARNING: Parasitic cargo opened! Vermin released!");
            } else {
                // Normal storage behavior
                storage.resources.push(event.shipment.contents.clone());
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Broadcaster:** Hook into the Chronicle or Notification system (`10` Chronicle System, `46` Notifications System) to alert the player when a parasitic cargo goes off, rather than `println!`.
- **Cargo Types:** Allow different payloads for parasitic cargo (e.g., Toxic Spores that apply disease instead of vermin, or Nano-Mites that degrade building integrity).
- **Inspection Mechanics:** Give the player a chance to intercept it. A high-skill Pop in an "Inspector" role could scan crates at the spaceport before haulers move them to main storage.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] `Shipment` struct supports an `is_parasitic` flag.
- [ ] Opening parasitic cargo spawns hostile entities (Vermin) and destroys the expected resource payload.
- [ ] Normal cargo functions correctly.

## 7. Technical Guidance
- The Trade UI should obscure the `is_parasitic` flag. It should only appear as a highly discounted, generic good (e.g., "-80% Market Price").
- Ensure the Vermin spawn directly at the storage tile coordinates to simulate them pouring out of the crate.

## 8. Questions
*Builder: add questions here if spec is unclear.*
