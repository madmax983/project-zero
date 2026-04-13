# 1001: Vacuum Welding

## 1. Overview
Building structures that are meant to last forever in the void. Structures built in Vacuum biomes have 2x HP and 0.5x Build Time because metals cold-weld instantly. However, they cannot be Deconstructed or Repaired (fused solid). To remove them, you must Destroy them (yielding no resources).

## 2. Dependencies
- Layer 1 Biome/Atmosphere mechanic (specifically Vacuum).
- Layer 1 Construction system (Build Time, Deconstruct, Repair, Destroy).
- Layer 1 Structures (HP).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_vacuum_welding_build_time_and_hp() {
        let mut app = App::new();
        app.add_systems(Update, apply_vacuum_welding);

        // Spawn a structure in a vacuum biome
        let structure = app.world_mut().spawn((
            Structure { hp: 100.0, max_hp: 100.0 },
            ConstructionState { build_time_remaining: 10.0, total_build_time: 10.0 },
            InBiome(BiomeType::Vacuum),
            PendingVacuumWelding, // Marker to indicate it needs processing
        )).id();

        app.update();

        let s = app.world().get::<Structure>(structure).unwrap();
        assert_eq!(s.max_hp, 200.0);
        assert_eq!(s.hp, 200.0);

        let c = app.world().get::<ConstructionState>(structure).unwrap();
        assert_eq!(c.total_build_time, 5.0);
        assert_eq!(c.build_time_remaining, 5.0);
    }

    #[test]
    fn test_vacuum_welding_prevents_repair_and_deconstruct() {
        let mut app = App::new();
        app.add_systems(Update, handle_repair_orders);
        app.add_systems(Update, handle_deconstruct_orders);

        let structure = app.world_mut().spawn((
            Structure { hp: 50.0, max_hp: 200.0 },
            VacuumWelded,
        )).id();

        // Issue repair order
        app.world_mut().spawn(RepairOrder { target: structure });

        // Issue deconstruct order
        app.world_mut().spawn(DeconstructOrder { target: structure });

        app.update();

        // The orders should be rejected/fail
        // Check that HP is not increased (repair failed)
        let s = app.world().get::<Structure>(structure).unwrap();
        assert_eq!(s.hp, 50.0);

        // Check that it's not marked for deconstruction
        assert!(app.world().get::<Deconstructing>(structure).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Structure {
    pub hp: f32,
    pub max_hp: f32,
}

#[derive(Component)]
pub struct ConstructionState {
    pub build_time_remaining: f32,
    pub total_build_time: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BiomeType {
    Vacuum,
    Atmosphere,
}

#[derive(Component)]
pub struct InBiome(pub BiomeType);

#[derive(Component)]
pub struct PendingVacuumWelding;

#[derive(Component)]
pub struct VacuumWelded;

#[derive(Component)]
pub struct RepairOrder {
    pub target: Entity,
}

#[derive(Component)]
pub struct DeconstructOrder {
    pub target: Entity,
}

#[derive(Component)]
pub struct Deconstructing;

pub fn apply_vacuum_welding(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Structure, &mut ConstructionState, &InBiome), With<PendingVacuumWelding>>,
) {
    for (entity, mut structure, mut construction, biome) in query.iter_mut() {
        if biome.0 == BiomeType::Vacuum {
            structure.max_hp *= 2.0;
            structure.hp *= 2.0;
            construction.total_build_time *= 0.5;
            construction.build_time_remaining *= 0.5;
            commands.entity(entity).insert(VacuumWelded);
        }
        commands.entity(entity).remove::<PendingVacuumWelding>();
    }
}

pub fn handle_repair_orders(
    mut commands: Commands,
    orders: Query<(Entity, &RepairOrder)>,
    targets: Query<&VacuumWelded>,
    mut structures: Query<&mut Structure>,
) {
    for (order_entity, order) in orders.iter() {
        if targets.contains(order.target) {
            // Cannot repair vacuum welded structures
            commands.entity(order_entity).despawn();
            continue;
        }

        if let Ok(mut structure) = structures.get_mut(order.target) {
            structure.hp = structure.max_hp; // Simple repair logic
        }
        commands.entity(order_entity).despawn();
    }
}

pub fn handle_deconstruct_orders(
    mut commands: Commands,
    orders: Query<(Entity, &DeconstructOrder)>,
    targets: Query<&VacuumWelded>,
) {
    for (order_entity, order) in orders.iter() {
        if targets.contains(order.target) {
            // Cannot deconstruct vacuum welded structures
            commands.entity(order_entity).despawn();
            continue;
        }

        commands.entity(order.target).insert(Deconstructing);
        commands.entity(order_entity).despawn();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: `apply_vacuum_welding` mutates states. It might be better to use Bevy's `Event` system for starting construction and applying biome effects at spawn time rather than checking a `PendingVacuumWelding` marker.
- **Design Improvements**: The `handle_repair_orders` and `handle_deconstruct_orders` are very rudimentary here. In reality, preventing an action should provide UI feedback to the player. The UI should likely query `Has<VacuumWelded>` to disable the repair/deconstruct buttons entirely before an order is even issued.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Structures in Vacuum biome have 2x HP and 0.5x build time.
- [ ] Structures with `VacuumWelded` cannot be repaired.
- [ ] Structures with `VacuumWelded` cannot be deconstructed (must be destroyed).

## 7. Technical Guidance
- Integrate with `layer1::construction` module.
- Add UI checks to disable the Deconstruct and Repair buttons for `VacuumWelded` entities.
- Ensure the 'Destroy' action bypasses the `VacuumWelded` restriction but yields 0 resources.

## 8. Questions
*Builder: add questions here if spec is unclear.*
