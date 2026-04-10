# 929 - Memorial Scrap

## 1. Overview

**Layer:** 2
**Fantasy:** Sacred reverence for destroyed ships preventing necessary recycling.
**Mechanic:** When a veteran fleet is destroyed, its wreckage is marked as "Memorial Scrap". Pops assigned to salvage it suffer massive morale penalties, and may even refuse the work order entirely.
**Emergence:** A vital hyperlane choke point becomes cluttered with the husks of past heroic last stands. You desperately need the alloys to build a defense fleet, but attempting to harvest them triggers an immediate mutiny in your salvage crews.
**Tension:** Do you leave the scrap as an orbital monument blocking traffic, or desecrate the memorial to build your next fleet?

## 2. Dependencies

- Fleet Combat / Destruction systems
- Salvage mechanics
- Pop Morale system
- Pop Assignment / Work order systems

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct Wreckage { is_memorial: bool }

    #[derive(Component)]
    struct PopAssignment { target: Entity }

    #[derive(Component)]
    struct PopMorale { value: f32 }

    #[test]
    fn test_salvaging_memorial_scrap_reduces_morale() {
        let mut app = App::new();
        app.add_systems(Update, process_memorial_salvage_system);

        let scrap = app.world_mut().spawn(Wreckage { is_memorial: true }).id();

        let pop = app.world_mut().spawn((
            PopAssignment { target: scrap },
            PopMorale { value: 1.0 },
        )).id();

        // Run system to process assignment
        app.update();

        // Morale should be significantly penalized
        let updated_morale = app.world().get::<PopMorale>(pop).unwrap();
        assert!(updated_morale.value < 0.5);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// pub fn process_memorial_salvage_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design

- Introduce a specific `MoodModifier` (e.g. "Desecrated Memorial") rather than modifying the base morale float directly, allowing it to decay over time.
- Implement logic where high-veterancy fleets dynamically get the `is_memorial` flag upon destruction.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops assigned to memorial wreckage suffer morale penalties.

## 7. Technical Guidance

- Integrate with the existing salvage work execution.
- If the morale drop pushes a Pop past a threshold, consider having the system unassign them automatically or trigger a minor mutiny event.

## 8. Questions
*Builder: add questions here if spec is unclear.*
