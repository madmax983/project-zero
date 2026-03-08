# Spec 433: The Phantom Debt

## 1. Overview
A heavily armed, automated Layer 3 "Collection Fleet" arrives in your system. They claim your colony world was purchased by a defunct precursor empire thousands of years ago, and they are here to collect the accumulated interest. They demand an impossible amount of resources or they will foreclose on the planet.

## 2. Dependencies
- `099-fleet-movement`
- `159-fleet-combat-resolution`
- `018-mining-resources`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_collection_fleet_arrives_and_demands() {
        let mut app = App::new();
        app.insert_resource(PhantomDebtEvent { active: false, demand: 0.0 });
        app.add_systems(Update, spawn_collection_fleet_system);

        app.world_mut().insert_resource(TriggerPhantomDebt);

        app.update();

        let debt = app.world().resource::<PhantomDebtEvent>();
        assert_eq!(debt.active, true);
        assert_eq!(debt.demand, 10_000_000.0); // An impossible amount
    }

    #[test]
    fn test_fleet_leaves_if_colony_goes_dark() {
        let mut app = App::new();
        app.insert_resource(ColonySignatures { power: 0.0, tech: 0.0 });
        app.insert_resource(PhantomDebtEvent { active: true, demand: 10_000_000.0 });
        app.add_systems(Update, evaluate_dark_age_survival_system);

        app.update();

        let debt = app.world().resource::<PhantomDebtEvent>();
        assert_eq!(debt.active, false); // Fleet leaves if signatures are zero
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct TriggerPhantomDebt;

#[derive(Resource)]
pub struct PhantomDebtEvent {
    pub active: bool,
    pub demand: f32,
}

#[derive(Resource)]
pub struct ColonySignatures {
    pub power: f32,
    pub tech: f32,
}

pub fn spawn_collection_fleet_system(
    mut commands: Commands,
    trigger: Option<Res<TriggerPhantomDebt>>,
    mut debt: ResMut<PhantomDebtEvent>,
) {
    if trigger.is_some() {
        debt.active = true;
        debt.demand = 10_000_000.0;
        commands.remove_resource::<TriggerPhantomDebt>();
    }
}

pub fn evaluate_dark_age_survival_system(
    signatures: Res<ColonySignatures>,
    mut debt: ResMut<PhantomDebtEvent>,
) {
    if debt.active && signatures.power == 0.0 && signatures.tech == 0.0 {
        debt.active = false;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate `ColonySignatures` dynamically by summing up active power generators and research stations.
- Spawn an actual invincible Fleet entity in Layer 2 that orbits the planet during the debt event.
- If the fleet doesn't leave and the debt isn't paid within a timer, trigger a catastrophic orbital bombardment event.

## 6. Acceptance Criteria
- [ ] `TriggerPhantomDebt` activates the impossible debt demand.
- [ ] Shutting down all power and tech (ColonySignatures to 0) causes the debt event to deactivate (fleet leaves).
- [ ] Tests pass with >= 85% coverage.

## 7. Technical Guidance
- Hook into the Energy System to track total power consumption for `ColonySignatures`.
- Ensure the fleet is properly despawned in Layer 2 when the event concludes.

## 8. Questions
- How long does the colony have to stay "dark" before the fleet is fully convinced and leaves?
- *Architect:* The colony must remain below the energy threshold for 3 in-game days to fully drain the fleet's patience and trigger their departure.
