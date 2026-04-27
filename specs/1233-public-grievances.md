# Public Grievances

## 1. Overview
**Layer:** 1
**Fantasy:** Airing dirty laundry. A village square where reputation is currency.
**Mechanic:** Buildable "Bulletin Board". Pops post "Grievances" (insults/complaints) or "Praise". Visible to player. Targets gain/lose Social Standing.

## 2. Dependencies
- Layer 1 Building Placement System
- Layer 1 Social System / Need System (Pops interacting)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_post_grievance_decreases_standing() {
        // Arrange
        let mut app = App::new();
        app.add_event::<PostGrievanceEvent>();

        let target_pop = app.world_mut().spawn(SocialStanding { value: 50.0 }).id();
        let poster_pop = app.world_mut().spawn(ColonyPop).id();
        let bulletin_board = app.world_mut().spawn(BulletinBoard).id();

        // Act
        app.world_mut().send_event(PostGrievanceEvent {
            poster: poster_pop,
            target: target_pop,
            board: bulletin_board,
            impact: -10.0,
        });
        app.update();

        // Assert
        let target_standing = app.world().get::<SocialStanding>(target_pop).unwrap().value;
        assert_eq!(target_standing, 40.0);
    }

    #[test]
    fn test_ostracization_on_low_standing() {
        // Arrange
        let mut app = App::new();
        let target_pop = app.world_mut().spawn(SocialStanding { value: -10.0 }).id();

        // Act
        app.update(); // Assume standing check system runs

        // Assert
        assert!(app.world().get::<Ostracized>(target_pop).is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct SocialStanding {
    pub value: f32,
}

#[derive(Component)]
pub struct Ostracized;

#[derive(Component)]
pub struct ColonyPop;

#[derive(Component)]
pub struct BulletinBoard;

#[derive(Event)]
pub struct PostGrievanceEvent {
    pub poster: Entity,
    pub target: Entity,
    pub board: Entity,
    pub impact: f32, // Positive for praise, negative for grievance
}

pub fn apply_grievance_system(
    mut events: EventReader<PostGrievanceEvent>,
    mut query: Query<&mut SocialStanding>,
) {
    for event in events.read() {
        if let Ok(mut standing) = query.get_mut(event.target) {
            standing.value += event.impact;
        }
    }
}

pub fn ostracization_system(
    mut query: Query<(Entity, &SocialStanding), Without<Ostracized>>,
    mut commands: Commands,
) {
    for (entity, standing) in query.iter_mut() {
        if standing.value <= 0.0 {
            commands.entity(entity).insert(Ostracized);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Limit how often a pop can post grievances to prevent spam and instantaneous ostracization.
- Pops that are ostracized should have restrictions applied to their interactions in `social_system` and `work_execution_system`.
- Integrate a mechanism for pops to regain social standing (e.g. through heroic acts).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] `PostGrievanceEvent` adjusts the target pop's `SocialStanding`.
- [ ] Pops with low standing correctly receive the `Ostracized` component.

## 7. Technical Guidance
- The `BulletinBoard` should be a requirement for pops to execute the action of posting a grievance (hooked into `UtilityAI`).
- The `Ostracized` component acts as a marker. Ensure other systems check for this marker before allowing trade or socializing.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
