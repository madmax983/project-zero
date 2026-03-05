# 301 - The Memory Tithe

## 1. Overview
The Memory Tithe forces players to sacrifice the hard-earned experience of their most skilled colonists. A powerful Layer 3 empire demands "Memory Cores" as a form of tribute. Players must use a new building, the "Siphon Box," to extract these cores from their highly-skilled Pops, which resets their Job XP to zero. Failing to provide the required number of cores within the tribute cycle results in severe consequences (e.g., orbital bombardment or resource drain).

## 2. Dependencies
- `004` Pop Entity
- `051` Pop Skills and Experience
- `009` Job System
- `146` Command Center & System Visibility (for tribute demands)
- `152` Orbital Stations (if tribute is shipped off-planet)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_extract_memory_core_resets_xp() {
        // Arrange: Setup a Pop with high XP and a Siphon Box
        let mut app = App::new();
        let siphon_box = app.world_mut().spawn(SiphonBox).id();
        let pop = app.world_mut().spawn((
            Pop,
            JobSkills { xp: vec![(JobType::Scientist, 1000.0)] },
            UsingSiphonBox(siphon_box),
        )).id();
        app.insert_resource(TributeCores { count: 0 });

        app.add_systems(Update, process_memory_extraction);

        // Act: Run the extraction process
        app.update();

        // Assert: Pop's XP should be 0, and a Core should be generated
        let skills = app.world().get::<JobSkills>(pop).unwrap();
        assert_eq!(skills.xp[0].1, 0.0, "Pop XP should be reset to zero.");
        assert_eq!(app.world().resource::<TributeCores>().count, 1, "A Memory Core should be added.");
    }

    #[test]
    fn test_failing_tribute_demand_triggers_consequence() {
        // Arrange: Setup a failing tribute scenario
        let mut app = App::new();
        app.insert_resource(TributeCores { count: 0 }); // They have no cores
        app.insert_resource(TributeDemand { required: 5, timer: Timer::from_seconds(0.0, TimerMode::Once) });
        app.add_event::<TributeFailedEvent>();

        app.add_systems(Update, check_tribute_deadline);

        // Act: Run the deadline check (timer expired)
        app.update();

        // Assert: A failure event should be emitted
        let failed_events: Vec<&TributeFailedEvent> = app.world().resource::<Events<TributeFailedEvent>>().get_reader().read(&app.world().resource::<Events<TributeFailedEvent>>()).collect();
        assert!(!failed_events.is_empty(), "TributeFailedEvent should be emitted.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// --- Components and Resources ---
#[derive(Component)]
pub struct Pop;

#[derive(Clone, PartialEq, Eq)]
pub enum JobType {
    Scientist,
    Engineer,
    // Add other relevant job types
}

#[derive(Component)]
pub struct JobSkills {
    pub xp: Vec<(JobType, f32)>,
}

#[derive(Component)]
pub struct SiphonBox;

#[derive(Component)]
pub struct UsingSiphonBox(pub Entity);

#[derive(Resource, Default)]
pub struct TributeCores {
    pub count: u32,
}

#[derive(Resource)]
pub struct TributeDemand {
    pub required: u32,
    pub timer: Timer,
}

#[derive(Event)]
pub struct TributeFailedEvent;

// --- Systems ---
pub fn process_memory_extraction(
    mut query: Query<(Entity, &mut JobSkills, &UsingSiphonBox)>,
    mut cores: ResMut<TributeCores>,
    mut commands: Commands,
) {
    for (entity, mut skills, _) in query.iter_mut() {
        let mut highest_xp_index = 0;
        let mut max_xp = 0.0;

        for (i, (_, xp)) in skills.xp.iter().enumerate() {
            if *xp > max_xp {
                max_xp = *xp;
                highest_xp_index = i;
            }
        }

        // If they have enough XP to form a core (e.g., threshold of 500)
        if max_xp >= 500.0 {
            skills.xp[highest_xp_index].1 = 0.0; // Reset XP
            cores.count += 1;
        }

        // Remove the component so they don't get siphoned continuously
        commands.entity(entity).remove::<UsingSiphonBox>();
    }
}

pub fn check_tribute_deadline(
    mut events: EventWriter<TributeFailedEvent>,
    time: Res<Time>,
    mut demand: ResMut<TributeDemand>,
    mut cores: ResMut<TributeCores>,
) {
    if demand.timer.tick(time.delta()).just_finished() {
        if cores.count >= demand.required {
            // Tribute paid
            cores.count -= demand.required;
            // Reset timer for next cycle (e.g., 30 in-game days)
            demand.timer.reset();
        } else {
            // Tribute failed
            events.send(TributeFailedEvent);
            // Optionally, penalize the missing cores instead of taking all, or set a new timer.
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactor Opportunity:** The `max_xp >= 500.0` check is hardcoded. Extract this into a `TributeConfig` resource.
- **Refactor Opportunity:** Handle the case where the Pop has multiple skills over the threshold; should they lose all of them, or just the highest? Design suggests taking a "core" per high skill.
- **Design Improvement:** Add a `MemoryLossTrauma` trait or stress debuff to Pops who are repeatedly siphoned, creating an "Amnesiac" caste.
- **Integration:** The `TributeFailedEvent` should trigger a massive negative consequence (e.g., Orbital Strike event) handled by another system.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.
- [ ] Pops using the Siphon Box lose XP correctly and generate cores.
- [ ] Failing to meet the `TributeDemand` timer emits the `TributeFailedEvent`.

## 7. Technical Guidance
- **ECS Pattern:** Use the `UsingSiphonBox` component as a marker during the interaction phase (e.g., a Job or Task) and clear it immediately after extraction to prevent infinite loops.
- **Seams:** Hook into the UI to prominently display the `TributeDemand` timer and current `TributeCores` count, as this is a core pressure mechanic.
- **Balance:** The XP threshold must be high enough that players must intentionally cultivate high-level specialists just to harvest them.

## 8. Questions
*Builder: add questions here if spec is unclear.*
