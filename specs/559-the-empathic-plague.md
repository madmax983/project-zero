# 559 - The Empathic Plague

## 1. Overview
A disease that doesn't kill your body, but forces you to feel everyone else's pain.

**Fantasy:** A minor food shortage in a distant mining outpost causes the miners' morale to plummet. The empathic plague spreads this misery to the nearby industrial sector, causing factory workers to strike out of overwhelming shared despair.

**Layer:** Cross-layer (1 -> 2)

## 2. Dependencies
- `005-pop-needs.md` (Morale/Needs)
- `127-stress-breakdowns.md` (Stress)
- `098-medical-triage.md` (Diseases/Infections)
- `047-pop-relationships.md` (Empathy/Links)
- `266-emotional-contagion.md` (Mood Spreading)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // 1. Empathic Link Formation
    #[test]
    fn test_plague_infection_links_morale_states() {
        // Arrange: Pop A (Infected), Pop B (Uninfected, low morale) in radius
        // Act: Run empathic update system
        // Assert: Pop A's morale drops due to Pop B's misery
    }

    // 2. Cascading Failure
    #[test]
    fn test_empathic_cascade_causes_unrest() {
        // Arrange: Multiple infected Pops near one starving Pop
        // Act: Run morale check system
        // Assert: All infected Pops experience stress breakdown due to shared starvation penalty
    }

    // 3. Treatment and Isolation
    #[test]
    fn test_medical_treatment_cures_empathic_plague() {
        // Arrange: Infected Pop in medical bed
        // Act: Run medical tick
        // Assert: Infection component removed, morale link broken
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct EmpathicPlagueInfection {
    pub radius: f32, // How far they feel pain
}

// System to apply empathic penalties
pub fn apply_empathic_pain(
    mut infected_query: Query<(&mut Morale, &GlobalTransform, &EmpathicPlagueInfection)>,
    others_query: Query<(&Morale, &GlobalTransform), Without<EmpathicPlagueInfection>>,
) {
    for (mut infected_morale, infected_transform, infection) in infected_query.iter_mut() {
        let mut total_absorbed_pain = 0.0;

        for (other_morale, other_transform) in others_query.iter() {
            let distance = infected_transform.translation().distance(other_transform.translation());

            if distance <= infection.radius {
                // If they are miserable, we feel it
                if other_morale.value < other_morale.max / 2.0 {
                    let pain_felt = (other_morale.max / 2.0) - other_morale.value;
                    total_absorbed_pain += pain_felt;
                }
            }
        }
        // Cap the pain so it doesn't instantly kill them
        infected_morale.value = f32::max(0.0, infected_morale.value - (total_absorbed_pain * 0.1));
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Visuals:** Add a visual link (faint tether or aura) between the infected and the source of their misery.
- **UI:** The Morale breakdown tooltip should clearly state: "Feeling the pain of others: -X."
- **Lore Integration:** Trigger a chronicle event when the first Pop breaks down due to shared empathy.

## 6. Acceptance Criteria
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `empathic_plague.rs`.
- [ ] Infected Pops suffer morale penalties based on the low morale of nearby Pops.
- [ ] Multiple unhappy Pops can cause an infected Pop to suffer a stress breakdown.
- [ ] The infection can be cured via existing medical triage systems.

## 7. Technical Guidance
- Be careful with O(N^2) complexity in the distance checks. Use a spatial partition or spatial hashing if the Pop count gets high.
- Ensure the plague can spread to other Pops like a standard disease.

## 8. Questions
*Builder: add questions here if spec is unclear.*
