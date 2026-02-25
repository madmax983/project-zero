# 226: Ship Personalities

## Overview

Ships in Layer 2 are no longer generic stats. They accumulate "Quirks" based on their history (surviving low hull, running out of fuel, successful combat). These quirks provide buffs/debuffs and give each ship a unique "Personality".

## Dependencies

- `157` — Ship Classes (Implemented)
- `159` — Fleet Combat (Implemented)

## RED Phase: Tests First

```rust
// src/layer2/ship_quirks_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_quirk_acquisition_on_damage() {
        let mut world = World::new();
        // Spawn ship with low health (after combat)
        let ship = world.spawn((
            Ship,
            ShipHealth { current: 1.0, max: 100.0 }, // 1% HP
            Quirks::default(),
        )).id();

        // Run system that checks for trauma/events
        let mut schedule = Schedule::default();
        schedule.add_systems(ship_quirk_generation_system);
        schedule.run(&mut world);

        // Verify "Lucky" or "Scarred" quirk added
        let quirks = world.get::<Quirks>(ship).unwrap();
        assert!(quirks.has("Lucky") || quirks.has("Rattled"));
    }

    #[test]
    fn test_quirk_effect_application() {
        let mut world = World::new();
        let mut stats = ShipStats { evasion: 0.1, fuel_efficiency: 1.0 };

        let quirks = Quirks {
            traits: vec![ShipQuirk::Lucky, ShipQuirk::FuelHog],
        };

        // Apply quirks to stats
        apply_quirks(&quirks, &mut stats);

        // Lucky: +Evasion, FuelHog: -Efficiency
        assert!(stats.evasion > 0.1);
        assert!(stats.fuel_efficiency < 1.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### Components

```rust
// src/layer2/quirks.rs

#[derive(Component, Default, Debug)]
pub struct Quirks {
    pub traits: Vec<ShipQuirk>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShipQuirk {
    Lucky,      // +Evasion
    Rattled,    // -Accuracy
    FuelHog,    // -Efficiency
    Veteran,    // +Accuracy
}

impl Quirks {
    pub fn has(&self, name: &str) -> bool {
        // Simple string match for test, or match enum
        match name {
            "Lucky" => self.traits.contains(&ShipQuirk::Lucky),
            "Rattled" => self.traits.contains(&ShipQuirk::Rattled),
            _ => false,
        }
    }
}
```

### Systems

```rust
// src/layer2/quirks.rs

pub fn ship_quirk_generation_system(
    mut commands: Commands,
    mut query: Query<(Entity, &ShipHealth, &mut Quirks)>,
) {
    for (entity, health, mut quirks) in query.iter_mut() {
        // If survived near death (1% HP)
        if health.current > 0.0 && health.current <= health.max * 0.05 {
            if !quirks.traits.contains(&ShipQuirk::Lucky) {
                // Chance to gain Lucky
                quirks.traits.push(ShipQuirk::Lucky);
                // Log event
            }
        }
    }
}

pub fn apply_quirks(quirks: &Quirks, stats: &mut ShipStats) {
    for quirk in &quirks.traits {
        match quirk {
            ShipQuirk::Lucky => stats.evasion += 0.1,
            ShipQuirk::Rattled => stats.accuracy -= 0.1, // Assuming stats has accuracy
            ShipQuirk::FuelHog => stats.fuel_efficiency *= 0.8,
            ShipQuirk::Veteran => stats.accuracy += 0.1,
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Event Driven**: Use `ShipDamageEvent` or `CombatEndEvent` instead of checking health every frame.
- **UI**: Display quirks in the Fleet Inspector.
- **Persistence**: Ensure quirks are saved/loaded.
- **Balance**: Limit max quirks per ship (e.g., 3).

## Acceptance Criteria

- [ ] `Quirks` component added to ships.
- [ ] Events (Low Health, Combat Win, Fuel Out) trigger Quirk generation.
- [ ] Quirks modify Ship Stats (Speed, Evasion, Fuel).
- [ ] Tests pass.

## Technical Guidance

- Implement `apply_quirks` as a helper called whenever stats are recalculated (e.g. `recalculate_fleet_stats`).
- Quirks should be rare; use a PRNG (Pseudo Random Number Generator) check.

## Questions

*Builder: add questions here if spec is unclear.*
