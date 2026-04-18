# 1092 - Attention Bandwidth

## 1. Overview
The "Attention Bandwidth" feature introduces a "Focus" system where only a limited number of colonies or entities can provide precise statistics (like detailed Thoughts, precise Needs) to the player. Non-focused entities will only yield averaged, fuzzy, or delayed data. This enforces the fantasy that an Emperor cannot perfectly micro-manage an entire galaxy and adds tension: focusing on a frontier crisis may cause subtle, unobserved cultural drift or issues back at the supposedly stable capital.

## 2. Dependencies
- Cross-layer simulation (Needs to distinguish between Layer 1, Layer 2, Layer 3 context)
- Existing component tracking for Pops/Needs (e.g., `Hunger`, `Rest`, `Social`, `Thoughts`).
- System that queries this information for the UI.

## 3. RED Phase: Tests First

```rust
// tests/integration/attention_bandwidth.rs

#[test]
fn test_focus_limit_enforced() {
    // Arrange: Create an app, add the `AttentionBandwidthPlugin` or `AttentionResource`.
    // Act: Try to add more entities to the focus list than `MAX_ATTENTION_FOCUS`.
    // Assert: The list refuses the addition or evicts the oldest entry (LRU style).
}

#[test]
fn test_focused_entity_returns_precise_stats() {
    // Arrange: Entity A is in the Focus list. It has a Hunger need of exactly 42.5.
    // Act: Query its hunger via the UI/data abstraction layer.
    // Assert: The returned data is `Precise(42.5)`.
}

#[test]
fn test_unfocused_entity_returns_fuzzy_stats() {
    // Arrange: Entity B is NOT in the Focus list. It has a Hunger need of exactly 42.5.
    // Act: Query its hunger via the UI/data abstraction layer.
    // Assert: The returned data is `Fuzzy(Level::Moderate)` or `Averaged(40.0)`.
}

#[test]
fn test_cultural_drift_while_unfocused() {
    // Arrange: Entity C has a stable culture. It is removed from the Focus list.
    // Act: Run the simulation tick multiple times.
    // Assert: A "cultural drift" or "unobserved divergence" metric increases faster for unfocused entities than focused ones.
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/shared/attention.rs (or similar appropriate module)

// The simple abstraction for returned data
pub enum DataResolution<T> {
    Precise(T),
    Fuzzy(String), // or an enum like Level::Moderate
}

// Resource to track where the player's attention is
#[derive(Resource)]
pub struct AttentionFocus {
    pub max_focus: usize,
    pub focused_entities: std::collections::VecDeque<Entity>,
}

impl AttentionFocus {
    pub fn new(limit: usize) -> Self {
        Self {
            max_focus: limit,
            focused_entities: std::collections::VecDeque::with_capacity(limit),
        }
    }

    pub fn focus_on(&mut self, entity: Entity) {
        if !self.focused_entities.contains(&entity) {
            if self.focused_entities.len() >= self.max_focus {
                self.focused_entities.pop_front();
            }
            self.focused_entities.push_back(entity);
        }
    }

    pub fn is_focused(&self, entity: Entity) -> bool {
        self.focused_entities.contains(&entity)
    }
}

// Minimal query abstraction (example)
pub fn query_hunger(entity: Entity, hunger_comp: &Hunger, focus: &AttentionFocus) -> DataResolution<f32> {
    if focus.is_focused(entity) {
        DataResolution::Precise(hunger_comp.value)
    } else {
        // Simple fuzzy logic for MVP
        let fuzzy_str = if hunger_comp.value > 80.0 { "High" } else if hunger_comp.value < 20.0 { "Low" } else { "Moderate" };
        DataResolution::Fuzzy(fuzzy_str.to_string())
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: The UI rendering logic will need to be refactored to consume `DataResolution<T>` instead of raw components directly. This might be a wide-reaching change, so isolate it behind a clear trait or data access layer first.
- **Performance**: Querying `is_focused` repeatedly could be slow. Consider tagging focused entities with a `#[derive(Component)] struct IsFocused` marker component that is kept in sync with the `AttentionFocus` resource.
- **Cultural Drift**: Implement a background system (`unobserved_drift_system`) that iterates over Pops *without* the `IsFocused` marker and randomly alters their UtilityWeights or generates rumors over time.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new attention bandwidth module.
- [ ] UI abstractions correctly return precise data for focused entities and fuzzy data for unfocused entities.
- [ ] The `AttentionFocus` system enforces a maximum limit (e.g., 5-10 entities/colonies).
- [ ] Cultural drift/divergence events correctly trigger at an accelerated rate for unfocused elements.

## 7. Technical Guidance
- **Start Small**: Begin by applying the Attention Bandwidth logic to a single subsystem (e.g., Pop Needs or just the colony summary view) before rolling it out across the entire UI.
- **Marker Components**: Use an `IsFocused` component instead of looking up an entity in a resource list inside tight loops. Update this component whenever the player changes focus.
- **UI State**: The `ratatui` UI needs to gracefully handle `DataResolution::Fuzzy`. It should display things like "Hunger: ??? (Moderate)" instead of breaking.

## 8. Questions
*Builder: add questions here if spec is unclear.*
