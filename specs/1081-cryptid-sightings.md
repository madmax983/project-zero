# 1081: Cryptid Sightings

## 1. Overview
There are things in the woods that don't show up on sensors. Rare, non-hostile entities spawn at map edges in Layer 1. They avoid pops but if seen, they cause "Awe" or "Dread" in the observing pops. Furthermore, they leave behind "Traces" (Slime, Fur, Artifacts) that can be studied by the colony. This provides a tension between capturing/killing the cryptid for science/resources, versus observing it for cultural mystery.

## 2. Dependencies
- Layer 1 terrain and pop systems.
- Pop perception or vision radius mechanics.
- Pop mood/memory system (for Awe/Dread).
- Item dropping mechanics for traces.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_cryptid_spawns_trace() {
        let mut app = App::new();
        app.add_systems(Update, cryptid_trace_system);
        app.init_resource::<Time>();

        let cryptid = app.world_mut().spawn((
            Cryptid { trace_timer: Timer::from_seconds(5.0, TimerMode::Repeating) },
            Transform::from_xyz(10.0, 0.0, 10.0),
        )).id();

        app.update();

        // Fast forward timer
        let mut timer = app.world_mut().get_mut::<Cryptid>(cryptid).unwrap();
        timer.trace_timer.tick(std::time::Duration::from_secs(6));

        app.update();

        let mut query = app.world_mut().query::<(&TraceItem, &Transform)>();
        let mut trace_count = 0;
        for (trace, transform) in query.iter(app.world()) {
            trace_count += 1;
            assert_eq!(transform.translation, Vec3::new(10.0, 0.0, 10.0));
            assert_eq!(trace.kind, TraceKind::Slime);
        }

        assert_eq!(trace_count, 1, "Cryptid should drop a trace");
    }

    #[test]
    fn test_pop_observes_cryptid() {
        let mut app = App::new();
        app.add_systems(Update, cryptid_observation_system);

        let _cryptid = app.world_mut().spawn((
            Cryptid { trace_timer: Timer::default() },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let pop = app.world_mut().spawn((
            Pop,
            Transform::from_xyz(2.0, 0.0, 0.0),
            PopMood { awe: 0.0, dread: 0.0 },
            VisionRadius(5.0),
        )).id();

        app.update();

        let mood = app.world().get::<PopMood>(pop).unwrap();
        assert!(mood.awe > 0.0 || mood.dread > 0.0, "Pop should feel awe or dread after seeing a cryptid");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Cryptid {
    pub trace_timer: Timer,
}

#[derive(Component)]
pub struct TraceItem {
    pub kind: TraceKind,
}

#[derive(PartialEq, Debug)]
pub enum TraceKind {
    Slime,
    Fur,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct PopMood {
    pub awe: f32,
    pub dread: f32,
}

#[derive(Component)]
pub struct VisionRadius(pub f32);

pub fn cryptid_trace_system(
    mut commands: Commands,
    mut query: Query<(&mut Cryptid, &Transform)>,
    time: Res<Time>,
) {
    for (mut cryptid, transform) in query.iter_mut() {
        cryptid.trace_timer.tick(time.delta());
        if cryptid.trace_timer.just_finished() {
            commands.spawn((
                TraceItem { kind: TraceKind::Slime },
                Transform::from_xyz(transform.translation.x, transform.translation.y, transform.translation.z),
            ));
        }
    }
}

pub fn cryptid_observation_system(
    cryptid_query: Query<&Transform, With<Cryptid>>,
    mut pop_query: Query<(&mut PopMood, &Transform, &VisionRadius), With<Pop>>,
) {
    for cryptid_transform in cryptid_query.iter() {
        for (mut mood, pop_transform, vision) in pop_query.iter_mut() {
            let dist = cryptid_transform.translation.distance(pop_transform.translation);
            if dist <= vision.0 {
                mood.awe += 1.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Hardcoded `TraceKind::Slime` and hardcoded awe/dread values. These should be driven by the specific type of cryptid.
- **Performance**: N-squared distance check between pops and cryptids. Given the rarity of cryptids, this might be fine, but could use spatial hashing if the number of pops gets huge.
- **Design Improvements**: Add specific logic for Pops to "avoid" or "follow" cryptids based on traits, or to form cults from repeated exposure.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Cryptids periodically drop traces at their location.
- [ ] Pops within vision range of a cryptid receive Awe or Dread mood adjustments.

## 7. Technical Guidance
- Integrate with Layer 1 map boundaries to handle spawning logic.
- Consider utilizing the `ChronicleEvent` system to record major cryptid sightings or the formation of cults.

## 8. Questions
*Builder: add questions here if spec is unclear.*
