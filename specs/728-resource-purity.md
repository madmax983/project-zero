# Spec 728: Resource Purity

## 1. Overview
**Layer:** 1
**Fantasy:** Not all dirt is created equal.
**Mechanic:** Ore nodes spawn with a "Purity" % (e.g., Iron 40%, Slag 60%). Low purity nodes produce more "Waste" items during refining. High purity nodes are rare and contested. Technology can improve extraction efficiency.
**Emergence:** You build a massive industrial complex on a convenient Iron deposit, only to realize it's 10% purity. Your base floods with slag, blocking all stockpiles.
**Tension:** Exploit the nearby low-quality source or travel for the high-quality one?

## 2. Dependencies
- Base simulation framework
- Map and Grid systems
- Resource Stockpiles (`022`)
- Refining Industry (`023`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::resources::{ResourceNode, ResourceType};
    use crate::layer1::refining::Refinery;

    #[test]
    fn test_resource_purity_yields_correct_ratio() {
        // Arrange
        let mut world = World::new();
        // A node with 40% purity
        let node_id = world.spawn(ResourceNode {
            resource_type: ResourceType::Iron,
            amount: 100,
            purity: 0.4,
        }).id();

        let mut refinery = Refinery::new();

        // Act
        // Process 10 units of ore from the node
        let (refined, waste) = refinery.process_ore(&world.get::<ResourceNode>(node_id).unwrap(), 10);

        // Assert
        // 40% purity of 10 units = 4 refined, 6 waste
        assert_eq!(refined, 4);
        assert_eq!(waste, 6);
    }

    #[test]
    fn test_resource_purity_high_efficiency_tech() {
        // Arrange
        let mut world = World::new();
        // A node with 40% purity
        let node_id = world.spawn(ResourceNode {
            resource_type: ResourceType::Iron,
            amount: 100,
            purity: 0.4,
        }).id();

        let mut refinery = Refinery::new();
        // Apply tech bonus (+10% effective purity)
        refinery.set_efficiency_bonus(0.1);

        // Act
        // Process 10 units of ore from the node
        let (refined, waste) = refinery.process_ore(&world.get::<ResourceNode>(node_id).unwrap(), 10);

        // Assert
        // (40% + 10%) of 10 units = 5 refined, 5 waste
        assert_eq!(refined, 5);
        assert_eq!(waste, 5);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// In src/layer1/resources.rs or similar
pub struct ResourceNode {
    pub resource_type: ResourceType,
    pub amount: u32,
    pub purity: f32, // 0.0 to 1.0
}

// In src/layer1/refining.rs or similar
pub struct Refinery {
    pub efficiency_bonus: f32,
}

impl Refinery {
    pub fn new() -> Self {
        Self { efficiency_bonus: 0.0 }
    }

    pub fn set_efficiency_bonus(&mut self, bonus: f32) {
        self.efficiency_bonus = bonus;
    }

    pub fn process_ore(&self, node: &ResourceNode, input_amount: u32) -> (u32, u32) {
        let effective_purity = (node.purity + self.efficiency_bonus).clamp(0.0, 1.0);
        let refined = (input_amount as f32 * effective_purity).round() as u32;
        let waste = input_amount.saturating_sub(refined);
        (refined, waste)
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate this properly with Bevy ECS by creating a `RefiningSystem` rather than an isolated struct, where `Refinery` is a component on a building entity.
- Extract magic numbers into constants or configuration.
- Handle fractional outputs over multiple ticks properly (maybe accumulating fractional progress rather than rounding).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code

## 7. Technical Guidance
- Integrate into the appropriate Layer simulation schedule (`update_refineries`).
- Consider how this interacts with hauling—does the player haul raw ore or wait to refine it? Assuming we haul raw ore to the refinery, the raw ore item itself needs to track its purity, or we abstract it and track average purity in stockpiles. For minimal implementation, it's easiest if raw ore items track the purity of the node they came from.

## 8. Questions
*Builder: add questions here if spec is unclear.*
