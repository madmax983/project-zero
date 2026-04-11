# Builder Insight: Generational Dissonance (Spec 800)

## Pattern: Context Mapping for Utility AI Modifiers
When modifying Utility AI choices (like `ignores_safety`), it's safest to define a marker component (e.g., `EdictCompliance`) and update it in the generic observation step (`src/layer1/systems/observation.rs`), rather than baking custom `Generation` queries deeply into `utility_ai.rs` hot loops.

## Anti-Pattern Avoidance: Bevy Change Detection Flooding
When iterating over entities to update a component value conditionally (e.g., setting `ignores_safety = true`), it's critical to check if the value is already set before applying the mutation (e.g., `if !compliance.ignores_safety { compliance.ignores_safety = true; }`). This prevents Bevy from triggering change detection (`Changed<EdictCompliance>`) on every single frame, preserving performance.

## Anti-Pattern Avoidance: Infinite Vector Growth
Similarly, when adding items to a collection (like `morale.modifiers`) inside a system that runs every frame/tick, check if an equivalent item already exists (e.g., by matching on a `label` or `source` string). If not checked, the vector will grow infinitely and trigger a memory leak and extreme slowdowns.
