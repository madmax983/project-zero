# 339 - Mentorship

## 1. Overview
The **Mentorship** feature models the transfer of generational knowledge. When a highly skilled Pop (Master) works alongside a less skilled Pop (Novice) on the same task or in the same room, the Novice gains XP significantly faster. The Master receives a mood buff for teaching, and the Novice receives a mood buff for learning. This incentivizes pairing experienced workers with new arrivals rather than just optimizing for pure immediate efficiency.

## 2. Dependencies
- `051` Pop Skills and Experience
- `016` Utility AI System
- `005` Pop Needs

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_novice_gains_xp_bonus_near_master() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, mentorship_xp_system);

        let master_skills = app.world_mut().spawn((
            Skills { level: 80.0, current_job: JobType::Mining },
            Transform::from_xyz(10.0, 10.0, 0.0),
        )).id();

        let novice_skills = app.world_mut().spawn((
            Skills { level: 10.0, current_job: JobType::Mining },
            Transform::from_xyz(11.0, 11.0, 0.0), // Adjacent
        )).id();

        // Store initial XP
        let initial_xp = app.world().get::<Skills>(novice_skills).unwrap().xp;

        // Act - Simulate work
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(1));
        app.update();

        // Assert
        let final_xp = app.world().get::<Skills>(novice_skills).unwrap().xp;
        let base_xp_gain = 1.0; // Assume 1 XP per tick without mentorship
        assert!(final_xp > initial_xp + base_xp_gain, "Novice should gain bonus XP when working near a Master.");
    }

    #[test]
    fn test_mentorship_grants_mood_buffs() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, mentorship_mood_system);

        let master = app.world_mut().spawn((
            Skills { level: 90.0, current_job: JobType::Crafting },
            Needs { mood: 50.0 },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let novice = app.world_mut().spawn((
            Skills { level: 5.0, current_job: JobType::Crafting },
            Needs { mood: 50.0 },
            Transform::from_xyz(1.0, 1.0, 0.0),
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<Needs>(master).unwrap().mood > 50.0, "Master should gain mood buff from teaching.");
        assert!(app.world().get::<Needs>(novice).unwrap().mood > 50.0, "Novice should gain mood buff from learning.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Skills {
    pub level: f32,
    pub xp: f32,
    pub current_job: JobType,
}

#[derive(PartialEq, Clone)]
pub enum JobType {
    Mining,
    Crafting,
    Farming,
    Idle,
}

#[derive(Component)]
pub struct Needs {
    pub mood: f32,
}

const MENTORSHIP_RADIUS: f32 = 3.0;
const SKILL_THRESHOLD_DIFF: f32 = 40.0;
const BONUS_XP_MULTIPLIER: f32 = 2.0;
const MOOD_BUFF: f32 = 0.5;

pub fn mentorship_xp_system(
    time: Res<Time>,
    mut workers: Query<(Entity, &mut Skills, &Transform)>,
) {
    let mut xp_gains: Vec<(Entity, f32)> = Vec::new();
    let mut masters_list: Vec<(Entity, f32, JobType, Vec3)> = Vec::new();

    // Identify masters
    for (entity, skills, transform) in workers.iter() {
        if skills.level >= 50.0 { // Arbitrary master threshold
            masters_list.push((entity, skills.level, skills.current_job.clone(), transform.translation));
        }
    }

    // Apply bonus XP to novices near masters doing the same job
    for (entity, mut skills, transform) in workers.iter_mut() {
        let mut base_xp_gain = 1.0 * time.delta_seconds(); // Simplified base gain
        let mut mentored = false;

        for (_, master_level, master_job, master_pos) in &masters_list {
            if skills.current_job == *master_job &&
               *master_level > skills.level + SKILL_THRESHOLD_DIFF &&
               transform.translation.distance(*master_pos) <= MENTORSHIP_RADIUS {
                mentored = true;
                break;
            }
        }

        if mentored {
            base_xp_gain *= BONUS_XP_MULTIPLIER;
        }

        skills.xp += base_xp_gain;
    }
}

pub fn mentorship_mood_system(
    mut workers: Query<(Entity, &Skills, &mut Needs, &Transform)>,
) {
    // Simplified O(N^2) for brevity; refactor needed for large pop counts
    let mut mood_buffs: Vec<Entity> = Vec::new();

    // Logic to identify pairs and collect entities omitted for brevity...
    // Assume we've identified entities `master_ent` and `novice_ent` interacting
    // mood_buffs.push(master_ent);
    // mood_buffs.push(novice_ent);

    for (entity, _skills, mut needs, _transform) in workers.iter_mut() {
        if mood_buffs.contains(&entity) {
            needs.mood += MOOD_BUFF;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**:
  - Change the `O(N^2)` check in the mentorship systems. Instead of checking every worker against every master, use a spatial partition or room-based check. If a room is designated a `Workshop`, check the occupants.
  - Apply mood buffs via the existing `Memory` system or status effects rather than directly mutating the `Needs` float every frame.
- **Code Smells**:
  - `JobType` needs to align with the existing `AssignmentType` enum. Ensure `current_job` is derived dynamically from what the pop is actually *doing* that tick, rather than just their general assignment.
- **Performance**:
  - Use Bevy's spatial queries if available, or update mentorship links periodically (e.g., once every 5 seconds) rather than every tick.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Novices working near a Master on the same task gain XP significantly faster.
- [ ] Both parties receive a mood/memory buff.

## 7. Technical Guidance
- Map this to the existing `JobTenure` (from Spec 264) or the core `Skills` struct.
- Consider what defines a "Master". Is it a flat skill value (e.g., >80), or just being significantly better than the novice (e.g., +40 levels higher)?
- Hook into the `AddChronicleEvent` or `Memories` if a Pop reaches a "Master" rank.

## 8. Questions
*Builder: add questions here if spec is unclear.*
