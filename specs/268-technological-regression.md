# 268: Technological Regression

## 1. Overview
The **Technological Regression** system simulates the loss of knowledge over time if skills are not practiced. In a generational colony setting, if no colonist works a specific advanced job type (e.g., Medicine, Advanced Engineering) for a long period, the colony's "Tech Level" for that category degrades. Advanced buildings may become inoperable "Black Boxes" until the knowledge is rediscovered or relearned.

## 2. Dependencies
- `011` Tech Tree Backend (for tech nodes)
- `009` Job System (to track active jobs)
- `051` Pop Skills (to track who has what knowledge)

## 3. RED Phase: Tests First

```rust
// tests/regression_tests.rs

use bevy::prelude::*;
use scale::layer1::tech::{TechTree, TechNode, TechStatus};
use scale::layer1::skills::SkillType;
use scale::layer1::regression::RegressionSystem;

#[test]
fn test_tech_regresses_when_skill_unused() {
    let mut app = App::new();
    app.add_systems(Update, RegressionSystem);

    // Setup TechTree with an advanced tech
    let mut tech_tree = TechTree::default();
    tech_tree.nodes.insert("AdvancedMedicine".to_string(), TechNode {
        status: TechStatus::Researched,
        associated_skill: Some(SkillType::Medicine),
        regression_threshold: 1000,
        time_since_last_use: 1001, // Over threshold
    });
    app.world.insert_resource(tech_tree);

    // Act
    app.update();

    // Assert
    let tree = app.world.get_resource::<TechTree>().unwrap();
    let node = tree.nodes.get("AdvancedMedicine").unwrap();
    assert_eq!(node.status, TechStatus::Forgotten, "Tech should regress to Forgotten if unused for too long.");
}

#[test]
fn test_tech_does_not_regress_if_skill_used() {
    let mut app = App::new();
    app.add_systems(Update, RegressionSystem);

    let mut tech_tree = TechTree::default();
    tech_tree.nodes.insert("AdvancedEngineering".to_string(), TechNode {
        status: TechStatus::Researched,
        associated_skill: Some(SkillType::Engineering),
        regression_threshold: 1000,
        time_since_last_use: 500, // Under threshold
    });
    app.world.insert_resource(tech_tree);

    // Act
    app.update();

    // Assert
    let tree = app.world.get_resource::<TechTree>().unwrap();
    let node = tree.nodes.get("AdvancedEngineering").unwrap();
    assert_eq!(node.status, TechStatus::Researched, "Tech should remain Researched if actively used.");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/regression.rs

use bevy::prelude::*;
use crate::layer1::tech::{TechTree, TechStatus};
use crate::layer1::skills::SkillType;
use crate::layer1::jobs::JobExecutionEvent;

pub fn update_tech_usage_system(
    mut events: EventReader<JobExecutionEvent>,
    mut tech_tree: ResMut<TechTree>,
) {
    for event in events.read() {
        // Reset timer for tech associated with this skill
        for (_, node) in tech_tree.nodes.iter_mut() {
            if Some(event.skill_used) == node.associated_skill {
                node.time_since_last_use = 0;
            }
        }
    }
}

pub fn regression_system(
    mut tech_tree: ResMut<TechTree>,
) {
    for (name, node) in tech_tree.nodes.iter_mut() {
        if node.status == TechStatus::Researched {
            node.time_since_last_use += 1;

            if node.time_since_last_use > node.regression_threshold {
                node.status = TechStatus::Forgotten;
                info!("Tech forgotten due to disuse: {}", name);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Gradual Degradation:** Instead of a binary `Researched` to `Forgotten`, introduce a `Degraded` state where buildings still work but at 50% efficiency or with high accident risks.
- **Visual Feedback:** When a tech is forgotten, related advanced buildings should show a "Lost Knowledge" warning icon and halt operation.

## 6. Acceptance Criteria (Testable!)
- [ ] Tech nodes track time since their associated skills were last used.
- [ ] Tech nodes revert to a `Forgotten` or `Locked` state if the threshold is passed.
- [ ] Active job execution resets the regression timer.
- [ ] All tests in RED phase pass.
- [ ] Test coverage ≥85% for `regression.rs`.

## 7. Technical Guidance
- **TechNode Extension:** Add `associated_skill: Option<SkillType>`, `time_since_last_use: u64`, and `regression_threshold: u64` to `TechNode` in `src/layer1/tech/mod.rs`.
- **Event Driven:** Hook into `handle_mining_work`, `handle_crafting_work`, etc. to emit `JobExecutionEvent`.
- **Re-researching:** Ensure the tech tree UI allows re-researching a `Forgotten` tech (maybe at a reduced cost).

## 8. Questions
*Builder: add questions here if spec is unclear.*
