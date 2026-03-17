# Specification 495: The Dream-State Economy

## 1. Overview
Pops sleeping in high-tech "Dream Pods" generate a new resource: "Dream Data". This data can be refined into "Inspiration" (boosts research/crafting quality) or "Horror" (weaponized stress). Over-harvesting leads to collective sleep deprivation and "Waking Nightmares" (hallucinations that disrupt work).

## 2. Dependencies
- Needs System (Sleep)
- Resource System (Dream Data, Inspiration, Horror)
- Buildings System (Dream Pods)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_sleeping_in_dream_pod_generates_dream_data() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(Layer1Plugin);

    // Arrange: Spawn a Dream Pod and a sleeping Pop
    let pop_entity = app.world_mut().spawn((
        PopBundle::default(),
        SleepNeed(100.0),
        SleepingIn(DreamPodId(1)),
    )).id();

    app.world_mut().insert_resource(ColonyInventory::default());

    // Act: Tick the sleep cycle system
    app.update();

    // Assert: Sleep is restored and Dream Data is generated
    let inventory = app.world().get_resource::<ColonyInventory>().unwrap();
    assert!(inventory.get_amount(ResourceType::DreamData) > 0);
}

#[test]
fn test_overharvesting_dream_data_causes_waking_nightmares() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(Layer1Plugin);

    // Arrange: A Pop sleeping in an "Overclocked" Dream Pod
    let pop_entity = app.world_mut().spawn((
        PopBundle::default(),
        SleepNeed(10.0), // Need sleep
        SleepingIn(DreamPodId(2)),
    )).id();

    app.world_mut().spawn((
        DreamPod { harvest_rate: 2.0, is_overclocked: true },
    ));

    // Act: Wake the pop up after the cycle
    app.update(); // Sleep cycle
    app.world_mut().send_event(WakeUpEvent { entity: pop_entity });
    app.update(); // Wake event processing

    // Assert: Pop gains WakingNightmare debuff
    assert!(app.world().get::<WakingNightmare>(pop_entity).is_some());
    let stress = app.world().get::<Stress>(pop_entity).unwrap();
    assert!(stress.0 > 0.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// Add `DreamData` to resources. If `SleepingIn` points to a `DreamPod`, yield `DreamData`.
// If `DreamPod.is_overclocked`, insert `WakingNightmare` and `Stress` on wake up.
```

## 5. REFACTOR Phase: Quality & Design
- Make Dream Pods an upgrade to existing beds or a specialized building.
- Refine the threshold for "Waking Nightmares" to scale with consecutive nights in a pod or a specific harvest toggle on the pod.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Pops successfully generate Dream Data when sleeping in specific pods, but suffer Waking Nightmares when over-harvested.

## 7. Technical Guidance
- Add `DreamPod` component to beds.
- Introduce `DreamData`, `Inspiration`, and `Horror` to `ResourceType` enum.
- Introduce a `DreamHarvestRate` setting on the `DreamPod`.
- The `WakingNightmare` component should severely reduce work efficiency or occasionally interrupt actions.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
