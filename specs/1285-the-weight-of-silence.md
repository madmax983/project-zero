# 1285: The Weight of Silence

## 1. Overview
**Layer:** Cross-layer (1 -> 3)

**Fantasy:** The eerie, growing anxiety of a colony that hasn't heard from the rest of the empire in centuries.

**Mechanic:** When a colony goes without incoming trade or communication from other nodes for an extended period, a new "Isolation" need begins to grow. High isolation spawns a "Silence Cult" that actively sabotages communication arrays to maintain the quiet.

## 2. Dependencies
- Base Layer 1 Population System
- Needs System
- Faction/Cult System
- Cross-Layer Communication (Layer 1 -> Layer 3)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // RED Phase Test Setup
    fn setup_app() -> App {
        let mut app = App::new();
        // Add systems
        app.add_systems(Update, (
            track_colony_isolation_system,
            process_isolation_needs_system,
            spawn_silence_cult_system,
        ));
        app
    }

    #[test]
    fn test_colony_isolation_increases_over_time() {
        let mut app = setup_app();

        let colony = app.world_mut().spawn(ColonyNode {
            last_communication_tick: 0,
            isolation_level: 0.0,
        }).id();

        app.insert_resource(SimulationTick(1000));
        app.update();

        let colony_data = app.world().get::<ColonyNode>(colony).unwrap();
        assert!(colony_data.isolation_level > 0.0, "Isolation level should increase over time without communication");
    }

    #[test]
    fn test_pops_gain_isolation_need() {
        let mut app = setup_app();

        let colony = app.world_mut().spawn(ColonyNode {
            last_communication_tick: 0,
            isolation_level: 100.0,
        }).id();

        let pop = app.world_mut().spawn((
            Pop,
            ResidentOf(colony),
            Needs { isolation: 0.0 }, // New need
        )).id();

        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(needs.isolation > 0.0, "Pop isolation need should increase if colony is highly isolated");
    }

    #[test]
    fn test_silence_cult_spawns_at_high_isolation() {
        let mut app = setup_app();

        let colony = app.world_mut().spawn(ColonyNode {
            last_communication_tick: 0,
            isolation_level: 500.0, // High enough to trigger cult
        }).id();

        let pop = app.world_mut().spawn((
            Pop,
            ResidentOf(colony),
            Needs { isolation: 100.0 },
        )).id();

        app.update();

        // Check if cult was spawned
        let mut cult_query = app.world_mut().query::<&SilenceCult>();
        assert!(cult_query.iter(app.world()).count() > 0, "Silence Cult should spawn at high isolation");

        // Check if pop joined
        assert!(app.world().get::<CultMember>(pop).is_some(), "Pop with high isolation need should join the Silence Cult");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct SimulationTick(pub u64);

#[derive(Component)]
pub struct ColonyNode {
    pub last_communication_tick: u64,
    pub isolation_level: f32,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct ResidentOf(pub Entity);

#[derive(Component, Default)]
pub struct Needs {
    pub isolation: f32,
}

#[derive(Component)]
pub struct SilenceCult {
    pub colony_entity: Entity,
}

#[derive(Component)]
pub struct CultMember;

pub fn track_colony_isolation_system(
    tick: Option<Res<SimulationTick>>,
    mut colonies: Query<&mut ColonyNode>,
) {
    if let Some(tick) = tick {
        for mut colony in colonies.iter_mut() {
            let time_since_comm = tick.0.saturating_sub(colony.last_communication_tick);
            // Minimal implementation: isolation level increases by 0.1 per tick without comms
            colony.isolation_level = (time_since_comm as f32) * 0.1;
        }
    }
}

pub fn process_isolation_needs_system(
    colonies: Query<&ColonyNode>,
    mut pops: Query<(&ResidentOf, &mut Needs)>,
) {
    for (resident_of, mut needs) in pops.iter_mut() {
        if let Ok(colony) = colonies.get(resident_of.0) {
            if colony.isolation_level > 50.0 {
                 needs.isolation += 1.0;
            }
        }
    }
}

pub fn spawn_silence_cult_system(
    mut commands: Commands,
    colonies: Query<(Entity, &ColonyNode)>,
    mut pops: Query<(Entity, &ResidentOf, &Needs), Without<CultMember>>,
    existing_cults: Query<&SilenceCult>,
) {
    for (colony_entity, colony) in colonies.iter() {
        if colony.isolation_level >= 500.0 {
             // Check if cult already exists for this colony
             let has_cult = existing_cults.iter().any(|c| c.colony_entity == colony_entity);

             if !has_cult {
                 commands.spawn(SilenceCult { colony_entity });
             }

             // Indoctrinate pops
             for (pop_entity, resident_of, needs) in pops.iter_mut() {
                 if resident_of.0 == colony_entity && needs.isolation >= 100.0 {
                     commands.entity(pop_entity).insert(CultMember);
                 }
             }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Configuration Resource**: Move magic numbers (e.g., `50.0`, `500.0`, `0.1`) to a `SilenceCultConfig` resource.
- **Needs System Integration**: Ensure `Needs { isolation: f32 }` fits with the existing broader needs architecture (if it's a `HashMap` or a larger struct).
- **Sabotage Logic**: The next step after this foundational spec is to add the actual communication array sabotage behavior for `CultMember`s.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- **Timestamps**: Ensure `last_communication_tick` is updated reliably by whatever Layer 3 networking system handles trade/messages.
- **Entity Linking**: The `ResidentOf` pattern is used here, but adapt to the project's standard parent/child or specific node-linking component structure.

## 8. Questions
*Builder: add questions here if spec is unclear.*
