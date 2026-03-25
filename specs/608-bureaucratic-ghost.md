# The Bureaucratic Ghost

**1. Overview**
Red tape so thick that even dead people are still filling out forms. If your colony's "Admin" capacity is heavily strained for a long time, the system begins to glitch. Dead Pops are not removed from the rosters. They are still assigned jobs, sent rations, and even "vote" in faction disputes. Living Pops become stressed trying to cover the shifts of their deceased coworkers, whom the system insists are just "running late." Expanding rapidly without building necessary, non-productive administrative infrastructure leads to the surreal, paralyzing horror of an automated system that refuses to acknowledge death.

**2. Dependencies**
- `layer1::admin::AdminCapacity`
- `layer1::population::PopDeath`
- `layer1::jobs::JobAssignment`
- `layer1::needs::Stress`

**3. RED Phase: Tests First**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_ghost_pop_retention() {
        let mut app = App::new();
        app.add_systems(Update, process_pop_death_system);

        // Arrange
        app.world_mut().insert_resource(AdminCapacity { current: 150, max: 100 }); // Over capacity
        let pop = app.world_mut().spawn((Pop, Dead, Job { role: "Engineer".to_string() })).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<BureaucraticGhost>(pop).is_some(), "Over-capacity admin should turn dead pop into a ghost instead of despawning");
        assert!(app.world().get::<Job>(pop).is_some(), "Ghost pop should retain its job");
    }

    #[test]
    fn test_living_coworker_stress() {
        let mut app = App::new();
        app.add_systems(Update, apply_ghost_coworker_stress_system);

        // Arrange
        app.world_mut().spawn((Pop, BureaucraticGhost, Job { role: "Engineer".to_string() }));
        let living_pop = app.world_mut().spawn((Pop, Stress(0), Job { role: "Engineer".to_string() })).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<Stress>(living_pop).unwrap().0 > 0, "Living pops should gain stress from covering for ghost coworkers");
    }

    #[test]
    fn test_ghost_resource_consumption() {
        let mut app = App::new();
        app.add_systems(Update, process_ghost_rations_system);

        // Arrange
        app.world_mut().insert_resource(ColonyRations { total: 100 });
        app.world_mut().spawn((Pop, BureaucraticGhost));
        app.world_mut().spawn((Pop, BureaucraticGhost));

        // Act
        app.update();

        // Assert
        assert!(app.world().resource::<ColonyRations>().total < 100, "Ghosts should still consume colony rations due to system errors");
    }
}
```

**4. GREEN Phase: Minimal Implementation**

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct BureaucraticGhost;

#[derive(Component)]
pub struct Job {
    pub role: String,
}

#[derive(Component)]
pub struct Stress(pub u32);

#[derive(Resource)]
pub struct AdminCapacity {
    pub current: u32,
    pub max: u32,
}

#[derive(Resource)]
pub struct ColonyRations {
    pub total: u32,
}

pub fn process_pop_death_system(
    mut commands: Commands,
    admin: Res<AdminCapacity>,
    dead_pops: Query<Entity, (With<Pop>, With<Dead>)>,
) {
    let is_strained = admin.current > admin.max;
    for entity in dead_pops.iter() {
        if is_strained {
            commands.entity(entity).remove::<Dead>().insert(BureaucraticGhost);
        } else {
            commands.entity(entity).despawn();
        }
    }
}

pub fn apply_ghost_coworker_stress_system(
    ghosts: Query<&Job, With<BureaucraticGhost>>,
    mut living: Query<(&Job, &mut Stress), Without<BureaucraticGhost>>,
) {
    for ghost_job in ghosts.iter() {
        for (living_job, mut stress) in living.iter_mut() {
            if living_job.role == ghost_job.role {
                stress.0 += 10;
            }
        }
    }
}

pub fn process_ghost_rations_system(
    mut rations: ResMut<ColonyRations>,
    ghosts: Query<(), With<BureaucraticGhost>>,
) {
    let ghost_count = ghosts.iter().count() as u32;
    if rations.total >= ghost_count {
        rations.total -= ghost_count;
    } else {
        rations.total = 0;
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- Implement a threshold or duration for how long `AdminCapacity` must be strained before the system "glitches" (to avoid ghosts forming the instant capacity exceeds max).
- Include an audit mechanic to clear ghosts once `AdminCapacity` is restored to a healthy level.
- Ghost votes in faction disputes should be handled in the relevant faction systems, checking for `BureaucraticGhost` components.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Dead pops become `BureaucraticGhost`s instead of despawning when admin capacity is strained
- [ ] Ghost pops still hold jobs and draw rations
- [ ] Living pops with the same job role as a ghost gain stress

**7. Technical Guidance**
- The transition from `Dead` to `BureaucraticGhost` requires removing the `Dead` component to prevent other cleanup systems from grabbing the entity.
- The stress mechanic is O(N*M) in the minimal implementation. For large colonies, consider an intermediate resource (e.g., a `HashMap` of job role to ghost count) to optimize the query.

**8. Questions**
*Builder: add questions here if spec is unclear.*
