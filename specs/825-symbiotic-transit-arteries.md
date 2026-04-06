# 825: Symbiotic Transit Arteries

## 1. Overview
Instead of constructing cold metal conveyor belts or pneumatic tubes, colonies can cultivate "Transit Arteries"—massive, hollow, biological worm-like structures that move items and Pops rapidly across the colony. They are cheap to maintain, self-repairing, and use organic waste as fuel. However, they are living organisms. If they are routed through extreme temperatures or subjected to toxic industrial byproducts, they can "fall sick" or suffer chemical shock. When severely poisoned, an Artery will violently "cough," ejecting its entire transit payload—which might include highly refined electronics and commuting workers—into a toxic slag heap, causing massive economic and localized casualty events.

## 2. Dependencies
- `src/layer1/logistics/transit.rs` for routing and item/Pop movement mechanics.
- `src/layer1/buildings.rs` for organic/symbiotic structure types.
- `src/layer1/nature/environment.rs` or `pollution.rs` for tracking toxic tile status.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_artery_absorbs_toxicity_from_environment() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<PollutionGrid>();
        app.add_systems(Update, process_artery_toxicity);

        app.world_mut().resource_mut::<PollutionGrid>().set_toxicity_at(5, 5, 100); // Highly toxic tile

        let artery = app.world_mut().spawn((
            TransitArtery { toxicity_level: 0 },
            Position { x: 5, y: 5 },
        )).id();

        // Act
        app.update();

        // Assert
        let artery_data = app.world().get::<TransitArtery>(artery).unwrap();
        // The artery should absorb toxicity from the underlying polluted tile
        assert!(artery_data.toxicity_level > 0);
    }

    #[test]
    fn test_poisoned_artery_coughs_payload() {
        // Test that if an Artery's toxicity_level exceeds a critical threshold,
        // it forcibly ejects all `TransitPayload` entities currently traversing it,
        // placing them on the ground and applying damage/stress to Pops.
    }

    #[test]
    fn test_artery_self_repairs_over_time() {
        // Test that an Artery slowly reduces its damage/toxicity over time if placed in a clean environment,
        // unlike mechanical transit lines which require active repair jobs.
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct TransitArtery {
    pub toxicity_level: u32,
}

#[derive(Resource, Default)]
pub struct PollutionGrid {
    pub toxicity: std::collections::HashMap<(i32, i32), u32>,
}

impl PollutionGrid {
    pub fn get_toxicity_at(&self, x: i32, y: i32) -> u32 {
        *self.toxicity.get(&(x, y)).unwrap_or(&0)
    }

    pub fn set_toxicity_at(&mut self, x: i32, y: i32, amount: u32) {
        self.toxicity.insert((x, y), amount);
    }
}

pub fn process_artery_toxicity(
    grid: Res<PollutionGrid>,
    mut query: Query<(&mut TransitArtery, &Position)>,
) {
    for (mut artery, pos) in query.iter_mut() {
        let local_toxicity = grid.get_toxicity_at(pos.x, pos.y);
        if local_toxicity > 50 {
            // Artery absorbs toxicity from the ground
            artery.toxicity_level += 5;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create an `ArteryCoughEvent` to handle the complex logic of ejecting payloads, dropping items, and stressing Pops without cluttering the toxicity update system.
- Ensure the routing algorithm recalculates if an Artery temporarily shuts down due to toxicity.
- Hook into the waste management systems so Arteries can actively consume `OrganicWaste` as their primary fuel/upkeep.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] `TransitArtery` component reliably increases its `toxicity_level` when resting on heavily polluted tiles.

## 7. Technical Guidance
- The "cough" mechanic needs to handle dropping items robustly. If multiple items are ejected onto the same tile, ensure the inventory/grid system can handle the overflow (e.g., spilling to adjacent tiles if necessary).
- Differentiate between "toxicity" (which causes coughing) and "damage" (which destroys the artery). Arteries should self-heal damage over time.

## 8. Questions
*Builder: add questions here if spec is unclear.*
