# 135: Trash Cannon Defense

## Overview

Introduces active defensive structures that repurpose industrial waste as ammunition. The **Trash Cannon** is a turret that consumes `Waste` from the global colony resources to fire at hostile entities. Upon impact, the projectile spawns a small amount of `Waste` on the ground, creating a "mess" that must be cleaned up but allowing some resource reclamation.

**Why:**
- Turns a liability (Waste) into an asset (Ammo).
- Adds active defense to the colony (beyond passive Walls).
- Creates a tactical choice: Recycle waste for resources or shoot it for defense?
- Adds a "cleanup" mechanic after battles.

## Dependencies

- `043` — Defensive Structures (for `BuildingType` and `Health`)
- `049` — Industrial Waste (for `ResourceType::Waste` and `Landfill` logic)
- `067` — Militia System (for `AttackProperties` and `CombatState`)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/turret_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::combat::{AttackProperties, CombatState};
    use crate::layer1::health::Health;
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
    use crate::layer1::turret::{Turret, turret_fire_system};
    use crate::layer1::fauna::{Fauna, FaunaType};

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(crate::layer1::map::ScreenShake::default());
        world
    }

    // 1. Turret Component Initialization
    #[test]
    fn test_turret_component_defaults() {
        let turret = Turret {
            attack: AttackProperties {
                damage: 10.0,
                range: 5.0,
                cooldown: 20,
                accuracy: 1.0,
            },
            ammo_cost: 1.0,
            ammo_type: ResourceType::Waste,
        };
        assert_eq!(turret.ammo_cost, 1.0);
        assert_eq!(turret.ammo_type, ResourceType::Waste);
    }

    // 2. Fire Logic - Needs Ammo
    #[test]
    fn test_trash_cannon_needs_ammo() {
        let mut world = setup_world();
        // Zero waste
        world.resource_mut::<ColonyResources>().waste = 0.0;

        // Spawn Turret
        let turret = world.spawn((
            Building { building_type: BuildingType::TrashCannon },
            Turret {
                attack: AttackProperties {
                    damage: 10.0,
                    range: 5.0,
                    cooldown: 0,
                    accuracy: 1.0,
                },
                ammo_cost: 1.0,
                ammo_type: ResourceType::Waste,
            },
            GridPosition { x: 0, y: 0 },
            CombatState::default(),
        )).id();

        // Spawn Enemy in range
        let enemy = world.spawn((
            Fauna { fauna_type: FaunaType::Wolf, ..Default::default() },
            GridPosition { x: 2, y: 0 },
            Health { current: 100.0, max: 100.0 },
        )).id();

        // Run system
        turret_fire_system(&mut world);

        // Check Enemy Health (Unchanged)
        let health = world.get::<Health>(enemy).unwrap();
        assert_eq!(health.current, 100.0, "Turret should not fire without ammo");
    }

    // 3. Fire Logic - Consumes Ammo & Deals Damage
    #[test]
    fn test_trash_cannon_fires_and_consumes_ammo() {
        let mut world = setup_world();
        // Add ammo
        world.resource_mut::<ColonyResources>().waste = 10.0;

        // Spawn Turret
        let turret = world.spawn((
            Building { building_type: BuildingType::TrashCannon },
            Turret {
                attack: AttackProperties {
                    damage: 10.0,
                    range: 5.0,
                    cooldown: 10, // Set cooldown
                    accuracy: 1.0,
                },
                ammo_cost: 1.0,
                ammo_type: ResourceType::Waste,
            },
            GridPosition { x: 0, y: 0 },
            CombatState::default(),
        )).id();

        // Spawn Enemy
        let enemy = world.spawn((
            Fauna { fauna_type: FaunaType::Wolf, ..Default::default() },
            GridPosition { x: 2, y: 0 },
            Health { current: 100.0, max: 100.0 },
        )).id();

        // Run system
        turret_fire_system(&mut world);

        // Check Enemy Health
        let health = world.get::<Health>(enemy).unwrap();
        assert_eq!(health.current, 90.0, "Turret should deal damage");

        // Check Ammo
        let res = world.resource::<ColonyResources>();
        assert_eq!(res.waste, 9.0, "Turret should consume 1.0 waste");

        // Check Cooldown
        let state = world.get::<CombatState>(turret).unwrap();
        assert_eq!(state.cooldown, 10, "Turret should set cooldown");
    }

    // 4. Impact Effect - Spawns Waste Item
    #[test]
    fn test_trash_cannon_spawns_mess_on_impact() {
        let mut world = setup_world();
        world.resource_mut::<ColonyResources>().waste = 10.0;

        let turret = world.spawn((
            Building { building_type: BuildingType::TrashCannon },
            Turret {
                attack: AttackProperties {
                    damage: 10.0,
                    range: 5.0,
                    cooldown: 0,
                    accuracy: 1.0,
                },
                ammo_cost: 1.0,
                ammo_type: ResourceType::Waste,
            },
            GridPosition { x: 0, y: 0 },
            CombatState::default(),
        )).id();

        let enemy_pos = GridPosition { x: 3, y: 3 };
        let enemy = world.spawn((
            Fauna { fauna_type: FaunaType::Wolf, ..Default::default() },
            enemy_pos,
            Health { current: 100.0, max: 100.0 },
        )).id();

        turret_fire_system(&mut world);

        // Check for Waste item at enemy position
        let mut found_waste = false;
        let mut query = world.query::<(&GridPosition, &ResourceItem)>();
        for (pos, item) in query.iter(&world) {
            if item.resource_type == ResourceType::Waste && *pos == enemy_pos {
                found_waste = true;
                break;
            }
        }
        assert!(found_waste, "Impact should spawn Waste item at target location");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `BuildingType` (`src/layer1/building.rs`)

```rust
pub enum BuildingType {
    // ...
    TrashCannon,
}

impl BuildingType {
    pub fn char(&self) -> char {
        match self {
            Self::TrashCannon => '♣', // Club/Weapon symbol
            // ...
        }
    }

    pub fn cost(&self, material: MaterialType) -> ColonyResources {
        match self {
            Self::TrashCannon => ColonyResources {
                metal: 20.0,
                stone: 10.0,
                ..ColonyResources::zeroed()
            },
            // ...
        }
    }

    // Spawn logic
    fn spawn_building(...) {
        match building_type {
            Self::TrashCannon => {
                entity.insert((
                    Turret {
                        attack: AttackProperties {
                            damage: 15.0,
                            range: 7.0,
                            cooldown: 30, // Slow fire rate
                            accuracy: 0.9,
                        },
                        ammo_cost: 1.0,
                        ammo_type: ResourceType::Waste,
                    },
                    CombatState::default(),
                ));
            },
            // ...
        }
    }
}
```

### 2. Define `Turret` Component (`src/layer1/turret.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::combat::{AttackProperties, CombatState, execute_attack};
use crate::layer1::resources::{ColonyResources, ResourceType, ResourceItem};
use crate::layer1::map::GridPosition;
use crate::layer1::building::Building;
use crate::layer1::fauna::Fauna; // Or generic Hostile marker
use crate::layer1::health::Health;

#[derive(Component, Debug, Clone)]
pub struct Turret {
    pub attack: AttackProperties,
    pub ammo_cost: f32,
    pub ammo_type: ResourceType,
}

pub fn turret_fire_system(world: &mut World) {
    // 1. Collect potential targets (Hostiles)
    // Optimization: Spatial query would be better, but O(N*M) is acceptable for MVP count.
    let mut targets = Vec::new();
    let mut query = world.query::<(Entity, &GridPosition, &Health)>();
    // Filter for Hostile Fauna or Raiders (check components)
    // For now, assume Fauna with Health > 0 are valid targets
    for (entity, pos, health) in query.iter(world) {
        if health.current > 0.0 {
            // Check if it's hostile? (Fauna usually is, or check specific component)
             if world.get::<Fauna>(entity).is_some() {
                 targets.push((entity, *pos));
             }
        }
    }

    // 2. Iterate Turrets
    let mut turrets = Vec::new();
    let mut query = world.query::<(Entity, &GridPosition, &Turret, &mut CombatState)>();
    for (entity, pos, turret, state) in query.iter_mut(world) {
        if state.cooldown == 0 {
            turrets.push((entity, *pos, turret.clone()));
        } else {
            state.cooldown -= 1;
        }
    }

    // 3. Fire Logic (Mutable World Access needed)
    for (turret_entity, turret_pos, turret_data) in turrets {
        // Check Ammo
        let mut resources = world.resource_mut::<ColonyResources>();
        let has_ammo = match turret_data.ammo_type {
            ResourceType::Waste => resources.waste >= turret_data.ammo_cost,
            _ => false, // Only Waste supported for now
        };

        if !has_ammo { continue; }

        // Find Target in Range
        let mut best_target = None;
        let mut min_dist = f32::MAX;

        for (target_entity, target_pos) in &targets {
            let dist = turret_pos.distance(*target_pos);
            if dist <= turret_data.attack.range && dist < min_dist {
                min_dist = dist;
                best_target = Some((*target_entity, *target_pos));
            }
        }

        if let Some((target_entity, target_pos)) = best_target {
            // Deduct Ammo
            let mut resources = world.resource_mut::<ColonyResources>();
            match turret_data.ammo_type {
                ResourceType::Waste => resources.waste -= turret_data.ammo_cost,
                _ => {},
            }

            // Deal Damage via shared combat logic
            // Note: execute_attack expects attacker to have Weapon component usually,
            // but we can manually apply damage or refactor execute_attack.
            // Better: Manual application here to avoid adding Weapon to Building.
            if let Some(mut health) = world.get_mut::<Health>(target_entity) {
                health.take_damage(turret_data.attack.damage);
            }

            // Set Cooldown
            if let Some(mut state) = world.get_mut::<CombatState>(turret_entity) {
                state.cooldown = turret_data.attack.cooldown;
            }

            // Spawn Impact Mess (Waste Item)
            // Small chance? Or always? Spec says "spawns a small amount".
            world.spawn((
                ResourceItem {
                    resource_type: ResourceType::Waste,
                    amount: 0.1, // Small amount
                },
                target_pos,
            ));

            // Visuals
            crate::layer1::particles::spawn_particle(
                world,
                target_pos,
                'x',
                ratatui::style::Color::DarkGray,
                5
            );
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Targeting Efficiency**: Querying all hostiles every frame per turret is slow (O(N*M)).
    - *Refactor*: Use a `SpatialGrid` or `KdTree` for target lookups.
- **Combat Logic**: `execute_attack` in `combat.rs` is coupled to `Weapon` component.
    - *Refactor*: Extract `apply_damage` helper that takes `damage` amount directly, used by both `execute_attack` (Pop) and `turret_fire_system` (Building).
- **Ammo Abstraction**: `ColonyResources` is hardcoded.
    - *Refactor*: Make `AmmoSource` a trait or component to allow local inventory consumption later.

## Acceptance Criteria

- [ ] `TrashCannon` is buildable.
- [ ] `Turret` component exists and is attached to Trash Cannon.
- [ ] Turrets automatically fire at `Fauna` within range.
- [ ] Firing consumes global `Waste` resource.
- [ ] Firing spawns `Waste` items on target.
- [ ] Visual feedback (particles) on impact.
- [ ] Tests pass.

## Technical Guidance

- Ensure `turret_fire_system` is added to the `SimulationSchedule`.
- `CombatState` on buildings must be ticked (decremented) every frame, just like pops.
- `distance` calculation should use `Chebyshev` (max(dx, dy)) to match grid movement, or `Euclidean` if preferred for range. `GridPosition::distance` usually implements one of these. Use existing method.

## Questions

*Builder: How much waste creates a "mess"?*
*Architect:* Any stack of waste greater than 5 units triggers the mess state.
*Architect:* A "mess" is defined as any single tile accumulating more than 5 units of industrial waste, triggering negative beauty effects.
