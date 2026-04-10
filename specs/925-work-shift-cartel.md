# 925 - The Work-Shift Cartel

## 1. Overview

**Layer:** 1
**Fantasy:** Workers organizing to control labor supply and demand informally.
**Mechanic:** Pops with high social influence in a specific sector form a "Shift Cartel," demanding higher leisure time and sabotaging productivity of non-members.

## 2. Dependencies

- Social interaction systems (`SocialInfluence` / `UtilityWeights`)
- Work Sector designations
- Productivity calculation

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct ShiftCartelMember { sector_id: u32 }

    #[derive(Component)]
    struct Worker { sector_id: u32, base_productivity: f32 }

    #[derive(Component)]
    struct ProductivityModifier { value: f32 }

    #[test]
    fn test_cartel_sabotages_non_members() {
        let mut app = App::new();
        app.add_systems(Update, apply_cartel_sabotage_system);

        // Cartel member
        app.world_mut().spawn((
            ShiftCartelMember { sector_id: 1 },
        ));

        // Non-cartel worker in same sector
        let victim = app.world_mut().spawn((
            Worker { sector_id: 1, base_productivity: 1.0 },
            ProductivityModifier { value: 1.0 },
        )).id();

        app.update();

        let modifier = app.world().get::<ProductivityModifier>(victim).unwrap();
        assert!(modifier.value < 1.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// pub fn apply_cartel_sabotage_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design

- Extract cartel grouping logic so sectors can be evaluated in bulk instead of per-pop.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] Cartel members reduce productivity of non-members in the same sector.

## 7. Technical Guidance

- Use resource/entity indices for `sector_id` to link pops to their workplaces.

## 8. Questions
*Builder: add questions here if spec is unclear.*
