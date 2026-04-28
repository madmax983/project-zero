# 1244: Acoustic Ecology

## 1. Overview
**Layer:** 1
**Fantasy:** A colony where sound is a physical force that shapes the environment.
**Mechanic:** Different buildings emit specific frequencies. "Acoustic Harvesters" can convert noise pollution into low-grade energy, but conflicting frequencies create "Resonance Zones" that damage fragile structures and cause chronic headaches in pops.

## 2. Dependencies
- Layer 1 Building/Structure System
- Layer 1 Resource Grid / Power Grid System
- Layer 1 Pop Health/Mood System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_acoustic_harvester_generates_energy() {
        // Arrange
        let mut app = App::new();
        app.init_resource::<PowerGrid>();

        let _factory = app.world_mut().spawn((
            NoiseEmitter { frequency: 440.0, volume: 50.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        let _harvester = app.world_mut().spawn((
            AcousticHarvester { efficiency: 0.5 },
            GridPosition { x: 1, y: 0 }, // Adjacent
        )).id();

        // Act
        app.add_systems(Update, acoustic_harvesting_system);
        app.update();

        // Assert
        let power = app.world().resource::<PowerGrid>();
        assert!(power.available_energy > 0.0, "Acoustic Harvester should generate energy from nearby noise");
    }

    #[test]
    fn test_resonance_zone_damages_fragile_structures() {
        // Arrange
        let mut app = App::new();

        // Two conflicting frequencies creating a resonance zone
        app.world_mut().spawn((NoiseEmitter { frequency: 440.0, volume: 100.0 }, GridPosition { x: 0, y: 0 }));
        app.world_mut().spawn((NoiseEmitter { frequency: 445.0, volume: 100.0 }, GridPosition { x: 2, y: 0 }));

        let fragile_structure = app.world_mut().spawn((
            Structure { current_hp: 100.0, is_fragile: true },
            GridPosition { x: 1, y: 0 }, // In the middle
        )).id();

        // Act
        app.add_systems(Update, calculate_resonance_damage_system);
        app.update();

        // Assert
        let structure = app.world().get::<Structure>(fragile_structure).unwrap();
        assert!(structure.current_hp < 100.0, "Fragile structures should take damage in resonance zones");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct NoiseEmitter {
    pub frequency: f32,
    pub volume: f32,
}

#[derive(Component)]
pub struct AcousticHarvester {
    pub efficiency: f32,
}

#[derive(Component)]
pub struct Structure {
    pub current_hp: f32,
    pub is_fragile: bool,
}

#[derive(Resource, Default)]
pub struct PowerGrid {
    pub available_energy: f32,
}

pub fn acoustic_harvesting_system(
    emitters: Query<(&GridPosition, &NoiseEmitter)>,
    harvesters: Query<(&GridPosition, &AcousticHarvester)>,
    mut power: ResMut<PowerGrid>,
) {
    for (h_pos, harvester) in harvesters.iter() {
        for (e_pos, emitter) in emitters.iter() {
            // Check if adjacent (distance squared <= 1)
            let dist_sq = (h_pos.x - e_pos.x).pow(2) + (h_pos.y - e_pos.y).pow(2);
            if dist_sq <= 1 {
                power.available_energy += emitter.volume * harvester.efficiency;
            }
        }
    }
}

pub fn calculate_resonance_damage_system(
    emitters: Query<(&GridPosition, &NoiseEmitter)>,
    mut structures: Query<(&GridPosition, &mut Structure)>,
) {
    for (s_pos, mut structure) in structures.iter_mut() {
        if !structure.is_fragile { continue; }

        let mut conflicting_noise = 0;
        for (e_pos, _) in emitters.iter() {
            // Simplified check: If within range of multiple emitters, assume resonance
            let dist_sq = (s_pos.x - e_pos.x).pow(2) + (s_pos.y - e_pos.y).pow(2);
            if dist_sq <= 2 {
                conflicting_noise += 1;
            }
        }

        if conflicting_noise > 1 {
            structure.current_hp -= 10.0; // Damage from resonance
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate actual beat frequencies or interference patterns instead of just counting nearby emitters for `calculate_resonance_damage_system`.
- Noise should fall off over distance rather than checking a hardcoded radius.
- Affect pops: Pops within resonance zones should receive a "Headache" debuff that lowers morale and work efficiency.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Acoustic harvesters generate energy from adjacent noise emitters.
- [ ] Conflicting noise frequencies damage fragile structures.

## 7. Technical Guidance
- **Spatial Queries:** If the number of emitters and buildings grows, nested loops in these systems will become an O(N*M) bottleneck. Consider using a `SpatialGrid` resource to query nearby entities quickly.
- **Visuals:** Add a debug or overlay view to visualize the "noise map" across the colony to help the player plan their layout.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
