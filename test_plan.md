Plan to implement 1030 Gravity Engineering:
1. `src/layer1/architecture/building.rs`: Add `pub struct Height { pub floors: u32 }` as a component.
2. `src/layer1/architecture/building.rs`: In `insert_base_building_components`, maybe add `Height { floors: 1 }` as default for buildings? The spec implies that tall buildings take structural damage. Or we just add it when spawning buildings. But the test just spawns manually, so we don't strictly need to add it to default spawn unless we want to. Let's add it to `insert_base_building_components` as `Height { floors: 1 }`. Wait, the spec's test just manually spawns. I'll add `Height` definition.
3. `src/layer1/architecture/building.rs`: Make `MaterialType` have an `is_reinforced(&self) -> bool` method. E.g., `match self { Self::Metal => true, _ => false }`. Wait, in the spec the RED phase explicitly shows `Material { is_reinforced: bool }`! "Material { is_reinforced: false }". I should implement exactly what the spec RED phase wrote, BUT `Material` is already `pub struct Material(pub MaterialType);`. I can add a `is_reinforced` method to `Material` instead and modify the RED phase test to use `Material(MaterialType::Wood)` to represent `is_reinforced: false`, and `Material(MaterialType::Metal)` to represent `is_reinforced: true`. Wait, the RED phase specifies:
```rust
Material { is_reinforced: false }
```
If I write this exact RED phase, it won't compile because `Material` is a tuple struct. I must change the test in the RED phase to match the real implementation. I will use `Material(MaterialType::Wood)` for non-reinforced, and `Material(MaterialType::Metal)` for reinforced, and define `impl Material { pub fn is_reinforced(&self) -> bool { self.0 == MaterialType::Metal } }`.

Wait! The damage event in the RED phase test:
```rust
use crate::layer1::health::{Health, DamageEvent};
```
But `DamageEvent` is actually in `crate::layer1::shields::DamageEvent` in this codebase! And for structures, we usually damage them directly via `structure.current_hp -= ...` or use a different damage system. The RED phase writes `DamageEvent { target, amount, source }` but `crate::layer1::shields::DamageEvent` is `pub struct DamageEvent { pub target: Entity, pub amount: f32, pub velocity: f32 }`. There is no `source` field! So the RED phase in the spec needs adapting.
I will adapt the GREEN phase to use `crate::layer1::shields::DamageEvent { target: event.building, amount: 1000.0, velocity: 0.0 }`. Wait, if `DamageEvent` goes to `shields::apply_damage_with_shields`, that only applies to entities with `Health`! Buildings have `Structure`.
Wait! How do structures take damage from events?
In `src/layer1/architecture/structure.rs`:
```rust
pub fn fire_damage_structure_system(world: &mut World) { ... }
```
There is no generic `DamageEvent` for structures! If I use `crate::layer1::shields::DamageEvent`, it won't affect structures because `apply_damage_with_shields` queries `Query<(&mut Health, Option<&mut KineticBarrier>)>`. Buildings don't have `Health`, they have `Structure`.
Therefore, my GREEN phase should directly apply the damage to the `Structure` component, OR define a new `StructureDamageEvent` if the spec wants an event. The spec RED phase creates `DamageEvent`, so I can define `pub struct StructureDamageEvent { pub target: Entity, pub amount: f32 }` or just use the system to modify `Structure` directly.
Let's look at the RED phase test again.
```rust
use crate::layer1::health::{Health, DamageEvent};
```
This is hallucinated by the spec. I will create a `GravityCollapseEvent` or simply check `Structure.current_hp` in the test instead of checking `Events<DamageEvent>`. Yes, the REFACTOR phase and the actual code should be idiomatic to Bevy and the codebase. I will adapt the RED phase test to assert on `Structure` HP.

Wait! The prompt says "Copy all test cases into the appropriate test module... Run `cargo test` - tests MUST fail".
If I copy them exactly, they won't even compile! "RED before GREEN — Write tests FIRST, watch them fail" "Must compile & test — Never commit failing tests or broken builds". This implies I MUST fix compilation errors in the RED phase tests so they can *fail* (by assertion) rather than failing to compile. I will adapt the tests to use `Structure` and `PlanetaryGravity` instead of `PlanetEnvironment`.

Let's check `PlanetaryGravity`. `src/layer2/syzygy.rs` defines `PlanetaryGravity { pub current: f32 }`.
Let's check `ConstructionEvent`. There is `BuildingCompletedEvent { pub entity: Entity }` in `src/layer1/core/events.rs`.

So the RED phase test should be adapted to:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::architecture::building::{Building, Material, MaterialType};
    use crate::layer1::architecture::structure::Structure;
    use crate::layer1::core::events::BuildingCompletedEvent;
    use crate::layer2::syzygy::PlanetaryGravity;

    #[test]
    fn test_building_exceeding_max_height_takes_structural_damage() {
        let mut app = App::new();
        app.insert_resource(PlanetaryGravity { current: 2.0 }); // High-G
        app.add_event::<BuildingCompletedEvent>();
        app.add_systems(Update, evaluate_structural_integrity_system);

        let tall_building = app.world_mut().spawn((
            Building { building_type: crate::layer1::architecture::building::BuildingType::Housing },
            Height { floors: 5 }, // Too tall for High-G standard materials
            Material(MaterialType::Wood), // not reinforced
            Structure { current_hp: 100.0, max_hp: 100.0 },
        )).id();

        app.world_mut().resource_mut::<Events<BuildingCompletedEvent>>().send(BuildingCompletedEvent {
            entity: tall_building,
        });

        app.update();

        let structure = app.world().get::<Structure>(tall_building).unwrap();
        assert!(structure.current_hp < 100.0, "A building exceeding the gravity height limit should suffer structural damage.");
    }

    #[test]
    fn test_reinforced_materials_bypass_height_limits() {
        let mut app = App::new();
        app.insert_resource(PlanetaryGravity { current: 2.0 });
        app.add_event::<BuildingCompletedEvent>();
        app.add_systems(Update, evaluate_structural_integrity_system);

        let reinforced_building = app.world_mut().spawn((
            Building { building_type: crate::layer1::architecture::building::BuildingType::Housing },
            Height { floors: 10 },
            Material(MaterialType::Metal), // reinforced
            Structure { current_hp: 100.0, max_hp: 100.0 },
        )).id();

        app.world_mut().resource_mut::<Events<BuildingCompletedEvent>>().send(BuildingCompletedEvent {
            entity: reinforced_building,
        });

        app.update();

        let structure = app.world().get::<Structure>(reinforced_building).unwrap();
        assert_eq!(structure.current_hp, 100.0, "Reinforced buildings should ignore gravity height limits.");
    }
}
```

Wait, what about the REFACTOR phase? "Progressive Collapse: Instead of instantly destroying the whole building, the system should only destroy the floors above the base_max_height."
If I just damage the structure HP (like `structure.current_hp -= 1000.0` or something), it works for the MVP. For the refactor, I can just reduce `height.floors = base_max_height` and apply damage.

So the plan:
1.  **RED Phase**:
    Create `src/layer1/architecture/gravity_engineering.rs`. Define `Height` component there or in `building.rs`. I'll define `Height` in `building.rs` so it's with other building components.
    I'll also implement `MaterialType::is_reinforced()` in `building.rs`.
    Add the failing tests in `src/layer1/architecture/gravity_engineering.rs`.
    Register module in `src/layer1/architecture/mod.rs`.
    Run `cargo test --lib layer1::architecture::gravity_engineering` and watch it fail. Commit.

2.  **GREEN Phase**:
    Implement `evaluate_structural_integrity_system` in `gravity_engineering.rs`.
    Make it pass. Commit.

3.  **REFACTOR Phase**:
    Make it progressively collapse: `height.floors = base_max_height` and deduct some HP instead of 1000.
    Actually, the REFACTOR phase says "destroy the floors above... This requires modifying the TerrainGrid Z-levels." But there is no Z-level system yet! So reducing `height.floors` is the closest we can get.
    And integrate the system into the main schedule in `src/simulation.rs`.
