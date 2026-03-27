# 689 - The Empathy Tax

## 1. Overview
**Layer:** Cross-layer
**Fantasy:** The heavy psychological toll of leadership, where making "optimal" strategic decisions breaks the minds of your leaders.
**Mechanic:** Pops assigned to leadership or administrative roles develop a hidden "Empathy" stat based on their traits and past actions. When the player executes a ruthless Layer 2/3 command (like orbital bombardment, rationing, or abandoning a colony), Layer 1 leaders with high Empathy suffer massive, permanent Morale penalties and may develop psychotic traits or commit suicide.

## 2. Dependencies
- `Pop`, `Morale`, and `Job` components (Layer 1)
- `Edict` or `ColonyCommand` system (Layer 2/3 actions)
- `Trait` component system (Layer 1)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_ruthless_command_penalizes_empathetic_leaders() {
        // Arrange
        let mut app = App::new();
        app.add_event::<RuthlessCommandExecutedEvent>();
        app.add_systems(Update, apply_empathy_tax);

        let leader = app.world_mut().spawn((
            Pop,
            Job { is_leadership: true },
            Empathy { value: 80.0 }, // High empathy
            Morale { value: 100.0 },
        )).id();

        // Act
        app.world_mut().send_event(RuthlessCommandExecutedEvent { severity: 50.0 });
        app.update();

        // Assert
        let morale = app.world().get::<Morale>(leader).unwrap();
        // Morale drop should be proportional to empathy and severity
        assert!(morale.value < 100.0, "Empathetic leader should lose morale after a ruthless command");
    }

    #[test]
    fn test_ruthless_command_does_not_penalize_sociopathic_leaders() {
        // Arrange
        let mut app = App::new();
        app.add_event::<RuthlessCommandExecutedEvent>();
        app.add_systems(Update, apply_empathy_tax);

        let sociopath = app.world_mut().spawn((
            Pop,
            Job { is_leadership: true },
            Empathy { value: 10.0 }, // Low empathy
            Morale { value: 100.0 },
        )).id();

        // Act
        app.world_mut().send_event(RuthlessCommandExecutedEvent { severity: 50.0 });
        app.update();

        // Assert
        let morale = app.world().get::<Morale>(sociopath).unwrap();
        // Morale drop should be minimal or non-existent
        assert_eq!(morale.value, 100.0, "Sociopathic leader should not lose morale");
    }

    #[test]
    fn test_non_leaders_ignore_empathy_tax() {
        // Arrange
        let mut app = App::new();
        app.add_event::<RuthlessCommandExecutedEvent>();
        app.add_systems(Update, apply_empathy_tax);

        let worker = app.world_mut().spawn((
            Pop,
            Job { is_leadership: false },
            Empathy { value: 90.0 }, // High empathy, but not a leader
            Morale { value: 100.0 },
        )).id();

        // Act
        app.world_mut().send_event(RuthlessCommandExecutedEvent { severity: 50.0 });
        app.update();

        // Assert
        let morale = app.world().get::<Morale>(worker).unwrap();
        assert_eq!(morale.value, 100.0, "Non-leaders should not suffer the empathy tax from strategic commands");
    }

    #[test]
    fn test_extreme_empathy_tax_causes_breakdown() {
        // Arrange
        let mut app = App::new();
        app.add_event::<RuthlessCommandExecutedEvent>();
        app.add_systems(Update, apply_empathy_tax);

        let leader = app.world_mut().spawn((
            Pop,
            Job { is_leadership: true },
            Empathy { value: 100.0 }, // Max empathy
            Morale { value: 10.0 },   // Already low morale
        )).id();

        // Act
        // A massively ruthless command (e.g., orbital bombardment)
        app.world_mut().send_event(RuthlessCommandExecutedEvent { severity: 100.0 });
        app.update();

        // Assert
        let breakdown = app.world().get::<MentalBreakdown>(leader);
        assert!(breakdown.is_some(), "Leader with max empathy and low morale should suffer a mental breakdown");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Job {
    pub is_leadership: bool,
}

#[derive(Component)]
pub struct Empathy {
    pub value: f32, // 0.0 to 100.0
}

#[derive(Component)]
pub struct Morale {
    pub value: f32, // 0.0 to 100.0
}

#[derive(Component)]
pub struct MentalBreakdown;

#[derive(Event)]
pub struct RuthlessCommandExecutedEvent {
    pub severity: f32, // e.g., 10 for rationing, 100 for orbital strike
}

pub fn apply_empathy_tax(
    mut events: EventReader<RuthlessCommandExecutedEvent>,
    mut query: Query<(Entity, &Job, &Empathy, &mut Morale), Without<MentalBreakdown>>,
    mut commands: Commands,
) {
    for event in events.read() {
        for (entity, job, empathy, mut morale) in query.iter_mut() {
            if job.is_leadership {
                // Empathy tax is proportional to how empathetic they are and the severity of the act
                // E.g., severity 50, empathy 80 -> penalty is 50 * (80/100) = 40
                let empathy_factor = (empathy.value / 100.0).clamp(0.0, 1.0);

                // Only apply penalty if empathy is significant (e.g., > 20)
                if empathy_factor > 0.2 {
                    let penalty = event.severity * empathy_factor;
                    morale.value -= penalty;

                    if morale.value <= 0.0 {
                        morale.value = 0.0;
                        commands.entity(entity).insert(MentalBreakdown);
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Event Detail**: Instead of just a generic `RuthlessCommandExecutedEvent`, perhaps pass an enum for the type of command (`Rationing`, `Bombardment`, `Abandonment`) so the UI can log exactly *why* the leader snapped.
- **Mental Breakdown Effects**: `MentalBreakdown` should have actual consequences—perhaps they sabotage their workstation, step down from their job, or cause a massive unrest spike in their sector.
- **Dynamic Empathy**: Empathy shouldn't be static. Executing a ruthless command might permanently lower a leader's empathy (desensitizing them over time) alongside the morale hit.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Empathetic leaders (but not non-leaders or sociopaths) suffer morale damage when ruthless commands are executed.
- [ ] Extreme morale drops from empathy tax result in a `MentalBreakdown` component being applied.

## 7. Technical Guidance
- **Event Bus Usage**: Ensure `RuthlessCommandExecutedEvent` is fired accurately when the player clicks the UI to execute Layer 2/3 actions. It serves as the critical bridge for this cross-layer mechanic.
- **Morale Clamping**: Ensure `Morale` never drops below 0.0 or exceeds 100.0, even with massive severity events.
- **Trait Synergies**: Look for existing traits like `Ruthless` or `Compassionate` and use them to initialize or modify the `Empathy` value when a pop is spawned.

## 8. Questions
*Builder: add questions here if spec is unclear.*
