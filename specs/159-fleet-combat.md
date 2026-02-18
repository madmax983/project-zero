# 159: Fleet Combat Resolution

## 1. Overview

As fleets begin to explore the system, they will inevitably encounter hostile forces (Pirates, Rivals).
This feature implements the **Combat Resolution** system for Layer 2.

When two fleets from different factions occupy the same orbital location (`InOrbit`), combat is initiated automatically.
Combat is resolved in turns (ticks), where ships exchange damage based on their class stats.
Ships that reach 0 HP are destroyed and removed from the fleet.
If a fleet loses all ships, the fleet entity is destroyed.

## 2. Dependencies

- `specs/157-ship-classes.md` (Ship, ShipType, FleetComposition)
- `specs/099-fleet-movement.md` (Fleet, InOrbit)

## 3. RED Phase: Tests First

Write these tests in `src/layer2/combat_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::fleet::{Fleet, FleetComposition, InOrbit};
    use crate::layer2::ship::{Ship, ShipType};
    use crate::layer2::combat::{FactionId, InCombat, fleet_combat_system};

    fn setup_combat_world() -> World {
        let mut world = World::new();
        // Register components
        world
    }

    #[test]
    fn test_combat_initiation() {
        let mut world = setup_combat_world();
        let planet = world.spawn_empty().id();

        // Player Fleet
        let player_fleet = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            FactionId(0), // Player
            FleetComposition {
                ships: vec![Ship::new(ShipType::Frigate)],
            },
        )).id();

        // Pirate Fleet
        let pirate_fleet = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            FactionId(1), // Pirate
            FleetComposition {
                ships: vec![Ship::new(ShipType::Frigate)],
            },
        )).id();

        // Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(fleet_combat_system);
        schedule.run(&mut world);

        // Verify both are InCombat
        assert!(world.get::<InCombat>(player_fleet).is_some());
        assert!(world.get::<InCombat>(pirate_fleet).is_some());
    }

    #[test]
    fn test_combat_damage_exchange() {
        let mut world = setup_combat_world();
        let planet = world.spawn_empty().id();

        // Player Frigate (100 HP)
        let player_fleet = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            FactionId(0),
            FleetComposition {
                ships: vec![Ship::new(ShipType::Frigate)],
            },
        )).id();

        // Pirate Scout (Weak, 50 HP)
        let pirate_fleet = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            FactionId(1),
            FleetComposition {
                ships: vec![Ship::new(ShipType::Scout)],
            },
        )).id();

        // Run System (One Round)
        let mut schedule = Schedule::default();
        schedule.add_systems(fleet_combat_system);
        schedule.run(&mut world);

        // Verify Damage
        let player_comp = world.get::<FleetComposition>(player_fleet).unwrap();
        let pirate_comp = world.get::<FleetComposition>(pirate_fleet).unwrap();

        // Frigate (Player) should have taken some damage from Scout
        assert!(player_comp.ships[0].health < 100.0);

        // Scout (Pirate) should have taken MORE damage from Frigate
        // Assuming Frigate ATK > Scout ATK
        assert!(pirate_comp.ships[0].health < 100.0);
    }

    #[test]
    fn test_ship_destruction() {
        let mut world = setup_combat_world();
        let planet = world.spawn_empty().id();

        // Player Fleet
        let player_fleet = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            FactionId(0),
            FleetComposition {
                ships: vec![Ship::new(ShipType::Frigate)],
            },
        )).id();

        // Pirate Fleet with 1 Scout at 1 HP
        let mut weak_scout = Ship::new(ShipType::Scout);
        weak_scout.health = 1.0;

        let pirate_fleet = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            FactionId(1),
            FleetComposition {
                ships: vec![weak_scout, Ship::new(ShipType::Miner)],
            },
        )).id();

        // Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(fleet_combat_system);
        schedule.run(&mut world);

        // Verify Scout is gone
        let pirate_comp = world.get::<FleetComposition>(pirate_fleet).unwrap();
        assert_eq!(pirate_comp.ships.len(), 1);
        assert_eq!(pirate_comp.ships[0].ship_type, ShipType::Miner);
    }

    #[test]
    fn test_fleet_destruction() {
        let mut world = setup_combat_world();
        let planet = world.spawn_empty().id();

        // Strong Player Fleet
        let player_fleet = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            FactionId(0),
            FleetComposition {
                ships: vec![Ship::new(ShipType::Frigate)],
            },
        )).id();

        // Weak Pirate Fleet (1 HP Scout)
        let mut weak_scout = Ship::new(ShipType::Scout);
        weak_scout.health = 1.0;

        let pirate_fleet = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            FactionId(1),
            FleetComposition {
                ships: vec![weak_scout],
            },
        )).id();

        // Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(fleet_combat_system);
        schedule.run(&mut world);

        // Verify Pirate Fleet Entity is Despawned
        assert!(world.get_entity(pirate_fleet).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer2/combat.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer2::fleet::{Fleet, InOrbit, FleetComposition};
use crate::layer2::ship::{Ship, ShipType};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FactionId(pub u32);

#[derive(Component, Debug, Clone, Copy)]
pub struct InCombat;

impl ShipType {
    pub fn attack_power(&self) -> f32 {
        match self {
            Self::Scout => 5.0,
            Self::Transport => 2.0,
            Self::Miner => 3.0,
            Self::Frigate => 20.0,
        }
    }

    pub fn max_health(&self) -> f32 {
        match self {
            Self::Scout => 50.0,
            Self::Transport => 200.0,
            Self::Miner => 150.0,
            Self::Frigate => 300.0,
        }
    }
}
```

### 2. Update `Ship::new` (`src/layer2/ship.rs`)

Ensure `Ship::new` uses `ship_type.max_health()` instead of hardcoded `100.0`.

```rust
impl Ship {
    pub fn new(ship_type: ShipType) -> Self {
        Self {
            ship_type,
            health: ship_type.max_health(),
            max_health: ship_type.max_health(),
        }
    }
}
```

### 3. Implement Combat System (`src/layer2/combat.rs`)

```rust
pub fn fleet_combat_system(
    mut commands: Commands,
    mut fleets: Query<(Entity, &InOrbit, &FactionId, &mut FleetComposition, Option<&InCombat>)>,
) {
    // 1. Reset InCombat state for all
    // In a real system, we might want to keep it if combat persists, but recalculating is safer for MVP.
    for (entity, _, _, _, in_combat) in fleets.iter() {
        if in_combat.is_some() {
            commands.entity(entity).remove::<InCombat>();
        }
    }

    // 2. Identify Hostile Pairs and Resolve Combat
    // We use `iter_combinations_mut` to check every pair of fleets.
    // This is O(N^2), but acceptable for MVP with low fleet counts.
    let mut iter = fleets.iter_combinations_mut();
    while let Some([
        (e1, orbit1, faction1, mut comp1, _),
        (e2, orbit2, faction2, mut comp2, _)
    ]) = iter.fetch_next() {
        // Must be in same orbit
        if orbit1.parent != orbit2.parent {
            continue;
        }
        // Must be different factions
        if faction1 == faction2 {
            continue;
        }

        // Combat!
        commands.entity(e1).insert(InCombat);
        commands.entity(e2).insert(InCombat);

        // Calculate Total Damage Output
        let dmg1: f32 = comp1.ships.iter().map(|s| s.ship_type.attack_power()).sum();
        let dmg2: f32 = comp2.ships.iter().map(|s| s.ship_type.attack_power()).sum();

        // Apply Damage to Fleet 2 (from 1)
        apply_damage(&mut comp2, dmg1);

        // Apply Damage to Fleet 1 (from 2)
        apply_damage(&mut comp1, dmg2);
    }
}

fn apply_damage(comp: &mut FleetComposition, mut damage: f32) {
    // Distribute damage to ships
    // Strategy: Focus fire first living ship.

    for ship in comp.ships.iter_mut() {
        if damage <= 0.0 { break; }

        if ship.health > damage {
            ship.health -= damage;
            damage = 0.0;
        } else {
            damage -= ship.health;
            ship.health = 0.0;
        }
    }

    // Remove dead ships
    comp.ships.retain(|s| s.health > 0.0);
}

// 4. Cleanup System
// Separate system to handle fleet destruction.
// Registered after combat system.
pub fn fleet_cleanup_system(
    mut commands: Commands,
    query: Query<(Entity, &FleetComposition), With<Fleet>>,
) {
    for (entity, comp) in query.iter() {
        if comp.ships.is_empty() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Event Log**: Emit `CombatLogEvent` when ships die or damage is dealt.
- **Fleeing**: Add logic where if `health < 20%`, fleet attempts `FleetOrder::MoveTo(SafeHaven)`.
- **Targeting**: Implement `TargetingStrategy` (Weakest First, Strongest First, Random).
- **Damage Mitigation**: Armor/Shields.
- **Optimization**: `iter_combinations_mut` is O(N^2). If 1000 fleets are in system, this is slow. Use spatial hashing (Grid/Orbit) to limit checks.

## 6. Acceptance Criteria

- [ ] `FactionId` and `InCombat` components defined.
- [ ] `ShipType` has combat stats.
- [ ] `Ship::new` uses max health from stats.
- [ ] System identifies hostile fleets in same orbit.
- [ ] Damage is exchanged and applied to ships.
- [ ] Dead ships are removed.
- [ ] Empty fleets are despawned.
- [ ] Tests pass.

## 7. Technical Guidance

- Use `iter_combinations_mut()` for the pair checks. It is safe and handles the borrowing.
- Remember to separate the "Damage Logic" from "Cleanup Logic" to avoid despawning an entity while another entity is trying to shoot it in the same frame (though `combinations` handles this via borrowing, logic order matters).
- Ensure `apply_damage` handles overflow (spillover damage) correctly.
