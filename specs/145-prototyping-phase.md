# 145: Prototyping Phase

## 1. Overview

The first few times a colony constructs a complex building, it is considered a "Prototype". Prototypes suffer from accelerated structural decay and higher malfunction rates due to the engineers learning on the job. Over time, as these prototypes operate, the colony gains "Mastery" over that specific building type. Once Mastery is achieved, subsequent constructions are "Standard" and suffer no penalties.

**Why:** Adds tension to early adoption of new technology. Discourages spamming new buildings immediately. Rewards keeping early, inefficient buildings running to "learn" the tech.

## 2. Dependencies

- `007` Building: Housing (Defines `Building` and `BuildingType`)
- `020` Construction Costs (Defines construction process)
- `112` Maintenance Debt (Defines `Structure` decay and `malfunction_system`)
- `029` Knowledge System (Optional, but conceptually related)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType, spawn_building}; // Adjust imports as needed
    use crate::layer1::structure::{Structure, entropy_system, calculate_malfunction_risk};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_first_construction_is_prototype() {
        let mut world = World::new();
        // Initialize resources
        world.insert_resource(BuildingMastery::default());
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());

        // Act: Spawn a new building type (e.g., Smelter)
        // Note: spawn_building signature might need checking in codebase
        let entity = spawn_building(&mut world, BuildingType::Smelter, crate::layer1::GridPosition { x: 0, y: 0 });

        // Assert: It has the Prototype component
        assert!(world.get::<Prototype>(entity).is_some());
    }

    #[test]
    fn test_mastery_accumulates_with_runtime() {
        let mut world = World::new();
        let mut mastery = BuildingMastery::default();
        world.insert_resource(mastery);

        // Arrange: Spawn a prototype
        let entity = world.spawn((
            Building { building_type: BuildingType::Smelter },
            Prototype,
        )).id();

        // Act: Run mastery system for 100 ticks
        for _ in 0..100 {
            update_mastery_system(&mut world);
        }

        // Assert: Mastery for Smelter has increased
        let mastery = world.resource::<BuildingMastery>();
        assert!(mastery.get_progress(BuildingType::Smelter) > 0.0);
    }

    #[test]
    fn test_mastery_prevents_prototype() {
        let mut world = World::new();
        let mut mastery = BuildingMastery::default();
        // Cheat: Set mastery to complete
        mastery.set_mastered(BuildingType::Smelter);
        world.insert_resource(mastery);
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());

        // Act: Spawn building
        let entity = spawn_building(&mut world, BuildingType::Smelter, crate::layer1::GridPosition { x: 0, y: 0 });

        // Assert: No Prototype component
        assert!(world.get::<Prototype>(entity).is_none());
    }

    #[test]
    fn test_prototype_accelerates_entropy() {
        let mut world = World::new();

        // Arrange: Two identical buildings, one prototype, one standard
        let standard = world.spawn((
            Structure { current_hp: 100.0, max_hp: 100.0 },
            Building { building_type: BuildingType::Smelter },
        )).id();

        let prototype = world.spawn((
            Structure { current_hp: 100.0, max_hp: 100.0 },
            Building { building_type: BuildingType::Smelter },
            Prototype,
        )).id();

        // Act: Run entropy system
        entropy_system(&mut world);

        // Assert: Prototype took more damage
        let s_std = world.get::<Structure>(standard).unwrap();
        let s_proto = world.get::<Structure>(prototype).unwrap();

        assert!(s_proto.current_hp < s_std.current_hp);
    }

    #[test]
    fn test_prototype_increases_malfunction_risk() {
        // We will test the helper function `calculate_malfunction_risk`.
        // Signature needs updating to accept is_prototype boolean

        let risk_std = calculate_malfunction_risk(100.0, 100.0, false); // No prototype
        let risk_proto = calculate_malfunction_risk(100.0, 100.0, true); // Prototype

        assert!(risk_proto > risk_std);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
/// Component marking a building as a prototype.
/// Prototypes suffer 2x entropy decay and 2x malfunction risk.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Prototype;

/// Resource tracking mastery over building types.
#[derive(Resource, Default, Debug)]
pub struct BuildingMastery {
    /// Progress towards mastery (0.0 to 1.0 or hours).
    /// Key: BuildingType, Value: Accumulated Runtime.
    pub progress: std::collections::HashMap<BuildingType, f32>,
}

impl BuildingMastery {
    pub const MASTERY_THRESHOLD: f32 = 1000.0; // Ticks or Hours

    pub fn get_progress(&self, b: BuildingType) -> f32 {
        *self.progress.get(&b).unwrap_or(&0.0)
    }

    pub fn is_mastered(&self, b: BuildingType) -> bool {
        self.get_progress(b) >= Self::MASTERY_THRESHOLD
    }

    pub fn add_progress(&mut self, b: BuildingType, amount: f32) {
        *self.progress.entry(b).or_insert(0.0) += amount;
    }

    // Test helper
    pub fn set_mastered(&mut self, b: BuildingType) {
        self.progress.insert(b, Self::MASTERY_THRESHOLD);
    }
}

/// Updates mastery based on active prototypes.
pub fn update_mastery_system(
    mut mastery: ResMut<BuildingMastery>,
    prototypes: Query<&Building, With<Prototype>>,
) {
    for building in &prototypes {
        // Gain 1.0 mastery per tick per prototype?
        // Or diminishing returns? For MVP, linear is fine.
        mastery.add_progress(building.building_type, 1.0);
    }
}

// In `spawn_building` (layer1/building.rs):
// ...
// let is_mastered = world.resource::<BuildingMastery>().is_mastered(building_type);
// if !is_mastered {
//     cmd.insert(Prototype);
// }

// In `entropy_system` (layer1/structure.rs):
// ...
// let is_prototype = prototype.is_some();
// let decay_mult = if is_prototype { 2.0 } else { 1.0 };
// structure.current_hp -= decay * modifier * decay_mult;

// Update `calculate_malfunction_risk` signature to accept `is_prototype: bool`.
// pub fn calculate_malfunction_risk(current: f32, max: f32, is_prototype: bool) -> f32 { ... }
```

## 5. REFACTOR Phase: Quality & Design

- **Balancing**: `MASTERY_THRESHOLD` should be tweaked based on tick rate (1 tick = ? time).
- **UI Feedback**: The player needs to know *why* the building is decaying fast. Add a "Prototype" tag in the Inspector.
- **Notification**: Notify when "Mastery Achieved" for a building type.
- **Save/Load**: Ensure `BuildingMastery` is serialized.
- **Performance**: `HashMap` lookup in `spawn_building` is negligible.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `BuildingMastery` resource exists and persists.
- [ ] First building of a type gets `Prototype` component.
- [ ] `Prototype` component doubles entropy decay.
- [ ] Mastery accumulates over time.
- [ ] After mastery, new buildings are normal.
- [ ] Existing prototypes remain prototypes (they don't magically upgrade).

## 7. Technical Guidance

- Modify `src/layer1/structure.rs` for `entropy_system` and `malfunction_system`.
- Modify `src/layer1/building.rs` for `spawn_building`.
- Create new module `src/layer1/mastery.rs` or add to `tech.rs` if small enough.
- Add `Prototype` to `src/ui/inspector.rs` logic to show "(Prototype)" label next to building name.
