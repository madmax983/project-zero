1.  **Add `Height` Component**: In `src/layer1/architecture/building.rs`, define a `Height` component to track the number of floors a building has. It needs to derive `Component`, `Debug`, `Clone`, and `Copy`.
    ```rust
    #[derive(Component, Debug, Clone, Copy)]
    pub struct Height {
        pub floors: u32,
    }
    ```
2.  **Add `Material::is_reinforced` capability**: In `src/layer1/architecture/building.rs`, add a method to `MaterialType` to check if it's reinforced (e.g., `Metal` might be reinforced, or we can add a new material, or check if it's `Metal`). Wait, the spec says "constructed using expensive 'Reinforced Materials'". Maybe we should just add `pub is_reinforced: bool` to the existing `Material` component or add an `is_reinforced(&self)` method to `MaterialType`. The test says `Material { is_reinforced: false }`. We need to change `Material` from a tuple struct `Material(pub MaterialType)` to a struct with `is_reinforced` field? Let's check how `Material` is defined. It is `pub struct Material(pub MaterialType);`. The spec RED phase specifically expects: `Material { is_reinforced: bool }`. I will change `Material` or just follow the spec to create a `Height` struct and modify `Material` if needed, or simply make `is_reinforced` a property. Wait! The RED phase explicitly writes: `Material { is_reinforced: false }`. This means I need to modify `Material` or create a new one. But modifying `Material` might break existing usages! Let's check `Material`'s current usages.

3.  **Implement `evaluate_structural_integrity_system`**: In a new file `src/layer1/architecture/gravity_engineering.rs` (or similar, or `src/layer1/physics/gravity_engineering.rs`), implement the structural integrity logic.
    - Inverse relationship: `let base_max_height = (10.0 / planet.gravity).floor() as u32;`
    - Iterate over `ConstructionEvent` (Wait, it's `BuildingCompletedEvent` in the actual codebase!). The spec uses `ConstructionEvent`. I should use `BuildingCompletedEvent` since that's what the codebase actually uses, but I'll write the RED phase tests exactly as in the spec, substituting `ConstructionEvent` with `BuildingCompletedEvent` if necessary to make it compile with the real codebase, or I can define `ConstructionEvent` if it doesn't exist. Actually, `BuildingCompletedEvent` exists and is sent when a building is completed. I will use `BuildingCompletedEvent`.

4.  **Register the new system**: In `src/layer1/core/integration.rs` or `src/simulation.rs`, register the `evaluate_structural_integrity_system` so it actually runs.

5.  **Refactoring / Adjusting for the Codebase**: The spec uses `Material { is_reinforced: false }`. Since `Material` is already `pub struct Material(pub MaterialType)`, I will add `is_reinforced: bool` to it: `pub struct Material { pub material_type: MaterialType, pub is_reinforced: bool }` - BUT this would break everything that uses `Material(MaterialType)`. A better way is to provide `impl Material { pub fn is_reinforced(&self) -> bool { ... } }` and in the test write `Material(MaterialType::Wood)` for not reinforced and `Material(MaterialType::Metal)` for reinforced? Let's see if the spec requires exact struct fields in tests. The spec tests are just a guide for RED phase. I'll adapt them to the codebase's actual types.

Wait, if I change `Material` to a normal struct, `grep -rn "Material(" src/layer1/` will show where it's used. Let's check usages of `Material(`.
