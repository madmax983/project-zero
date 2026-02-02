# Forge's Journal

**[Manual Copy Implementation]**
**Learning:** `#[derive(Copy)]` adds a `Copy` bound to generic type parameters even if they are only used in references. This can cause errors if the generic type (like `RandomState` in `HashMap`) is not `Copy`.
**Action:** When defining structs with references to generic types, implement `Clone` and `Copy` manually instead of deriving them, or pass by reference if appropriate.
