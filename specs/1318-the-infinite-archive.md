# The Infinite Archive

**1. Overview**
Knowledge is infinite, but hard drives are not. Research generates "Data", which takes up physical space (Server Racks). As archives grow, "Search Time" increases, making tech unlocks slower. You must "Delete" old data (forgetting lower-tier tech) to learn new things efficiently. This creates a tension between the volume of knowledge and the accessibility of knowledge.

**2. Dependencies**
- Base Layer 1 simulation framework
- Research/Tech system
- Resource storage system

**3. RED Phase: Tests First**
```rust
#[test]
fn test_data_generation_consumes_storage() {
    let mut app = setup_test_app();
    let server_rack = app.world_mut().spawn(ServerRack { capacity: 100 }).id();
    app.world_mut().insert_resource(ResearchProgress { current_tech: TechId::Laser, progress: 0.0 });

    app.update();

    let storage = app.world().get::<DataStorage>(server_rack).unwrap();
    assert!(storage.used > 0);
}

#[test]
fn test_search_time_increases_with_data_volume() {
    let mut app_empty = setup_test_app();
    app_empty.world_mut().insert_resource(TotalData(0));
    app_empty.world_mut().insert_resource(ResearchProgress { current_tech: TechId::Laser, progress: 0.0 });

    let mut app_full = setup_test_app();
    app_full.world_mut().insert_resource(TotalData(1000));
    app_full.world_mut().insert_resource(ResearchProgress { current_tech: TechId::Laser, progress: 0.0 });

    app_empty.update();
    app_full.update();

    let empty_progress = app_empty.world().resource::<ResearchProgress>().progress;
    let full_progress = app_full.world().resource::<ResearchProgress>().progress;

    assert!(empty_progress > full_progress);
}

#[test]
fn test_deleting_data_forgets_tech_and_restores_speed() {
    let mut app = setup_test_app();
    app.world_mut().insert_resource(TotalData(1000));
    app.world_mut().insert_resource(UnlockedTechs(vec![TechId::SteamEngine]));

    let mut system_state = SystemState::<Commands>::new(app.world_mut());
    let mut commands = system_state.get_mut(app.world_mut());
    commands.trigger(DeleteTechDataEvent(TechId::SteamEngine));
    system_state.apply(app.world_mut());
    app.update();

    let unlocked = app.world().resource::<UnlockedTechs>();
    assert!(!unlocked.0.contains(&TechId::SteamEngine));
    assert!(app.world().resource::<TotalData>().0 < 1000);
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
pub struct ServerRack { pub capacity: u32 }
pub struct DataStorage { pub used: u32 }

#[derive(Resource)]
pub struct TotalData(pub u32);

#[derive(Resource)]
pub struct UnlockedTechs(pub Vec<TechId>);

#[derive(Resource)]
pub struct ResearchProgress {
    pub current_tech: TechId,
    pub progress: f32,
}

pub fn research_tick_system(
    mut progress: ResMut<ResearchProgress>,
    total_data: Res<TotalData>,
) {
    let penalty = 1.0 - (total_data.0 as f32 / 1000.0).clamp(0.0, 0.9);
    progress.progress += 1.0 * penalty;
}

pub fn delete_tech_system(
    mut events: EventReader<DeleteTechDataEvent>,
    mut unlocked: ResMut<UnlockedTechs>,
    mut total_data: ResMut<TotalData>,
) {
    for event in events.read() {
        if let Some(pos) = unlocked.0.iter().position(|t| *t == event.0) {
            unlocked.0.remove(pos);
            total_data.0 = total_data.0.saturating_sub(100);
        }
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- Consider extracting the search time penalty calculation into a separate helper or resource multiplier for reusability.
- Manage data storage per `ServerRack` entity properly instead of just a global `TotalData` resource for more complex spatial gameplay.
- Add `EventWriter` for `TechForgottenEvent` so other systems (like UI or available building menus) can react appropriately.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified (data volume slows research, forgetting restores speed)

**7. Technical Guidance**
- Tie data footprint to the complexity of the tech. Steam Engine should take less space than Anti-Matter Physics.
- The interaction with building availability is crucial. If a tech is forgotten, the player should not be able to build new instances of that tech's buildings until relearned.

**8. Questions**
*Builder: add questions here if spec is unclear.*
