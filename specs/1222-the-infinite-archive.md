# 1222: The Infinite Archive

## 1. Overview
**Layer:** 1

**Fantasy:** Knowledge is infinite, but hard drives are not.

**Mechanic:** Research generates "Data". Data takes up physical space (Server Racks). As archives grow, "Search Time" increases (tech unlocks slower). You must "Delete" old data (forgetting lower-tier tech) to learn new things efficiently.

**Emergence:** To learn "Anti-Matter Physics", you have to delete the knowledge of "Steam Engines". Later, a magnetic storm kills your high-tech grid, and you realize you forgot how to build a steam boiler.

**Tension:** Volume of Knowledge vs. Accessibility of Knowledge.

## 2. Dependencies
- Tech / Research system
- Building system (Server Racks)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_data_volume_increases_research_time() {
    let mut app = App::new();
    app.add_systems(Update, calculate_research_efficiency);

    // High data volume
    app.world_mut().insert_resource(DataArchive { current_data: 5000.0 });

    // Spawn a researcher
    let researcher = app.world_mut().spawn((
        Pop,
        Researcher { base_speed: 1.0, current_speed: 1.0 },
    )).id();

    app.update();

    let res = app.world().get::<Researcher>(researcher).unwrap();
    // Speed should be reduced due to massive archive search times
    assert!(res.current_speed < res.base_speed);
}

#[test]
fn test_deleting_tech_frees_data() {
    let mut app = App::new();
    app.add_systems(Update, process_data_deletion);

    app.world_mut().insert_resource(DataArchive { current_data: 1000.0 });
    app.world_mut().insert_resource(Events::<DeleteTechEvent>::default());

    // Send event to delete a tech that costs 200 data
    app.world_mut().send_event(DeleteTechEvent { data_freed: 200.0 });

    app.update();

    let archive = app.world().resource::<DataArchive>();
    assert_eq!(archive.current_data, 800.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn calculate_research_efficiency(
    archive: Res<DataArchive>,
    mut query: Query<&mut Researcher>,
) {
    for mut researcher in query.iter_mut() {
        // Slow down by 1% for every 100 data
        let penalty = (archive.current_data / 100.0) * 0.01;
        researcher.current_speed = (researcher.base_speed - penalty).max(0.1);
    }
}

fn process_data_deletion(
    mut events: EventReader<DeleteTechEvent>,
    mut archive: ResMut<DataArchive>,
) {
    for ev in events.read() {
        archive.current_data = (archive.current_data - ev.data_freed).max(0.0);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Hook the "Delete Tech" event into the actual technology tree to remove the recipes from the colony's unlocked list.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Verify integration with `src/layer1/tech/mod.rs` or equivalent.

## 8. Questions
*Builder: add questions here if spec is unclear.*
