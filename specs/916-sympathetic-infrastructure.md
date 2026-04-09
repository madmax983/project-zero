# 916: The Sympathetic Infrastructure

## 1. Overview
Buildings that reflect the collective mood of the colony. Advanced biomimetic structures subtly adjust their environment (temperature, lighting) based on the average morale of the Pops inside them. This creates emergent feedback loops where extremely low morale can accidentally destroy climate-sensitive things (like agricultural flora) by triggering a "hibernation" reflex in the building itself.

## 2. Dependencies
- `layer1::infrastructure` (Building, Biomimetic)
- `layer1::pops` (Pop, Morale)
- `layer1::environment` (Temperature)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::infrastructure::{Building, Biomimetic};
    use crate::layer1::pops::{Pop, Morale};
    use crate::layer1::environment::Temperature;

    #[test]
    fn test_sympathetic_temperature_drop_low_morale() {
        let mut app = App::new();
        app.add_systems(Update, adjust_sympathetic_infrastructure_system);

        // Spawn a biomimetic building
        let building_entity = app.world_mut().spawn((
            Building,
            Biomimetic,
            Temperature { value: 20.0 },
        )).id();

        // Spawn a pop with very low morale associated with the building (simulated via query iteration here, or parenting in full impl)
        app.world_mut().spawn((
            Pop,
            Morale { value: 5.0, threshold: 20.0 }, // Very sad
            // Parent(building_entity) or similar connection component
        ));

        app.update();

        // Temperature should have plummeted due to sympathy with the sad pop
        let temp = app.world().get::<Temperature>(building_entity).unwrap();
        assert!(temp.value < 20.0);
    }

    #[test]
    fn test_sympathetic_temperature_rise_high_morale() {
        let mut app = App::new();
        app.add_systems(Update, adjust_sympathetic_infrastructure_system);

        let building_entity = app.world_mut().spawn((
            Building,
            Biomimetic,
            Temperature { value: 20.0 },
        )).id();

        // Spawn a pop with high morale
        app.world_mut().spawn((
            Pop,
            Morale { value: 90.0, threshold: 20.0 }, // Very happy
        ));

        app.update();

        // Temperature should rise (or remain optimal)
        let temp = app.world().get::<Temperature>(building_entity).unwrap();
        assert!(temp.value > 20.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::infrastructure::{Building, Biomimetic};
use crate::layer1::pops::{Pop, Morale};
use crate::layer1::environment::Temperature;

pub fn adjust_sympathetic_infrastructure_system(
    mut building_query: Query<&mut Temperature, (With<Building>, With<Biomimetic>)>,
    pop_query: Query<&Morale, With<Pop>>,
) {
    // In a real implementation, you'd match pops to their specific building.
    // For this minimal pass, we average global morale.
    let mut total_morale = 0.0;
    let mut pop_count = 0;

    for morale in pop_query.iter() {
        total_morale += morale.value;
        pop_count += 1;
    }

    if pop_count == 0 { return; }

    let average_morale = total_morale / (pop_count as f32);

    for mut temp in building_query.iter_mut() {
        if average_morale < 20.0 {
            // Sadness triggers cold hibernation
            temp.value -= 5.0;
        } else if average_morale > 70.0 {
            // Happiness triggers warmth
            temp.value += 5.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Localization:** The `GREEN` phase averages *global* morale. The refactored version *must* localize this to the specific Pops housed or working in that specific building entity (e.g., via `Parent` or a `HousedIn` component).
- **Rate of Change:** Temperature shouldn't drop infinitely. It needs a min/max clamp based on the severity of the morale.
- **Secondary Effects:** The system should probably emit an event (e.g., `BiomimeticShiftEvent`) so other systems (like agriculture) can react to the sudden temperature change without polling.

## 6. Acceptance Criteria
- [ ] TDD Tests written and passing.
- [ ] Test coverage >= 85%.
- [ ] Biomimetic buildings adjust their temperature based on the localized average morale of their occupants.
- [ ] Temperature changes correctly affect secondary systems (like food production/plant death) if applicable.

## 7. Technical Guidance
- Implement in `src/layer1/infrastructure/biomimetic.rs`.
- Ensure you query for the relationship between Pops and Buildings properly (e.g., children of the building entity, or referencing the building's entity ID).

## 8. Questions
*Builder: add questions here if spec is unclear.*
