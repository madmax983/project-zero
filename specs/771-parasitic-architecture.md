# 771: Parasitic Architecture

## 1. Overview
A sprawling, magnificent city that slowly digests its own foundations to stay aloft. Late-game "Megastructures" provide incredible colony-wide buffs but have a "Consumption" radius. Over time, they literally absorb the building materials and structural integrity of smaller, lower-tier buildings built beneath them to maintain themselves.

## 2. Dependencies
- Layer 1 Map Grid (`GridPosition`, `Building`, `StructuralIntegrity`)
- Layer 1 Construction System
- Time tracking resource (`SimulationTime`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::{GridPosition, Building, BuildingType, StructuralIntegrity};
    use crate::layer1::time::SimulationTime;

    #[test]
    fn test_megastructure_consumes_nearby_building_integrity() {
        let mut app = App::new();
        app.add_systems(Update, process_megastructure_consumption);

        let megastructure = app.world_mut().spawn((
            GridPosition { x: 10, y: 10 },
            Building { btype: BuildingType::Spire, ..default() },
            ParasiticArchitecture { radius: 2.0, consumption_rate: 10.0 },
        )).id();

        let victim = app.world_mut().spawn((
            GridPosition { x: 11, y: 10 },
            Building { btype: BuildingType::Housing, ..default() },
            StructuralIntegrity { current: 100.0, max: 100.0 },
        )).id();

        // Assume consumption event occurs every tick
        app.update();

        // Assert the victim building lost integrity
        let integrity = app.world().get::<StructuralIntegrity>(victim).unwrap();
        assert_eq!(integrity.current, 90.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::map::{GridPosition, Building, BuildingType, StructuralIntegrity};

#[derive(Component)]
pub struct ParasiticArchitecture {
    pub radius: f32,
    pub consumption_rate: f32,
}

pub fn process_megastructure_consumption(
    mut commands: Commands,
    megastructures: Query<(&GridPosition, &ParasiticArchitecture)>,
    mut buildings: Query<(Entity, &GridPosition, &mut StructuralIntegrity), Without<ParasiticArchitecture>>,
) {
    for (mega_pos, parasitic) in megastructures.iter() {
        for (entity, build_pos, mut integrity) in buildings.iter_mut() {
            let distance = ((mega_pos.x as f32 - build_pos.x as f32).powi(2) +
                            (mega_pos.y as f32 - build_pos.y as f32).powi(2)).sqrt();

            if distance <= parasitic.radius {
                integrity.current -= parasitic.consumption_rate;

                // Destroy if integrity is depleted
                if integrity.current <= 0.0 {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactor:** Avoid `O(N*M)` query comparisons by using a spatial hash map or querying grid tiles directly.
- **Smell:** Relying entirely on distance calculations per entity every tick is expensive. Pre-cache affected tiles when the megastructure is built.
- **Design:** Ensure that pops inside a building being consumed also suffer consequences (or are consumed directly if no buildings remain).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Megastructures slowly degrade the integrity of buildings within their radius.
- [ ] Buildings whose structural integrity reaches zero are successfully despawned.

## 7. Technical Guidance
- Combine the radius calculation with the colony grid logic to optimize spatial lookups.
- Emphasize visual feedback when a megastructure feeds (e.g., visual particles flowing towards the megastructure, warning notifications for players).

## 8. Questions
*Builder: add questions here if spec is unclear.*
