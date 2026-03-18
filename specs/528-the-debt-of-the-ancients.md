# Overview

**The Debt of the Ancients**
**Layer:** Cross-layer
**Fantasy:** You inherit the sins of a precursor race. The galaxy holds you accountable for things you didn't do.
**Mechanic:** By settling a specific world or using precursor tech, you trigger an ancient, dormant diplomatic treaty or debt. Layer 3 empires suddenly demand tribute, or conversely, offer subservience based on an agreement signed millennia ago by people who are not you.
**Emergence:** You unearth a powerful shield generator. The moment you turn it on, a neighboring empire declares a holy war, claiming you have activated the "Engine of their Ancestors' Doom," dragging you into a conflict over a history you don't even understand.
**Tension:** Utilizing powerful found advantages vs. the unpredictable, massive diplomatic consequences of stepping into someone else's ancient shoes.

# Dependencies

- `246-legacy-code.md` (or general Precursor/Ruin systems)
- `156-xeno-artifacts.md`
- `010-chronicle-system.md`

# RED Phase: Tests First

```rust
// tests/integration/debt_of_ancients_tests.rs

#[test]
fn test_activating_precursor_tech_triggers_diplomatic_debt() {
    let mut app = setup_world();

    // Arrange: Precursor artifact and a Layer 3 faction
    let artifact = app.world_mut().spawn((
        PrecursorArtifact { active: false },
        AncientDebtTrigger { faction_id: FactionId(1), penalty: -50 },
    )).id();

    app.world_mut().resource_mut::<DiplomacyMap>().set_relation(FactionId::Player, FactionId(1), 0);

    // Act: Activate the artifact
    app.world_mut().send_event(CommandEvent::ActivateArtifact(artifact));
    app.update();

    // Assert: Diplomatic relation dropped
    let relation = app.world().resource::<DiplomacyMap>().get_relation(FactionId::Player, FactionId(1));
    assert_eq!(relation, -50);
}

#[test]
fn test_ancient_debt_chronicle_event_created() {
    let mut app = setup_world();

    let artifact = app.world_mut().spawn((
        PrecursorArtifact { active: false },
        AncientDebtTrigger { faction_id: FactionId(2), penalty: 100 }, // Positive debt (subservience)
    )).id();

    // Act
    app.world_mut().send_event(CommandEvent::ActivateArtifact(artifact));
    app.update();

    // Assert: Chronicle recorded the event
    let chronicle = app.world().resource::<Chronicle>();
    let events = chronicle.get_recent();
    assert!(events.iter().any(|e| e.template_id == "ANCIENT_DEBT_ACTIVATED"));
}
```

# GREEN Phase: Minimal Implementation

```rust
// src/layer1/integration/debt_of_ancients.rs

use bevy::prelude::*;
use crate::shared::events::CommandEvent;
use crate::layer3::diplomacy::{DiplomacyMap, FactionId};
use crate::layer1::chronicle::{Chronicle, AddChronicleEvent, EventImportance};

#[derive(Component)]
pub struct PrecursorArtifact {
    pub active: bool,
}

#[derive(Component)]
pub struct AncientDebtTrigger {
    pub faction_id: FactionId,
    pub penalty: i32, // Positive or negative
}

pub fn process_ancient_debt_system(
    mut commands: Commands,
    mut events: EventReader<CommandEvent>,
    mut query: Query<(&mut PrecursorArtifact, &AncientDebtTrigger)>,
    mut diplomacy: ResMut<DiplomacyMap>,
    mut chronicle_writer: EventWriter<AddChronicleEvent>,
) {
    for event in events.read() {
        if let CommandEvent::ActivateArtifact(entity) = event {
            if let Ok((mut artifact, trigger)) = query.get_mut(*entity) {
                if !artifact.active {
                    artifact.active = true;

                    // Apply diplomatic shift
                    let current = diplomacy.get_relation(FactionId::Player, trigger.faction_id);
                    diplomacy.set_relation(FactionId::Player, trigger.faction_id, current + trigger.penalty);

                    // Add to chronicle
                    chronicle_writer.send(AddChronicleEvent {
                        template_id: "ANCIENT_DEBT_ACTIVATED".to_string(),
                        importance: EventImportance::Major,
                        slots: vec![("FACTION".to_string(), trigger.faction_id.0.to_string())],
                    });
                }
            }
        }
    }
}
```

# REFACTOR Phase: Quality & Design

- **Chronicle**: Update `lore/TEMPLATES.md` with `ANCIENT_DEBT_ACTIVATED` templates.
- **Emergence**: It shouldn't just be instant numbers changing. The faction should actively hail the player and offer an ultimatum or tribute depending on the relation shift.
- **Architectural Cleanup**: Place this logic in an integration bridge (e.g., `src/layer1/integration.rs`) to keep Layer 1 artifact logic cleanly separated from Layer 3 diplomacy logic.

# Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code.
- [ ] Activating specific ancient artifacts shifts Layer 3 diplomacy.
- [ ] The event generates a `Major` chronicle entry.

# Technical Guidance

- `AncientDebtTrigger` should be added to select artifacts during world generation based on the history generated for the sector.
- Ensure the `DiplomacyMap` has bounded min/max values (e.g., -100 to 100) so massive penalties don't underflow.

# Questions

*Builder: add questions here if spec is unclear.*
