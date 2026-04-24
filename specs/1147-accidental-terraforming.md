# 1147: Accidental Terraforming

## 1. Overview
The "Accidental Terraforming" feature allows extensive mining and industrial activity on Layer 1 to generate atmospheric byproducts. Over decades, these byproducts alter the `Atmosphere` and permanently shift the planet's biome type (e.g., from Arid to Toxic, or Ice to Tundra). This creates a tension between throttling industrial output to preserve the ecosystem and running factories hot and accepting the ecological collapse.

## 2. Dependencies
- `bevy_ecs` setup for handling entities.
- Layer 1 colony/planet components that hold `BiomeType` and `AtmosphericByproducts`.
- Building components that can generate `IndustrialOutput` or `Emissions`.
- A simulation time/tick system to slowly accumulate byproducts.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_emissions_accumulate_byproducts() {
        // Arrange
        let mut world = World::new();

        let planet = world.spawn((
            BiomeType::Ice,
            AtmosphericByproducts { heat: 0.0, toxins: 0.0 },
        )).id();

        let building = world.spawn((
            Building,
            Emissions { heat_per_tick: 10.0, toxins_per_tick: 5.0 },
        )).id();

        // Act: Run the accumulation system
        world.run_system_once(accumulate_emissions_system);

        // Assert: Planet should have accumulated the byproducts
        let byproducts = world.get::<AtmosphericByproducts>(planet).unwrap();
        assert_eq!(byproducts.heat, 10.0, "Heat should accumulate from buildings");
        assert_eq!(byproducts.toxins, 5.0, "Toxins should accumulate from buildings");
    }

    #[test]
    fn test_terraforming_biome_shift() {
        // Arrange
        let mut world = World::new();

        // Planet with high accumulated heat
        let planet = world.spawn((
            BiomeType::Ice,
            AtmosphericByproducts { heat: 1000.0, toxins: 0.0 },
        )).id();

        world.init_resource::<Events<BiomeShiftEvent>>();

        // Act: Run the terraforming system
        world.run_system_once(accidental_terraforming_system);

        // Assert: Biome should have changed to Ocean (or Tundra, depending on threshold mapping)
        let biome = world.get::<BiomeType>(planet).unwrap();
        assert_eq!(*biome, BiomeType::Ocean, "High heat on Ice should melt it to Ocean");

        // Ensure event was fired
        let events = world.resource::<Events<BiomeShiftEvent>>();
        let mut reader = events.get_reader();
        assert_eq!(reader.read(events).count(), 1, "Should emit a BiomeShiftEvent");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub enum BiomeType {
    Ice,
    Arid,
    Tundra,
    Ocean,
    Toxic,
}

#[derive(Component, Default)]
pub struct AtmosphericByproducts {
    pub heat: f32,
    pub toxins: f32,
}

#[derive(Component)]
pub struct Emissions {
    pub heat_per_tick: f32,
    pub toxins_per_tick: f32,
}

#[derive(Component)]
pub struct Building;

#[derive(Event)]
pub struct BiomeShiftEvent {
    pub planet: Entity,
    pub old_biome: BiomeType,
    pub new_biome: BiomeType,
}

pub fn accumulate_emissions_system(
    mut planets: Query<&mut AtmosphericByproducts>,
    buildings: Query<&Emissions, With<Building>>,
) {
    let mut total_heat = 0.0;
    let mut total_toxins = 0.0;

    for emission in buildings.iter() {
        total_heat += emission.heat_per_tick;
        total_toxins += emission.toxins_per_tick;
    }

    for mut byproducts in planets.iter_mut() {
        byproducts.heat += total_heat;
        byproducts.toxins += total_toxins;
    }
}

pub fn accidental_terraforming_system(
    mut commands: Commands,
    mut planets: Query<(Entity, &mut BiomeType, &mut AtmosphericByproducts)>,
    mut events: EventWriter<BiomeShiftEvent>,
) {
    for (entity, mut biome, mut byproducts) in planets.iter_mut() {
        let mut changed = false;
        let old_biome = *biome;

        // Example thresholds
        if *biome == BiomeType::Ice && byproducts.heat >= 1000.0 {
            *biome = BiomeType::Ocean;
            byproducts.heat -= 1000.0; // Consume the accumulated heat to threshold
            changed = true;
        } else if byproducts.toxins >= 1000.0 {
            *biome = BiomeType::Toxic;
            byproducts.toxins -= 1000.0;
            changed = true;
        }

        if changed {
            events.send(BiomeShiftEvent {
                planet: entity,
                old_biome,
                new_biome: *biome,
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Hardcoded thresholds (`1000.0`) in the terraforming system. These should be extracted to a `TerraformConfig` resource or component fields to allow different planet sizes or thresholds.
- **Multiple Planets**: The `accumulate_emissions_system` assumes all buildings contribute to all planets in the query. In a multi-planet system (Layer 2), buildings need to be linked to the specific planet they are on (e.g., via a `Parent` or `Location` component). For Layer 1, assuming one colony map, it's okay, but it should be noted.
- **Biome Transition Map**: Rather than hardcoded `if/else` logic, use a transition map or state machine (e.g., `Ice -> Tundra -> Ocean` based on heat).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] `accumulate_emissions_system` correctly aggregates emissions.
- [ ] `accidental_terraforming_system` shifts biome when threshold is reached.

## 7. Technical Guidance
- **Integration**: `AtmosphericByproducts` might overlap with existing `AtmosphereGrid` concepts. If `AtmosphereGrid` tracks per-tile pressure, this system might need to either sum the grid's total pressure/heat or operate on a separate global metric for the planet.
- **Lore/Chronicle**: Hook into the `BiomeShiftEvent` to write a Chronicle entry about the planet's irreversible change.

## 8. Questions
*Builder: add questions here if spec is unclear.*
