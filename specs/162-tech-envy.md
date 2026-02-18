# 162 - Tech Envy

**Layer:** 1
**Status:** Draft
**Type:** Feature

## 1. Overview

As the colony advances technologically, Pops become aware of the disparity between "state-of-the-art" equipment and "legacy" tools. Pops assigned to work on outdated machines (Tier 1) when advanced machines (Tier 2+) of the same category exist in the colony will suffer a mood penalty called "Tech Envy" (or "Obsolescence").

This introduces a tension: Upgrading a single facility (e.g., for the Chief Engineer) creates resentment among the rest of the workforce, forcing the player to either upgrade everything (expensive) or deal with the morale hit.

## 2. Dependencies

- `specs/004-building-system.md` (Building types and entities)
- `specs/019-pop-thoughts.md` (Mood/Morale system)
- `specs/006-job-assignment.md` (Pops assigned to buildings)

## 3. RED Phase: Tests First

These tests should be placed in `src/layer1/tech_envy.rs` (or a test file).

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{BuildingType, Building, Category, Tier};
    use crate::layer1::morale::{Morale, MoodModifier};
    use crate::layer1::actions::{AssignedTo, AssignmentType};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_building_tiers_and_categories() {
        // Verify metadata exists
        assert_eq!(BuildingType::Farm.tier_info(), Some((Category::FoodProduction, Tier::Basic)));
        assert_eq!(BuildingType::HydroponicsBay.tier_info(), Some((Category::FoodProduction, Tier::Advanced)));
        assert_eq!(BuildingType::Housing.tier_info(), None); // Housing doesn't trigger envy (handled by Room Quality)
    }

    #[test]
    fn test_tech_envy_trigger() {
        let mut world = World::new();
        // Setup systems and resources
        world.insert_resource(crate::layer1::tech_envy::TechEnvyConfig::default());

        // 1. Spawn a Tier 1 Building (Farm)
        let farm = world.spawn((
            Building { building_type: BuildingType::Farm },
        )).id();

        // 2. Spawn a Pop assigned to the Farm
        let pop = world.spawn((
            AssignedTo { entity: farm, assignment_type: AssignmentType::FarmWorker },
            Morale::default(),
        )).id();

        // 3. Spawn a Tier 2 Building (HydroponicsBay) of the same category
        world.spawn((
            Building { building_type: BuildingType::HydroponicsBay },
        ));

        // 4. Run the system
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::tech_envy::tech_envy_system);
        schedule.run(&mut world);

        // 5. Assert Pop has "Obsolescence" modifier
        let morale = world.get::<Morale>(pop).unwrap();
        assert!(morale.modifiers.iter().any(|m| m.label == "Obsolescence"));
        assert!(morale.modifiers.iter().any(|m| m.value < 0.0));
    }

    #[test]
    fn test_no_envy_same_tier() {
        let mut world = World::new();

        // Spawn Tier 1 Farm and Pop
        let farm = world.spawn(Building { building_type: BuildingType::Farm }).id();
        let pop = world.spawn((
            AssignedTo { entity: farm, assignment_type: AssignmentType::FarmWorker },
            Morale::default(),
        )).id();

        // Spawn another Tier 1 Farm
        world.spawn(Building { building_type: BuildingType::Farm });

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::tech_envy::tech_envy_system);
        schedule.run(&mut world);

        // Assert NO modifier
        let morale = world.get::<Morale>(pop).unwrap();
        assert!(!morale.modifiers.iter().any(|m| m.label == "Obsolescence"));
    }

    #[test]
    fn test_no_envy_different_category() {
        let mut world = World::new();

        // Spawn Tier 1 Farm and Pop
        let farm = world.spawn(Building { building_type: BuildingType::Farm }).id();
        let pop = world.spawn((
            AssignedTo { entity: farm, assignment_type: AssignmentType::FarmWorker },
            Morale::default(),
        )).id();

        // Spawn Tier 3 Power Building (AncientReactor)
        world.spawn(Building { building_type: BuildingType::AncientReactor });

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::tech_envy::tech_envy_system);
        schedule.run(&mut world);

        // Assert NO modifier
        let morale = world.get::<Morale>(pop).unwrap();
        assert!(!morale.modifiers.iter().any(|m| m.label == "Obsolescence"));
    }

    #[test]
    fn test_envy_clears_when_condition_met() {
        // Setup: Pop has envy.
        // Action: Upgrade Pop's building (or move Pop).
        // Assert: Envy removed (or decays).
        // Since modifiers have duration, we just check that the system doesn't re-apply it.
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Define Tiers and Categories
In `src/layer1/building.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    Basic = 1,
    Advanced = 2,
    HighTech = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    FoodProduction,
    Manufacturing,
    Power,
    Research,
}

impl BuildingType {
    pub fn tier_info(&self) -> Option<(Category, Tier)> {
        match self {
            Self::Farm => Some((Category::FoodProduction, Tier::Basic)),
            Self::Greenhouse => Some((Category::FoodProduction, Tier::Advanced)),
            Self::HydroponicsBay => Some((Category::FoodProduction, Tier::HighTech)),

            Self::Smithy | Self::LumberMill | Self::StoneMason => Some((Category::Manufacturing, Tier::Basic)),
            Self::Smelter | Self::Refinery => Some((Category::Manufacturing, Tier::Advanced)), // Assuming Smelter is better? Or maybe Smelter is Basic and Fabricator is HighTech?
            Self::AncientFabricator => Some((Category::Manufacturing, Tier::HighTech)),

            Self::Generator => Some((Category::Power, Tier::Basic)),
            Self::AncientReactor => Some((Category::Power, Tier::HighTech)),

            Self::Library => Some((Category::Research, Tier::Basic)),
            Self::Observatory => Some((Category::Research, Tier::Advanced)),
            Self::AICore => Some((Category::Research, Tier::HighTech)), // Or similar

            _ => None,
        }
    }
}
```

### 2. Tech Envy System
In `src/layer1/tech_envy.rs`:

```rust
pub fn tech_envy_system(
    mut pops: Query<(&AssignedTo, &mut Morale)>,
    buildings: Query<&Building>,
) {
    // 1. Calculate Max Tier per Category
    let mut max_tiers = HashMap::new();
    for building in buildings.iter() {
        if let Some((category, tier)) = building.building_type.tier_info() {
             let current_max = max_tiers.entry(category).or_insert(tier);
             if tier > *current_max {
                 *current_max = tier;
             }
        }
    }

    // 2. Check Pops
    for (assignment, mut morale) in pops.iter_mut() {
        if let Ok(building) = buildings.get(assignment.entity) {
            if let Some((category, tier)) = building.building_type.tier_info() {
                if let Some(&max_tier) = max_tiers.get(&category) {
                    if tier < max_tier {
                        // Apply Envy
                        morale.add_modifier(MoodModifier {
                            label: "Obsolescence".to_string(),
                            value: -0.1 * (max_tier as i32 - tier as i32) as f32, // Scales with gap
                            duration: 10, // Short duration, refreshed constantly
                        });
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase

- **Performance**: Iterating all buildings every frame is O(N). For a large colony, this is fine, but caching via a resource `MaxTechLevels` that updates only when `Building` components are added/removed would be better.
- **Deduplication**: Ensure multiple modifiers don't stack infinitely. `Morale` system might simply list all modifiers. If `Obsolescence` is added every tick, the list grows.
    - *Fix*: The system should check if `Obsolescence` already exists and just refresh duration, OR clear old ones.
    - *Better Fix*: `Morale` system handles decay. `TechEnvy` system should only add if not present, or `MoodModifier` should have unique ID/Type to prevent stacking.
    - *Simplest*: `morale.modifiers.retain(|m| m.label != "Obsolescence");` before adding.

## 6. Acceptance Criteria

- [ ] `BuildingType` has `tier_info()` method returning correct Category/Tier.
- [ ] Pops working at lower-tier buildings receive "Obsolescence" mood penalty when higher-tier exists.
- [ ] Penalty scales with the tier gap (Tier 1 vs Tier 3 is worse than Tier 1 vs Tier 2).
- [ ] Penalty does not apply if only lower-tier buildings exist.
- [ ] Penalty does not cross categories (Power plants don't make Farmers jealous).
- [ ] Test coverage > 85%.

## 7. Technical Guidance

- Use `strum` or `Ord` derivation for `Tier` comparison.
- Ensure `tech_envy_system` runs in the main simulation loop (e.g., `Update`).
- Be careful with `AssignedTo` pointing to non-existent entities (use `get`).
