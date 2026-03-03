# Spec 276: Diplomatic Immunity

## 1. Overview
High-ranking dignitaries from powerful Layer 3 empires visit your Layer 1 colony. They possess "Diplomatic Immunity." If they commit a crime (e.g., assault, theft, vandalism due to low mood), your local Justice System cannot arrest them without triggering a massive diplomatic incident or war.

## 2. Dependencies
- `234` The Visitor / `047` Pop Relationships (for interaction context)
- `072` Justice System
- `031` Pop Morale

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diplomat_commits_crime_no_auto_arrest() {
        // Arrange: A Pop with DiplomaticImmunity commits a crime
        // Act: Run justice/police system tick
        // Assert: The Pop is NOT automatically arrested by normal guards
    }

    #[test]
    fn test_arresting_diplomat_causes_diplomatic_incident() {
        // Arrange: A Pop with DiplomaticImmunity
        // Act: Player issues a manual override to arrest the diplomat
        // Assert: Diplomat is arrested, but a DiplomaticIncident event is fired
    }

    #[test]
    fn test_unpunished_diplomat_crime_increases_unrest() {
        // Arrange: A diplomat commits a crime and is not arrested
        // Act: Run morale update tick
        // Assert: Local colony unrest/stress increases due to perceived injustice
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct DiplomaticImmunity {
    pub faction_id: Entity,
}

// Modify JusticeSystem to check `!query.contains::<DiplomaticImmunity>(entity)` before auto-arresting.
// Add logic to increase global/local unrest if a crime goes unpunished by an immune pop.
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: Ensure the existing `JusticeSystem` gracefully handles immune entities. Unrest should specifically tag the diplomat as the cause if possible.
- **Player Options**: Provide a way for the player to "Bribe" local pops or "Settle" the issue quietly to reduce unrest without arresting the diplomat.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.
- [ ] Immune entities are ignored by automated law enforcement.
- [ ] Unpunished crimes by immune entities generate unrest among the general population.
- [ ] Manually arresting an immune entity triggers a severe negative consequence (event).

## 7. Technical Guidance
- Add a `DiplomaticImmunity` component.
- The `JusticeSystem` needs an early return or skip condition for immune pops.
- Generate a `Grievance` or apply a negative `MoodModifier` to witnesses of the unpunished crime.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
