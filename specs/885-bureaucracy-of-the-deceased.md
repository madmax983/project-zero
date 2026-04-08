# 885: Bureaucracy of the Deceased

## 1. Overview
The dead still have paperwork to file. When a Pop dies, their administrative load doesn't instantly vanish. "Death Certificates" and "Estate Transfers" must be processed by the Bureaucrat Pops. Until processed, the dead Pop's former housing remains locked, and their next of kin suffer a "Limbo" stress debuff. Expanding an otherwise useless bureaucratic sector to handle sudden mortality spikes becomes essential post-disaster to deal with severe housing and morale crises.

## 2. Dependencies
- `057-funeral-rites.md` (Death handling)
- `175-bureaucratic-drag.md` (Admin mechanics)
- `064-room-quality.md` (Housing occupancy)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_death_spawns_paperwork() {
        // Arrange
        let mut app = App::new();
        // Add death event reader system
        let pop = app.world_mut().spawn((
            Pop,
            Name("Deceased".into()),
        )).id();

        // Act
        app.world_mut().send_event(PopDiedEvent { entity: pop });
        app.update();

        // Assert
        let paperwork_query = app.world_mut().query::<&DeathPaperwork>();
        assert_eq!(paperwork_query.iter(app.world()).count(), 1, "Death should spawn a pending DeathPaperwork entity/component");
    }

    #[test]
    fn test_housing_locked_until_paperwork_processed() {
        // Arrange
        let mut app = App::new();
        // Setup pop assigned to a house
        let house = app.world_mut().spawn((
            Housing { capacity: 1, occupants: vec![] }, // Need to implement actual struct fields here
        )).id();
        let deceased = app.world_mut().spawn((
            Pop,
            AssignedHousing(house),
        )).id();

        // Act
        app.world_mut().send_event(PopDiedEvent { entity: deceased });
        app.update();

        // Assert
        let house_state = app.world().get::<Housing>(house).unwrap();
        assert!(house_state.is_locked_for_estate_transfer(), "Housing must be locked until estate is processed");

        // Act: Process the paperwork
        let paperwork = app.world_mut().query_filtered::<Entity, With<DeathPaperwork>>().iter(app.world()).next().unwrap();
        app.world_mut().send_event(ProcessPaperworkEvent { entity: paperwork });
        app.update();

        // Assert
        let house_state_after = app.world().get::<Housing>(house).unwrap();
        assert!(!house_state_after.is_locked_for_estate_transfer(), "Housing should be available after processing");
    }

    #[test]
    fn test_next_of_kin_limbo_stress() {
        // Arrange
        let mut app = App::new();
        let deceased = app.world_mut().spawn(Pop).id();
        let kin = app.world_mut().spawn((
            Pop,
            Relationship { target: deceased, affinity: 80.0 }, // Close kin
            Stress(0.0),
        )).id();

        // Act
        app.world_mut().send_event(PopDiedEvent { entity: deceased });
        app.update(); // Generates paperwork

        // Apply limbo stress tick
        app.update(); // Advance time for stress to accumulate

        // Assert
        let kin_stress = app.world().get::<Stress>(kin).unwrap().0;
        assert!(kin_stress > 0.0, "Kin should gain 'Limbo' stress while paperwork is pending");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Event)]
pub struct PopDiedEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct ProcessPaperworkEvent {
    pub entity: Entity,
}

#[derive(Component)]
pub struct DeathPaperwork {
    pub deceased_pop: Entity,
    pub previous_housing: Option<Entity>,
}

#[derive(Component)]
pub struct EstateLocked; // Tag for housing

pub fn handle_pop_death_system(
    mut commands: Commands,
    mut events: EventReader<PopDiedEvent>,
    pop_housing_query: Query<&AssignedHousing>,
) {
    for event in events.read() {
        let housing = pop_housing_query.get(event.entity).ok().map(|h| h.0);

        commands.spawn(DeathPaperwork {
            deceased_pop: event.entity,
            previous_housing: housing,
        });

        if let Some(house) = housing {
            commands.entity(house).insert(EstateLocked);
        }
    }
}

pub fn apply_limbo_stress_system(
    paperwork_query: Query<&DeathPaperwork>,
    mut kin_query: Query<(&Relationship, &mut Stress)>,
) {
    // For every pending paperwork, apply stress to related kin
}

pub fn process_estate_paperwork_system(
    mut commands: Commands,
    mut events: EventReader<ProcessPaperworkEvent>,
    paperwork_query: Query<&DeathPaperwork>,
) {
    for event in events.read() {
        if let Ok(paperwork) = paperwork_query.get(event.entity) {
            if let Some(house) = paperwork.previous_housing {
                commands.entity(house).remove::<EstateLocked>();
            }
            commands.entity(event.entity).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate `DeathPaperwork` directly into the existing `bureaucratic-drag.md` admin queues.
- Ensure the "Limbo" debuff replaces or interacts properly with the standard "Grief" debuff. It could be an additional modifier that clears when the paperwork finishes.
- Check how `AssignedHousing` logic works to ensure `EstateLocked` safely prevents new move-ins without breaking the AI.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Death generates paperwork tasks for Bureaucrats.
- [ ] Housing remains locked until paperwork finishes.
- [ ] Kin suffer stress while paperwork is pending.

## 7. Technical Guidance
- The Bureaucrat job should have a UtilityAI task specifically targeting `DeathPaperwork` entities.
- Depending on the implementation of Administrative points, this might consume X Admin points rather than an explicit "Process" event.

## 8. Questions
*Builder: add questions here if spec is unclear.*
