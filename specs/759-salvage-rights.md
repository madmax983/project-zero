# Spec 759: Salvage Rights

## 1. Overview
**Layer:** 3
**Fantasy:** Scavenging the graveyards of the galaxy.
**Mechanic:** You purchase "Rights" to debris fields in other systems. You send a Layer 2 fleet off-map for a duration. They return with randomized loot (Components, Scrap, Artifacts) or damage.
**Emergence:** Your salvage fleet brings back a "Dormant Warform". It wakes up in your stockpile and rampage.
**Tension:** Safe Trade vs. Risky Scavenging.

## 2. Dependencies
- Layer 3 `Diplomacy` / `Factions` for purchasing rights.
- Layer 2 `Fleet` system for dispatching ships off-map.
- RNG loot tables and potential threat generation (e.g., Warform awakening).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer3::diplomacy::Credits;

    #[test]
    fn test_purchasing_salvage_rights_deducts_credits() {
        let mut app = App::new();

        let faction_id = app.world_mut().spawn(Credits { amount: 1000 }).id();

        // Setup purchase event
        app.add_event::<PurchaseSalvageRightsEvent>();
        app.world_mut().resource_mut::<Events<PurchaseSalvageRightsEvent>>()
            .send(PurchaseSalvageRightsEvent {
                buyer: faction_id,
                target_system: "graveyard_sector".into(),
                cost: 500,
            });

        app.add_systems(Update, process_salvage_purchases_system);
        app.update();

        let credits = app.world().get::<Credits>(faction_id).unwrap();
        assert_eq!(credits.amount, 500, "Credits should be deducted upon purchase");
    }

    #[test]
    fn test_fleet_returns_with_salvage() {
        let mut app = App::new();

        let fleet_id = app.world_mut().spawn((
            Fleet,
            SalvageMission { duration_left: 1 },
            Cargo { items: vec![] }
        )).id();

        app.add_systems(Update, process_salvage_missions_system);
        app.update(); // Mission duration hits 0

        let cargo = app.world().get::<Cargo>(fleet_id).unwrap();
        assert!(!cargo.items.is_empty(), "Fleet should return with salvaged items");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Credits {
    pub amount: u32,
}

#[derive(Event)]
pub struct PurchaseSalvageRightsEvent {
    pub buyer: Entity,
    pub target_system: String,
    pub cost: u32,
}

#[derive(Component)]
pub struct Fleet;

#[derive(Component)]
pub struct Cargo {
    pub items: Vec<String>,
}

#[derive(Component)]
pub struct SalvageMission {
    pub duration_left: u32,
}

pub fn process_salvage_purchases_system(
    mut events: EventReader<PurchaseSalvageRightsEvent>,
    mut credits_query: Query<&mut Credits>,
) {
    for event in events.read() {
        if let Ok(mut credits) = credits_query.get_mut(event.buyer) {
            if credits.amount >= event.cost {
                credits.amount -= event.cost;
                // Grant rights (simplified)
            }
        }
    }
}

pub fn process_salvage_missions_system(
    mut missions: Query<(&mut SalvageMission, &mut Cargo)>,
) {
    for (mut mission, mut cargo) in missions.iter_mut() {
        if mission.duration_left > 0 {
            mission.duration_left -= 1;
        }

        if mission.duration_left == 0 {
            // Give randomized loot
            cargo.items.push("scrap_metal".into());
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Mission Completion**: Handle removing the `SalvageMission` component once `duration_left` reaches 0 so we don't grant loot every tick thereafter.
- **RNG Loot**: Implement a weighted loot table that has a chance to drop high-value artifacts, standard scrap, or dangerous "Dormant Warforms" that can trigger an event upon unloading at the Layer 1 colony.
- **Off-map Status**: Fleets should probably be marked with an `OffMap` or `InTransit` component so they aren't interactable by standard Layer 2 combat/movement systems while salvaging.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code
- [ ] Players can purchase rights and fleets return with randomized cargo after a duration.

## 7. Technical Guidance
- The actual return of the fleet should trigger a `FleetArrivedEvent` or equivalent to integrate with existing cargo unloading systems.
- If a Warform is salvaged, generate a `HostileEntitySpawnEvent` when the cargo is opened/unloaded.

## 8. Questions
*Builder: add questions here if spec is unclear.*
