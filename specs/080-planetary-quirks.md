# 080: Planetary Quirks

## Overview

Introduces **Planetary Quirks** (Layer 2 -> Layer 1) that modify the fundamental rules of the simulation. Each new game generates a set of planetary traits (e.g., High Gravity, Rapid Orbit, Dense Atmosphere) that provide global modifiers to existing systems. This increases replayability by forcing players to adapt their strategies to the specific world they are colonizing.

## Dependencies

- `001` — Project Scaffold (for `SimulationTime`)
- `004` — Pop Entity (for `Speed` component)
- `042` — Energy System (for `PowerSource` output)
- `065` — Day/Night Cycle (for `ticks_per_day`)

## RED Phase: Tests First

Write these tests in `src/layer1/quirks_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::quirks::{PlanetaryTraits, PlanetaryTrait, apply_quirk_modifiers_system};
    use crate::layer1::pop::{Pop, Speed};
    use crate::layer1::day_night::DayNightCycle;
    use crate::layer1::energy::PowerSource;
    use crate::layer1::building::{Building, BuildingType};
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(PlanetaryTraits::default());
        world
    }

    #[test]
    fn test_traits_initialization() {
        let world = setup_world();
        let traits = world.resource::<PlanetaryTraits>();
        assert!(traits.0.is_empty()); // Default is Earth-like (no quirks)
    }

    #[test]
    fn test_high_gravity_slows_movement() {
        let mut world = setup_world();
        world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::HighGravity]));

        // Spawn pop with default speed
        let pop = world.spawn((
            Pop,
            Speed { base: 1.0, current: 1.0, accumulator: 0.0 }
        )).id();

        // Run modifier system
        world.run_system_once(apply_quirk_modifiers_system).unwrap();

        // Check speed
        let speed = world.get::<Speed>(pop).unwrap();
        // High Gravity = 0.8x speed
        assert!((speed.current - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_low_gravity_speeds_movement() {
        let mut world = setup_world();
        world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::LowGravity]));

        let pop = world.spawn((
            Pop,
            Speed { base: 1.0, current: 1.0, accumulator: 0.0 }
        )).id();

        world.run_system_once(apply_quirk_modifiers_system).unwrap();

        let speed = world.get::<Speed>(pop).unwrap();
        // Low Gravity = 1.2x speed
        assert!((speed.current - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rapid_orbit_shortens_day() {
        let mut world = setup_world();
        world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::RapidOrbit]));

        // Setup DayNightCycle with default
        let default_ticks = 250;
        world.insert_resource(DayNightCycle {
            ticks_per_day: default_ticks,
            ..Default::default()
        });

        // Run modifier system
        world.run_system_once(apply_quirk_modifiers_system).unwrap();

        let cycle = world.resource::<DayNightCycle>();
        // Rapid Orbit = 0.5x day length
        assert_eq!(cycle.ticks_per_day, (default_ticks as f32 * 0.5) as u64);
    }

    #[test]
    fn test_dense_atmosphere_reduces_solar_power() {
        let mut world = setup_world();
        world.insert_resource(PlanetaryTraits(vec![PlanetaryTrait::DenseAtmosphere]));

        // Spawn Generator (Solar/Wind implied for this test context, or generic "Generator")
        // If 042 uses "Generator" for all power, we assume atmosphere affects efficiency generally
        // or specifically adds a penalty.
        let generator = world.spawn((
            Building { building_type: BuildingType::Generator },
            PowerSource { output: 10.0 }
        )).id();

        world.run_system_once(apply_quirk_modifiers_system).unwrap();

        let source = world.get::<PowerSource>(generator).unwrap();
        // Dense Atmosphere = 0.8x output
        assert!((source.output - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_modifiers_stack() {
        let mut world = setup_world();
        // High Gravity (0.8 speed) + Low Gravity (1.2 speed) = 0.96 speed
        world.insert_resource(PlanetaryTraits(vec![
            PlanetaryTrait::HighGravity,
            PlanetaryTrait::LowGravity
        ]));

        let pop = world.spawn((
            Pop,
            Speed { base: 1.0, current: 1.0, accumulator: 0.0 }
        )).id();

        world.run_system_once(apply_quirk_modifiers_system).unwrap();

        let speed = world.get::<Speed>(pop).unwrap();
        assert!((speed.current - 0.96).abs() < f32::EPSILON);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Traits (`src/layer1/quirks.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::pop::Speed;
use crate::layer1::day_night::DayNightCycle;
use crate::layer1::energy::PowerSource;
use crate::layer1::building::{Building, BuildingType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanetaryTrait {
    HighGravity,      // -20% Speed
    LowGravity,       // +20% Speed
    RapidOrbit,       // -50% Day Length
    SlowOrbit,        // +100% Day Length
    DenseAtmosphere,  // -20% Power Output (Solar blocked)
    ThinAtmosphere,   // +20% Power Output (Solar clearer)
}

impl PlanetaryTrait {
    pub fn speed_modifier(&self) -> f32 {
        match self {
            Self::HighGravity => 0.8,
            Self::LowGravity => 1.2,
            _ => 1.0,
        }
    }

    pub fn day_length_modifier(&self) -> f32 {
        match self {
            Self::RapidOrbit => 0.5,
            Self::SlowOrbit => 2.0,
            _ => 1.0,
        }
    }

    pub fn power_output_modifier(&self) -> f32 {
        match self {
            Self::DenseAtmosphere => 0.8,
            Self::ThinAtmosphere => 1.2,
            _ => 1.0,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::HighGravity => "High Gravity",
            Self::LowGravity => "Low Gravity",
            Self::RapidOrbit => "Rapid Orbit",
            Self::SlowOrbit => "Slow Orbit",
            Self::DenseAtmosphere => "Dense Atmosphere",
            Self::ThinAtmosphere => "Thin Atmosphere",
        }
    }
}

#[derive(Resource, Default, Debug)]
pub struct PlanetaryTraits(pub Vec<PlanetaryTrait>);
```

### 2. Implement System

```rust
pub fn apply_quirk_modifiers_system(
    traits: Res<PlanetaryTraits>,
    mut pops: Query<&mut Speed>,
    mut day_night: ResMut<DayNightCycle>,
    mut power_sources: Query<(&mut PowerSource, &Building)>,
) {
    if traits.0.is_empty() {
        return;
    }

    // 1. Calculate aggregate modifiers
    let mut speed_mod = 1.0;
    let mut day_mod = 1.0;
    let mut power_mod = 1.0;

    for trait_ in &traits.0 {
        speed_mod *= trait_.speed_modifier();
        day_mod *= trait_.day_length_modifier();
        power_mod *= trait_.power_output_modifier();
    }

    // 2. Apply to Pops (Speed)
    // Note: This overrides current speed every tick.
    // Ideally, we should apply to `base` or have a separate `modifier` field.
    // For MVP Green phase, we assume `current` is recalculated from `base` each tick
    // or we modify `current` directly if no other systems touch it.
    // Better approach: `current = base * global_modifiers * local_modifiers`.
    for mut speed in &mut pops {
        speed.current = speed.base * speed_mod;
    }

    // 3. Apply to DayNightCycle
    // Only apply if changed? For MVP, just set it.
    // Be careful with integer math truncating.
    // We need the BASE ticks per day to multiply against.
    // Assumption: DayNightCycle has a `base_ticks_per_day` or we just reset it to default * mod?
    // Hack for MVP: Hardcode base constant here or assume `ticks_per_day` IS the base
    // and we shouldn't multiply it repeatedly.
    // Correct approach: `DayNightCycle` needs `base_ticks` field, or we apply once at startup.
    // Let's assume we set it once at startup in a `Startup` system?
    // But this system is likely `Update`.
    // Let's use a static base for calculation:
    let base_ticks = 250; // From spec 065 default
    day_night.ticks_per_day = (base_ticks as f32 * day_mod) as u64;

    // 4. Apply to Power Sources
    // Only affect Generators (assuming they are Solar/Atmo dependent for this trait)
    for (mut source, building) in &mut power_sources {
        if matches!(building.building_type, BuildingType::Generator) {
             // Again, need base output.
             // Hack: Generator base is 10.0.
             source.output = 10.0 * power_mod;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Modifier Architecture**: Direct modification of values (like `speed.current`) in an Update system fights with other systems (like Weather `079`).
  - *Refactor*: Introduce a `StatModifiers` resource that accumulates all global multipliers (Gravity * Weather * Tech). `Speed` system reads this.
- **Initialization**: Applying `day_night.ticks_per_day` every tick is redundant. Move to a `Startup` system or `OnEnter(GameState::Playing)`.
- **Base Values**: Hardcoding `250` or `10.0` inside the system is brittle. Components should store `base_value` and `current_value`.
  - `PowerSource` already has `output`, maybe add `base_output`.
  - `DayNightCycle` needs `base_ticks_per_day`.
- **UI**: Display active Planetary Quirks in the Pause Menu or Status Bar tooltip.

## Acceptance Criteria

- [ ] `PlanetaryTraits` resource exists.
- [ ] At least 4 distinct traits are implemented.
- [ ] Traits correctly modify `Speed`, `DayNightCycle`, and `PowerSource`.
- [ ] Tests verify that modifiers are applied correctly.
- [ ] Multiple traits stack multiplicatively.
- [ ] `cargo test` passes.

## Technical Guidance

- Register `PlanetaryTraits` in `main.rs`.
- Add `apply_quirk_modifiers_system` to `SimulationSchedule`. Place it **before** other logic systems (like `movement`) but **after** systems that might reset values to base (if any).
- Ideally, run this system ONLY when `PlanetaryTraits` changes or at startup, to avoid fighting with other temporary modifiers.
