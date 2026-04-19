# Spec 1106: Photophobic Resources

## 1. Overview
**Layer:** 1
**Fantasy:** Mining in the pitch black. The moment you bring a torch, the treasure evaporates.
**Mechanic:** Certain deep-crust ores ("Shadow-Glass") degrade rapidly when exposed to any LightSource. They must be mined, transported, and processed in total darkness.
**Emergence:** You have to build a specialized, unlit logistics network just for this resource, forcing your pops to work in the dark, massively increasing their stress and accident rates.
**Tension:** Extreme resource value vs. Worker safety and sanity.

## 2. Dependencies
- Layer 1 Lighting System (`LightGrid`, `LightSource`)
- Layer 1 Mining and Resources
- Layer 1 Pop Stress/Health

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_shadow_glass_degrades_in_light() {
    // Arrange: Spawn Shadow-Glass resource in a lit tile
    // Act: Run the degradation system
    // Assert: The resource amount decreases over time
}

#[test]
fn test_shadow_glass_stable_in_darkness() {
    // Arrange: Spawn Shadow-Glass in total darkness
    // Act: Run the degradation system
    // Assert: The resource amount does not decrease
}

#[test]
fn test_pops_mining_in_dark_gain_stress() {
    // Arrange: Spawn a Pop mining in total darkness
    // Act: Run the mining and stress systems
    // Assert: Pop's stress increases significantly faster than normal
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN

#[derive(Component)]
pub struct PhotophobicResource {
    pub amount: f32,
    pub degradation_rate: f32,
}

#[derive(Component)]
pub struct LightLevel {
    pub intensity: f32,
}

pub fn photophobic_degradation_system(
    mut query: Query<(&mut PhotophobicResource, &LightLevel)>,
) {
    for (mut resource, light) in query.iter_mut() {
        if light.intensity > 0.0 {
            resource.amount -= resource.degradation_rate * light.intensity;
            if resource.amount < 0.0 {
                resource.amount = 0.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Degradation Effects:** When degrading, the resource could emit a toxic gas or bright light, causing a chain reaction if multiple are clustered.
- **Specialized Storage:** Allow specialized "Dark-Containers" to transport it through lit areas safely.
- **Refining:** The refinery for this resource also needs to operate in darkness.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Photophobic resources degrade when exposed to light.
- [ ] Pops mining without light suffer increased stress/accidents.

## 7. Technical Guidance
- Hook into the existing `LightGrid` or `LightingSystem`. The resource item itself needs a component to track light exposure.
- This creates a strong incentive to manage separate logistics networks, tying into hauling systems.

## 8. Questions
*Builder: add questions here if spec is unclear.*
