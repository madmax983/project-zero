# 159: Fleet Combat Resolution

## 1. Overview

**Fantasy:** Space battles are massive, chaotic events observed from a distance. Two fleets merge into a signal blip, flashes of light appear, and only one signal remains—surrounded by debris.

**Mechanic:**
- **Deterministic Resolution**: Combat occurs automatically when two hostile fleets occupy the same coordinate/node.
- **Auto-Calc**: Battles are resolved in "Rounds" or instantly (for MVP, instant resolution based on stats).
- **Casualties**: Ships are damaged or destroyed.
- **Debris**: Destroyed ships leave `OrbitalDebris` (Spec 184) or `Loot` (Cargo).

**Why:** Layer 2 is currently peaceful. We need conflict to drive the need for `Ship Classes` (Spec 157) and `Defenses`.

## 2. Dependencies

- `099` — Fleet Movement (Implemented)
- `157` — Ship Classes (In Backlog - **MUST BE IMPLEMENTED FIRST**)
- `104` — Fuel Industry (Implied for resources/loot)

## 3. RED Phase: Tests First

Write these tests in `src/layer2/combat_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::fleet::{Fleet, FleetComposition, FleetFaction};
    use crate::layer2::ship::{Ship, ShipType};
    use crate::layer2::combat::{resolve_combat, CombatResult};

    // Helper to create a test fleet
    fn create_fleet(faction: FleetFaction, ships: Vec<ShipType>) -> FleetComposition {
        let mut comp = FleetComposition::default();
        for t in ships {
            comp.add_ship(Ship::new(t));
        }
        comp
    }

    #[test]
    fn test_combat_resolution_stronger_wins() {
        // Fleet A: 10 Frigates (High Combat)
        let fleet_a = create_fleet(FleetFaction::Player, vec![ShipType::Frigate; 10]);

        // Fleet B: 1 Scout (Low Combat)
        let fleet_b = create_fleet(FleetFaction::Pirate, vec![ShipType::Scout; 1]);

        // Resolve
        let result = resolve_combat(&fleet_a, &fleet_b);

        // A should win
        assert_eq!(result.winner, FleetFaction::Player);
        // B should be wiped out
        assert!(result.loser_survivors.ships.is_empty());
        // A should take minimal/no damage
        assert_eq!(result.winner_survivors.ships.len(), 10);
    }

    #[test]
    fn test_combat_casualties() {
        // Fleet A: 5 Frigates
        let fleet_a = create_fleet(FleetFaction::Player, vec![ShipType::Frigate; 5]);

        // Fleet B: 5 Frigates
        let fleet_b = create_fleet(FleetFaction::Pirate, vec![ShipType::Frigate; 5]);

        // Even fight, both sides should take losses
        let result = resolve_combat(&fleet_a, &fleet_b);

        // We assume deterministic behavior or seeded RNG in implementation
        // For this test, just ensure *someone* died
        let total_survivors = result.winner_survivors.ships.len() + result.loser_survivors.ships.len();
        assert!(total_survivors < 10);
    }

    #[test]
    fn test_loot_generation() {
        // Fleet B (Transport) has Cargo. If destroyed, it should drop loot.
        // This requires FleetCargo component integration, which we mock here or add to resolve_combat args.
        // For MVP, resolve_combat just returns a "Loot" object.

        let fleet_a = create_fleet(FleetFaction::Player, vec![ShipType::Frigate; 10]);
        let fleet_b = create_fleet(FleetFaction::Pirate, vec![ShipType::Transport; 1]); // Transport has high cargo

        let result = resolve_combat(&fleet_a, &fleet_b);

        // If Transport destroyed, loot generated
        assert!(result.loot.is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Define `FleetFaction`

In `src/layer2/fleet.rs`:
```rust
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FleetFaction {
    Player,
    Pirate,
    Merchant,
    // ...
}
```

### 2. Define `CombatResult`

In `src/layer2/combat.rs`:
```rust
use crate::layer2::fleet::{FleetComposition, FleetFaction};
use crate::layer2::ship::ShipType;

pub struct CombatResult {
    pub winner: FleetFaction,
    pub winner_survivors: FleetComposition,
    pub loser_survivors: FleetComposition,
    pub loot: Option<Vec<(crate::layer1::resources::ResourceType, f32)>>,
}

pub fn resolve_combat(attacker: &FleetComposition, defender: &FleetComposition) -> CombatResult {
    // 1. Calculate Total Attack Power
    let att_power: f32 = attacker.ships.iter().map(|s| s.ship_type.attack_power()).sum();
    let def_power: f32 = defender.ships.iter().map(|s| s.ship_type.attack_power()).sum(); // Simplified: Attack = Defense power for MVP?

    // Spec 157 needs `attack_power()` on ShipType. If not there, add it.

    // 2. Determine Winner
    // Simple logic: Higher power wins.
    // RNG: power * random(0.8, 1.2)

    // 3. Apply Damage
    // "Damage" = Losing Power / 2.
    // Apply damage to ships health. Remove dead ships.

    // 4. Return Result
    CombatResult {
        // ...
    }
}
```

### 3. Implement System

```rust
pub fn fleet_combat_system(
    mut commands: Commands,
    // Query for fleets with position, faction, and composition
    mut fleets: Query<(Entity, &crate::layer2::fleet::FleetPosition, &FleetFaction, &mut FleetComposition)>,
) {
    // 1. Group fleets by location
    // 2. If multiple factions at same location, trigger combat
    // 3. Apply results (despawn dead fleets, update survivors, spawn debris)
}
```

## 5. REFACTOR Phase: Quality & Design

- **Ship Stats**: `ShipType` (Spec 157) needs `attack`, `defense`, `health`.
- **Combat Logic**: Move from "Total Power" to "Ship vs Ship" targeting rounds for more realism.
- **Debris**: Spawn `OrbitalDebris` entity (Spec 184) containing the loot.
- **Notifications**: "Fleet Battle at Sector 4: Victory!"

## 6. Acceptance Criteria

- [ ] `FleetFaction` component exists.
- [ ] `resolve_combat` correctly identifies winner based on strength.
- [ ] Ships are removed from `FleetComposition` if destroyed.
- [ ] Fleets at the same position automatically fight if hostile.
- [ ] Tests pass.

## 7. Technical Guidance

- Use `IterTools` or a `HashMap` to group fleets by position in the system.
- Combat should be instantaneous for MVP.
- Ensure `ShipType` has the necessary stats. If Spec 157 didn't add `attack_power`, you must add it (part of the RED/GREEN cycle for this feature).

## 8. Questions

- *Builder: How do I calculate loot?*
  *Architect: Loot should be a percentage of the destroyed ship's construction cost, plus any cargo it was carrying.*
*Architect: For now, grant a percentage (e.g. 10-20%) of the destroyed ship\'s construction cost in raw materials, plus a small chance for a "Salvage Data" item.*
    - *Architect: Take 50% of the destroyed fleet's cargo.*
