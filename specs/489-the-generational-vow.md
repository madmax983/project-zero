# Specification: The Generational Vow

## 1. Overview
"The Generational Vow" is a Layer 1 mechanic where a highly respected or deeply wronged Pop issues a permanent directive upon their death. This "vow" is passed down to all their descendants, applying a permanent, unremovable directive (e.g., "Always work in Agriculture" or "Never work in Industry"). Descendants receive massive morale boosts when fulfilling the vow and severe penalties when forced to ignore or break it, forcing the player to balance colony needs against deeply held familial beliefs.

## 2. Dependencies
- **Pop Lineage/Family Tree System:** To track descendants of a specific Pop.
- **Death/Chronicle Events:** To trigger the creation of a vow upon a notable Pop's death.
- **Job/Task System:** To allow vows to interact with assigned roles or locations.
- **Needs/Morale System:** To apply buffs/debuffs based on vow adherence.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Needs, JobId};
    use crate::layer1::life_cycle::PopDeathEvent;

    #[test]
    fn test_notable_pop_death_creates_vow() {
        // Arrange
        let mut app = App::new();
        app.add_event::<PopDeathEvent>();
        app.add_systems(Update, handle_death_vow_creation_system);

        // Act
        app.world_mut().send_event(PopDeathEvent {
            entity: Entity::from_raw(1),
            is_notable: true,
            lineage_id: 100,
        });
        app.update();

        // Assert
        let vows = app.world().query::<&GenerationalVow>().iter(app.world()).count();
        assert_eq!(vows, 1, "A vow should be created for the lineage upon a notable death");
    }

    #[test]
    fn test_descendants_inherit_vow() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_vows_to_descendants_system);

        let vow_entity = app.world_mut().spawn(GenerationalVow {
            lineage_id: 100,
            directive: VowDirective::MustWorkAgriculture,
        }).id();

        let descendant_id = app.world_mut().spawn((Pop, Lineage(100))).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().entity(descendant_id).has::<VowBearer>());
    }

    #[test]
    fn test_vow_compliance_affects_needs() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, evaluate_vow_compliance_system);

        let compliant_pop = app.world_mut().spawn((
            Pop,
            Lineage(100),
            VowBearer(VowDirective::MustWorkAgriculture),
            JobId(1), // Assume JobId(1) is an Agriculture job
            Needs { morale: 50.0, ..default() }
        )).id();

        let defiant_pop = app.world_mut().spawn((
            Pop,
            Lineage(100),
            VowBearer(VowDirective::MustWorkAgriculture),
            JobId(2), // Assume JobId(2) is an Industry job
            Needs { morale: 50.0, ..default() }
        )).id();

        // Act
        app.update();

        // Assert
        let compliant_morale = app.world().get::<Needs>(compliant_pop).unwrap().morale;
        let defiant_morale = app.world().get::<Needs>(defiant_pop).unwrap().morale;

        assert!(compliant_morale > 50.0, "Compliant pop should gain morale");
        assert!(defiant_morale < 50.0, "Defiant pop should lose morale");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pop::{Pop, Needs, JobId};
use crate::layer1::life_cycle::PopDeathEvent;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VowDirective {
    MustWorkAgriculture,
    NeverWorkIndustry,
}

#[derive(Component)]
pub struct GenerationalVow {
    pub lineage_id: u32,
    pub directive: VowDirective,
}

#[derive(Component)]
pub struct Lineage(pub u32);

#[derive(Component)]
pub struct VowBearer(pub VowDirective);

pub fn handle_death_vow_creation_system(
    mut commands: Commands,
    mut events: EventReader<PopDeathEvent>,
) {
    for event in events.read() {
        if event.is_notable {
            commands.spawn(GenerationalVow {
                lineage_id: event.lineage_id,
                directive: VowDirective::MustWorkAgriculture, // Hardcoded for minimal implementation
            });
        }
    }
}

pub fn apply_vows_to_descendants_system(
    mut commands: Commands,
    vow_query: Query<&GenerationalVow>,
    descendant_query: Query<(Entity, &Lineage), (With<Pop>, Without<VowBearer>)>,
) {
    for vow in vow_query.iter() {
        for (entity, lineage) in descendant_query.iter() {
            if lineage.0 == vow.lineage_id {
                commands.entity(entity).insert(VowBearer(vow.directive));
            }
        }
    }
}

pub fn evaluate_vow_compliance_system(
    mut query: Query<(&VowBearer, &JobId, &mut Needs), With<Pop>>,
) {
    for (bearer, job, mut needs) in query.iter_mut() {
        match bearer.0 {
            VowDirective::MustWorkAgriculture => {
                if job.0 == 1 { // Assuming 1 == Agriculture
                    needs.morale += 5.0; // Buff
                } else {
                    needs.morale -= 5.0; // Debuff
                }
            }
            _ => {}
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Dynamic Vow Generation:** Instead of hardcoding `MustWorkAgriculture`, vows should be dynamically generated based on the dying Pop's traits, life events, or the circumstances of their death (e.g., if they starved, they vow to never stop farming).
- **Global Vow Registry:** Consider moving active vows to a global Resource (like a `Chronicle`) rather than floating entities, making it easier to lookup and display in the UI.
- **Job Integration:** Replace the hardcoded `JobId(1)` check with proper interrogation of the building/job type the Pop is currently assigned to.
- **Decay/Absolution:** While vows are permanent, consider adding a mechanic where an extreme colony event (like a successful holy festival) can clear a negative vow.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Vows are created upon the death of notable Pops.
- [ ] Vows correctly attach to descendants matching the lineage ID.
- [ ] Fulfilling or breaking the vow correctly applies morale modifiers.

## 7. Technical Guidance
- **System Placement:** `handle_death_vow_creation_system` and `apply_vows_to_descendants_system` should likely run in `Layer1SystemSet::Update` or an event-handling set, while `evaluate_vow_compliance_system` should run in `Layer1SystemSet::Observation`.
- **Lineage Implementation:** This feature heavily relies on the pre-existence of a robust `Lineage` or family tree tracking mechanism. Ensure this is solid before integrating the vow logic.
- **Performance:** `apply_vows_to_descendants_system` could become expensive if there are many vows and pops. If moved to a global Resource, descendants could simply look up their lineage ID in the Resource map on spawn.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
