# 163: Vermin Evolution

## Overview

Vermin populations adapt to the colony's industrial byproducts. When vermin consume exotic resources (specifically **Waste** and **Fuel**), the global infestation gains unique **Traits** that apply colony-wide effects. This creates a dynamic where ignoring specific stockpiles leads to specialized threats.

- **Toxic Vermin** (from Waste): Resistant to extermination policies.
- **Volatile Vermin** (from Fuel): Chance to spontaneously ignite fuel stockpiles.

## Dependencies

- `073` — Vermin Infestation (Implemented)
- `104` — Fuel Industry (Fuel Resource)
- `049` — Industrial Waste (Waste Resource)
- `033` — Fire Propagation (Volatile trait trigger)

## RED Phase: Tests First

Write these tests in `src/layer1/vermin_evolution_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::vermin::{VerminState, VerminTrait, vermin_growth_system};
    use crate::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
    use crate::layer1::edicts::{ColonyPolicies, Policy};
    use crate::layer1::fire::Fire;
    use crate::layer1::map::GridPosition;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(VerminState::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(ColonyPolicies::default());
        world
    }

    #[test]
    fn test_gain_toxic_trait_from_waste() {
        let mut world = setup_world();

        // Add lots of Waste
        {
            let mut res = world.resource_mut::<ColonyResources>();
            res.waste = 1000.0;
            res.food = 10.0; // Minimal food so waste dominates
        }

        // Run growth system multiple times to trigger evolution
        // Evolution might be probability based or threshold based
        for _ in 0..100 {
            world.run_system_once(vermin_growth_system).unwrap();
        }

        let vermin = world.resource::<VerminState>();
        assert!(vermin.traits.contains(&VerminTrait::Toxic), "Vermin should become Toxic from waste");
    }

    #[test]
    fn test_gain_volatile_trait_from_fuel() {
        let mut world = setup_world();

        // Add lots of Fuel
        {
            let mut res = world.resource_mut::<ColonyResources>();
            res.fuel = 1000.0;
            res.food = 10.0;
        }

        for _ in 0..100 {
            world.run_system_once(vermin_growth_system).unwrap();
        }

        let vermin = world.resource::<VerminState>();
        assert!(vermin.traits.contains(&VerminTrait::Volatile), "Vermin should become Volatile from fuel");
    }

    #[test]
    fn test_toxic_vermin_resist_pest_control() {
        let mut world = setup_world();

        // Setup Toxic Vermin
        {
            let mut vermin = world.resource_mut::<VerminState>();
            vermin.severity = 50.0;
            vermin.traits.insert(VerminTrait::Toxic);
        }

        // Enable Pest Control
        {
            let mut policies = world.resource_mut::<ColonyPolicies>();
            policies.toggle(Policy::PestControl);
        }

        // Add Food to stimulate growth
        {
            let mut res = world.resource_mut::<ColonyResources>();
            res.food = 1000.0;
        }

        // Run system
        world.run_system_once(vermin_growth_system).unwrap();

        let vermin = world.resource::<VerminState>();
        // Normal pest control halves growth. Toxic should reduce effectiveness (e.g. only 25% reduction or no reduction).
        // If normal growth is X, with pest control it's 0.5*X.
        // With Toxic, maybe it's 0.8*X or 1.0*X.
        // We just assert it grew MORE than it would have with normal pest control.
        // Calculating expected:
        // Food impact = 1000 * 0.00005 = 0.05
        // Normal Pest Control = 0.025 growth.
        // Toxic Pest Control = > 0.025 growth.

        // Let's rely on specific numbers for the test or logic check
        let growth = vermin.severity - 50.0;
        assert!(growth > 0.026, "Toxic vermin should resist pest control (growth > 0.025)");
    }

    #[test]
    fn test_volatile_vermin_ignite_fuel() {
        let mut world = setup_world();

        // Setup Volatile Vermin
        {
            let mut vermin = world.resource_mut::<VerminState>();
            vermin.traits.insert(VerminTrait::Volatile);
            vermin.severity = 50.0;
        }

        // Spawn Fuel Item
        world.spawn((
            ResourceItem { resource_type: ResourceType::Fuel, amount: 10.0 },
            GridPosition { x: 5, y: 5 }
        ));

        // Run a new system: vermin_effect_system
        // This system handles the active effects of traits
        world.run_system_once(crate::layer1::vermin::vermin_effect_system).unwrap();

        // Check for Fire
        let fires = world.query::<&Fire>().iter(&world).count();
        // Since it's probabilistic, we might need to mock RNG or force it.
        // For TDD, assume we can force probability or run enough times.
        // Let's loop.

        let mut ignited = false;
        for _ in 0..100 {
             world.run_system_once(crate::layer1::vermin::vermin_effect_system).unwrap();
             if world.query::<&Fire>().iter(&world).count() > 0 {
                 ignited = true;
                 break;
             }
        }

        assert!(ignited, "Volatile vermin should eventually ignite fuel");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `VerminState` (`src/layer1/vermin.rs`)

```rust
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerminTrait {
    Toxic,
    Volatile,
}

#[derive(Resource)]
pub struct VerminState {
    pub severity: f32,
    pub max_severity: f32,
    pub traits: HashSet<VerminTrait>, // New field
}

impl Default for VerminState {
    fn default() -> Self {
        Self {
            severity: 0.0,
            max_severity: 100.0,
            traits: HashSet::new(),
        }
    }
}
```

### 2. Update `vermin_growth_system` logic

```rust
// In vermin_growth_system
// Track consumption types
let mut waste_consumed = total_waste * GROWTH_FACTOR_WASTE;
let mut fuel_consumed = total_fuel * GROWTH_FACTOR_FUEL; // Define this constant

// Update traits
// If > 20% of growth comes from Waste, become Toxic
if waste_consumed > (growth * 0.2) {
    vermin.traits.insert(VerminTrait::Toxic);
}
// If > 20% from Fuel, become Volatile
if fuel_consumed > (growth * 0.2) {
    vermin.traits.insert(VerminTrait::Volatile);
}

// Pest Control Resistance
let mut control_efficacy = 0.5;
if vermin.traits.contains(&VerminTrait::Toxic) {
    control_efficacy = 0.9; // Only 10% reduction instead of 50%
}

if policies.is_active(Policy::PestControl) {
    growth *= control_efficacy;
}
```

### 3. Implement `vermin_effect_system`

```rust
use crate::layer1::fire::Fire;

pub fn vermin_effect_system(
    mut commands: Commands,
    vermin: Res<VerminState>,
    query: Query<(Entity, &ResourceItem, &GridPosition)>,
) {
    if !vermin.traits.contains(&VerminTrait::Volatile) {
        return;
    }

    let mut rng = rand::thread_rng();

    // Check Fuel items
    for (entity, item, pos) in &query {
        if item.resource_type == ResourceType::Fuel {
            // Chance based on severity
            let chance = 0.001 * vermin.severity;
            if rng.gen_bool(chance as f64) {
                // Ignite!
                commands.spawn((Fire::default(), *pos));
                // Optional: Destroy fuel?
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Notifications**: Add a `NotificationQueue` call when a new trait is acquired ("The vermin are glowing...").
- **Trait Decay**: If the colony stops producing waste, the `Toxic` trait should eventually fade.
- **Visuals**: If `Volatile`, render small sparks over vermin-infested tiles (future).
- **Optimization**: `vermin_effect_system` queries all items every tick. This might be slow if there are thousands of items. Optimize by caching fuel locations or running less frequently (Timer).

## Acceptance Criteria

- [ ] `VerminState` includes `traits`.
- [ ] Tests verify `Toxic` trait acquisition from Waste.
- [ ] Tests verify `Volatile` trait acquisition from Fuel.
- [ ] `Toxic` vermin resist Pest Control (verified via growth rate).
- [ ] `Volatile` vermin cause fires on Fuel items.
- [ ] Code compiles and passes all tests.

## Technical Guidance

- Modify `ColonyResources` to include `fuel` if not used in calculation yet (it is in struct, but maybe not in growth system).
- Add `GROWTH_FACTOR_FUEL` constant.
- Remember to register `vermin_effect_system` in `simulation.rs`.

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
