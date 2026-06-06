# 1295: Corporate Rebranding

## Overview

The whims of the boardroom affect the frontier. The parent corporation (Layer 3) occasionally mandates a "Corporate Rebranding" event, changing its official logo or color scheme. The colony (Layer 1) is given a deadline (e.g., 30 days) to repaint all designated "Official" buildings to the new standard. Failing to comply results in significant funding cuts or prestige penalties, while succeeding drains massive amounts of raw resources and labor for zero functional benefit.

## Dependencies

- `012` — Faction System (Base Layer 3 parent faction)
- `045` — Structure Maintenance / Actions
- `090` — Trade & Funding (For funding penalties)

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::structure::{Structure, StructureType};
    use crate::layer3::faction::Faction;

    #[test]
    fn test_rebranding_mandate_event_triggers() {
        let mut app = App::new();
        // Setup parent faction and colony link
        app.world_mut().spawn(Faction { id: 1, name: "Synergy Corp".to_string() });

        // Trigger the rebranding mandate
        app.world_mut().send_event(CorporateRebrandingEvent {
            faction_id: 1,
            new_color: "Synergy Blue".to_string(),
            deadline_ticks: 30_000,
        });

        app.update();

        // Ensure the mandate resource is created/active
        let mandate = app.world().resource::<RebrandingMandate>();
        assert!(mandate.is_active);
        assert_eq!(mandate.target_color, "Synergy Blue");
    }

    #[test]
    fn test_structures_require_repainting() {
        let mut app = App::new();

        // Spawn an official structure
        let hq_entity = app.world_mut().spawn((
            Structure { structure_type: StructureType::Headquarters, ..Default::default() },
            ColorTheme { current_color: "Standard Gray".to_string() },
        )).id();

        app.insert_resource(RebrandingMandate {
            is_active: true,
            target_color: "Synergy Blue".to_string(),
            deadline_ticks: 30_000,
        });

        app.update();

        // The structure should now have a 'NeedsRepainting' marker component
        assert!(app.world().get::<NeedsRepainting>(hq_entity).is_some());
    }

    #[test]
    fn test_funding_cut_on_missed_deadline() {
        let mut app = App::new();

        app.insert_resource(ColonyFunding { current_budget: 1000.0 });
        app.insert_resource(RebrandingMandate {
            is_active: true,
            target_color: "Synergy Blue".to_string(),
            deadline_ticks: 1, // Will expire next tick
        });

        // Spawn a structure that hasn't been repainted
        app.world_mut().spawn((
            Structure { structure_type: StructureType::Headquarters, ..Default::default() },
            ColorTheme { current_color: "Standard Gray".to_string() },
            NeedsRepainting,
        ));

        // Advance past deadline
        app.update();
        app.update();

        let funding = app.world().resource::<ColonyFunding>();
        let mandate = app.world().resource::<RebrandingMandate>();

        assert_eq!(mandate.is_active, false); // Event over
        assert!(funding.current_budget < 1000.0); // Funding was penalized
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Event)]
pub struct CorporateRebrandingEvent {
    pub faction_id: u32,
    pub new_color: String,
    pub deadline_ticks: i32,
}

#[derive(Resource, Default)]
pub struct RebrandingMandate {
    pub is_active: bool,
    pub target_color: String,
    pub deadline_ticks: i32,
}

#[derive(Component)]
pub struct ColorTheme {
    pub current_color: String,
}

#[derive(Component)]
pub struct NeedsRepainting;

// Placeholder for existing funding resource
#[derive(Resource, Default)]
pub struct ColonyFunding {
    pub current_budget: f32,
}

// Placeholder for existing structure component
#[derive(Component)]
pub struct Structure {
    pub structure_type: StructureType,
}

pub enum StructureType {
    Headquarters,
    Housing,
}

pub fn handle_rebranding_event_system(
    mut events: EventReader<CorporateRebrandingEvent>,
    mut mandate: ResMut<RebrandingMandate>,
) {
    for event in events.read() {
        mandate.is_active = true;
        mandate.target_color = event.new_color.clone();
        mandate.deadline_ticks = event.deadline_ticks;
    }
}

pub fn flag_structures_for_repainting_system(
    mut commands: Commands,
    mandate: Res<RebrandingMandate>,
    structures: Query<(Entity, &Structure, &ColorTheme), Without<NeedsRepainting>>,
) {
    if mandate.is_active {
        for (entity, structure, theme) in structures.iter() {
            // Only official structures care about corporate branding
            if matches!(structure.structure_type, StructureType::Headquarters) {
                if theme.current_color != mandate.target_color {
                    commands.entity(entity).insert(NeedsRepainting);
                }
            }
        }
    }
}

pub fn process_rebranding_deadline_system(
    mut mandate: ResMut<RebrandingMandate>,
    mut funding: ResMut<ColonyFunding>,
    unpainted_structures: Query<Entity, With<NeedsRepainting>>,
) {
    if mandate.is_active {
        mandate.deadline_ticks -= 1;

        if mandate.deadline_ticks <= 0 {
            mandate.is_active = false;

            // Check if there are still structures that need repainting
            if !unpainted_structures.is_empty() {
                // Apply penalty
                funding.current_budget -= 500.0;
                if funding.current_budget < 0.0 {
                    funding.current_budget = 0.0;
                }
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Code Smell:** Hardcoded penalty of `500.0` in the deadline system. The penalty should be configurable or scale based on the size of the colony and the parent faction's relationship.
- **Integration:** Hook `NeedsRepainting` into the job system so Pops can actually perform the "Repaint" task, consuming resources (e.g., `Dye` or `Paint`).
- **Optimization:** `flag_structures_for_repainting_system` runs every frame while the mandate is active. It should only run when the event triggers or when a new official structure is built.
- **Narrative:** Add Chronicle events for when the mandate is issued ("Corporate has decided that Red is too aggressive...") and when it resolves (compliance or failure).

## Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Structures gain `NeedsRepainting` when mandate is active.
- [ ] Missed deadlines result in a funding penalty.

## Technical Guidance

- Place this logic near the Layer 3 <-> Layer 1 interaction modules (e.g., `src/layer1/core/integration.rs` or a dedicated `events` module).
- The `CorporateRebrandingEvent` should likely be emitted by a Layer 3 simulation system processing corporate whims.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
