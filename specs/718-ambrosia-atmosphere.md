# 718: The Ambrosia Atmosphere

## 1. Overview
**Layer:** Cross-layer (2 -> 1)
**Fantasy:** A planet where the air itself is an addictive, euphoric drug that makes you forget why you ever wanted to leave.
**Mechanic:** A planet features a unique atmospheric composition that provides a permanent, massive Morale boost to all colonists breathing it, entirely nullifying all minor grievances and Unrest. However, it is highly addictive. If a Pop attempts to leave the planet (e.g., assigned to a Layer 2 fleet or a colony ship), they suffer crippling withdrawal, becoming permanently catatonic unless supplied with expensive, refined "Ambrosia Gas" exports from the homeworld.

## 2. Dependencies
- Morale/Unrest system (`crate::layer1::social::morale::Morale`)
- Planetary Environment/Layer 2 (`crate::layer2::planet::Planet`, `crate::layer2::atmosphere::Atmosphere`)
- Fleet/Space Travel assignment (`crate::layer2::fleet::FleetAssignment` or similar)
- Resource/Supply system (`crate::layer1::economy::ColonyResources`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::social::morale::Morale;
    use crate::layer2::planet::Atmosphere;
    use crate::layer1::economy::ColonyResources;
    use crate::layer2::fleet::FleetAssignment;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (apply_ambrosia_atmosphere_system, process_ambrosia_withdrawal_system));
        app
    }

    #[test]
    fn test_ambrosia_atmosphere_boosts_morale_and_addicts() {
        let mut app = setup_app();

        let planet = app.world_mut().spawn((
            Planet,
            AmbrosiaAtmosphere,
        )).id();

        // Pop on the planet
        let pop = app.world_mut().spawn((
            Pop,
            Location(planet),
            Morale { value: 50.0 }, // Base morale
        )).id();

        app.update();

        // Morale should be maxed or significantly boosted (e.g., 100.0)
        let updated_morale = app.world().entity(pop).get::<Morale>().unwrap();
        assert!(updated_morale.value >= 100.0);

        // Pop should now be addicted
        assert!(app.world().entity(pop).contains::<AmbrosiaAddicted>());
    }

    #[test]
    fn test_leaving_ambrosia_planet_without_supply_causes_withdrawal() {
        let mut app = setup_app();

        let mut resources = ColonyResources::default();
        resources.set(ResourceType::AmbrosiaGas, 0.0); // No gas supply
        app.insert_resource(resources);

        // Addicted pop leaving (e.g., in a fleet)
        let pop = app.world_mut().spawn((
            Pop,
            AmbrosiaAddicted,
            FleetAssignment { fleet_id: 1 }, // Left the planet
        )).id();

        app.update();

        // Pop should enter withdrawal/catatonic state
        assert!(app.world().entity(pop).contains::<Catatonic>());
        // Morale crashes
        let morale = app.world().entity(pop).get::<Morale>().unwrap();
        assert!(morale.value <= 10.0);
    }

    #[test]
    fn test_leaving_with_supply_consumes_gas_prevents_withdrawal() {
        let mut app = setup_app();

        let mut resources = ColonyResources::default();
        resources.set(ResourceType::AmbrosiaGas, 10.0); // Has gas supply
        app.insert_resource(resources);

        let pop = app.world_mut().spawn((
            Pop,
            AmbrosiaAddicted,
            FleetAssignment { fleet_id: 1 },
        )).id();

        app.update();

        // No withdrawal
        assert!(!app.world().entity(pop).contains::<Catatonic>());

        // Gas consumed
        let updated_resources = app.world().resource::<ColonyResources>();
        assert!(updated_resources.get(ResourceType::AmbrosiaGas) < 10.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::social::morale::Morale;
use crate::layer1::economy::{ColonyResources, ResourceType};
use crate::layer2::fleet::FleetAssignment;

#[derive(Component)]
pub struct AmbrosiaAtmosphere;

#[derive(Component)]
pub struct AmbrosiaAddicted;

#[derive(Component)]
pub struct Catatonic;

pub fn apply_ambrosia_atmosphere_system(
    mut commands: Commands,
    // Assuming pops share a location with the planet or the planet is implicitly the current level
    mut query: Query<(Entity, &mut Morale), (With<Pop>, Without<AmbrosiaAddicted>)>,
    planet_query: Query<&AmbrosiaAtmosphere>, // Check if the world has this
) {
    if planet_query.is_empty() { return; }

    for (entity, mut morale) in query.iter_mut() {
        morale.value = 100.0; // Max morale
        commands.entity(entity).insert(AmbrosiaAddicted);
    }
}

pub fn process_ambrosia_withdrawal_system(
    mut commands: Commands,
    time: Res<Time>,
    mut resources: ResMut<ColonyResources>,
    mut query: Query<(Entity, &mut Morale, &FleetAssignment), With<AmbrosiaAddicted>>,
) {
    let dt = time.delta_secs();
    let consumption_rate = 1.0 * dt; // Arbitrary drain rate

    for (entity, mut morale, _) in query.iter_mut() {
        // Pop has left the planet
        let current_gas = resources.get(ResourceType::AmbrosiaGas);
        if current_gas >= consumption_rate {
            // Consume gas, keep them somewhat sane
            resources.consume(ResourceType::AmbrosiaGas, consumption_rate);
        } else {
            // Withdrawal hits hard
            morale.value = 10.0; // Crash morale
            commands.entity(entity).insert(Catatonic);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Location Checking:** The `apply_ambrosia_atmosphere_system` needs to properly resolve the pop's location against the planet's atmospheric composition. Use a `Location(Entity)` or similar component.
- **Resource Definitions:** Ensure `ResourceType::AmbrosiaGas` is defined in the economy module, and perhaps a building (e.g., `AtmosphericCondenser`) to produce it.
- **Catatonic State Logic:** `Catatonic` pops should be prevented from working. The `calculate_work_amount` system must zero out production for them.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the Ambrosia mechanics.
- [ ] Pops on Ambrosia worlds gain perfect morale and become addicted.
- [ ] Addicted pops leaving the world consume Ambrosia Gas or become Catatonic.

## 7. Technical Guidance
- **Module:** This fits best in `src/layer2/atmosphere.rs` with integration into layer 1 `social` or `health`.
- **Integration:** The `FleetAssignment` check is a placeholder for "left the planet". Use the actual Layer 2 location transitions.
- **Morale Floor:** When crashing morale, ensure it doesn't drop below 0 (clamp it).

## 8. Questions
*Builder: add questions here if spec is unclear.*
