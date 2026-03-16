# ADR 039: Integrate Gut Biome Mechanics

**Status:** Accepted
**Date:** 2026-03-16

## Context

The simulation requires a layer of biological inertia applied to the food economy to prevent players from instantly swapping between drastically different food types without consequences. As outlined in Spec 211, sudden dietary shifts should cause indigestion and low nutrition yield, while maintaining a consistent diet should provide comfort and better nutritional outcomes.

A recent update (PR #1653) introduced the `layer1::gut_biome` module, mapping different item types to `BiomeCategory` (e.g., Plant, Meat, Synthetic) and providing a `GutBiome` component that tracks and adapts to the food consumed by a `Pop`. This ties into the existing `consume_food_system`, adjusting hunger gain and morale based on a Pop's familiarity with a food category.

## Decision

We have encapsulated the digestive adaptation logic into a dedicated module (`src/layer1/gut_biome.rs`) and a `GutBiome` component.

- `GutBiome` has been integrated into the `PopBundle`.
- The `consume_food_system` was updated to read the `GutBiome` component to calculate a digestion efficiency multiplier and trigger potential mood effects ("Gut Comfort" or "Indigestion").
- Food items are categorized via the `get_biome_category` helper, allowing integration with the Xeno-Gastronomy (ItemType classification) module.

## Consequences

- **Depth in Economy:** The food economy now punishes volatile switching of staple diets, rewarding players who establish and maintain reliable agricultural and synthetic supply chains.
- **Component Complexity:** Pops have an additional component (`GutBiome`) with a `HashMap` of familiarity values, slightly increasing memory overhead per Pop.
- **System Coupling:** The `consume_food_system` is now lightly coupled to `GutBiome`, though we handle missing components safely via `Option<&mut GutBiome>`.
- **Future Integration:** This lays the foundation for advanced sickness mechanics (e.g., extremely low familiarity triggering vomiting or health loss) and item-driven medical interventions (e.g., probiotics).
