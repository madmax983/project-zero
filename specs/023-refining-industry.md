# 023: Refining Industry

## 1. Overview
Introduces the first tier of industrial processing: refining raw materials into construction materials.
The colony can now build **Lumber Mills** (convert Wood to Planks) and **Stone Masons** (convert Stone to Blocks).
These refined materials will be required for advanced buildings in future specs.

## 2. Dependencies
- [x] `001` Project Scaffold
- [x] `018` Mining (provides Stone)
- [x] `019` Forestry (provides Wood)
- [x] `022` Resource Stockpiles (provides storage limits)

## 3. RED Phase: Tests First

```rust
// src/layer1/resources.rs
// Test that ColonyResources includes new materials
#[test]
fn test_colony_resources_refined_materials() {
    let resources = ColonyResources::default();
    assert_eq!(resources.planks, 0.0);
    assert_eq!(resources.blocks, 0.0);
    assert_eq!(resources.max_planks, 50.0); // Default cap
    assert_eq!(resources.max_blocks, 20.0); // Default cap
}

// src/layer1/building.rs
// Test that new building types exist and have costs
#[test]
fn test_refining_building_types() {
    let mill = BuildingType::LumberMill;
    let mason = BuildingType::StoneMason;

    assert_eq!(mill.label(), "Lumber Mill");
    assert_eq!(mason.label(), "Stone Mason");

    // Mill costs wood to build
    assert!(mill.cost().wood > 0.0);
    // Mason costs wood and stone to build
    assert!(mason.cost().wood > 0.0);
    assert!(mason.cost().stone > 0.0);
}

// src/layer1/refining.rs
// Test the lumber mill component
#[test]
fn test_lumber_mill_component() {
    let mill = LumberMill::default();
    assert_eq!(mill.capacity, 2); // 2 workers
    assert!(mill.workers.is_empty());
    assert_eq!(mill.progress, 0.0);
    assert_eq!(mill.max_progress, 10.0); // Work ticks per batch
}

// Test production logic (batch processing)
#[test]
fn test_process_lumber_mill_system() {
    let mut world = World::new();
    world.insert_resource(ColonyResources {
        wood: 10.0,
        planks: 0.0,
        ..Default::default()
    });

    let worker = world.spawn(Pop).id();
    let mut mill = LumberMill::default();
    mill.workers.push(worker);
    mill.max_progress = 1.0; // 1 tick to complete for test
    world.spawn(mill);

    // Run system
    process_lumber_mill_system(&mut world);

    let resources = world.resource::<ColonyResources>();
    assert_eq!(resources.wood, 9.0); // -1 Wood
    assert_eq!(resources.planks, 1.0); // +1 Plank
}

#[test]
fn test_process_lumber_mill_no_resources() {
    let mut world = World::new();
    world.insert_resource(ColonyResources {
        wood: 0.0, // No wood
        planks: 0.0,
        ..Default::default()
    });

    let worker = world.spawn(Pop).id();
    let mut mill = LumberMill::default();
    mill.workers.push(worker);
    mill.max_progress = 1.0;
    let entity = world.spawn(mill).id();

    process_lumber_mill_system(&mut world);

    let resources = world.resource::<ColonyResources>();
    assert_eq!(resources.planks, 0.0);

    let mill_after = world.get::<LumberMill>(entity).unwrap();
    assert_eq!(mill_after.progress, 0.0); // No progress made
}
```

## 4. GREEN Phase: Minimal Implementation

### `src/layer1/resources.rs`
- Add `planks`, `blocks`, `max_planks`, `max_blocks` to `ColonyResources`.
- Update `Default` impl.
- Add `add_planks`, `add_blocks` methods.

### `src/layer1/building.rs`
- Add `LumberMill`, `StoneMason` to `BuildingType` enum.
- Update `cost()` and `label()` methods.
- Update `spawn_building` to attach `LumberMill` / `StoneMason` components.

### `src/layer1/refining.rs` (New Module)
- Define `LumberMill` component.
  - Fields: `capacity`, `workers`, `progress`, `max_progress`.
- Define `StoneMason` component.
  - Fields: `capacity`, `workers`, `progress`, `max_progress`.
- Implement `process_lumber_mill_system`.
  - Query `LumberMill`.
  - Calculate total work (workers * speed).
  - Check `ColonyResources` for Wood.
  - Add progress.
  - If progress >= max: deduct 1 Wood, add 1 Plank, reset progress.
- Implement `process_stone_mason_system`.
  - Similar logic: Stone -> Blocks.

## 5. REFACTOR Phase: Quality & Design
- **Generic Refinery:** In the future, if we add Smelters/Weavers, we should create a generic `Refinery` component with `Recipe` struct to avoid code duplication. For now, separate components are fine for YAGNI.
- **Progress Visualization:** The `progress` field can be visualized in the UI (Inspector) later.
- **Job Assignment:** Ensure the `JobSystem` (or Utility AI) can assign workers to these buildings. (This spec focuses on the building mechanics; job assignment is handled by the AI seeking the `Work` action on these entities).

## 6. Acceptance Criteria
- [ ] `ColonyResources` tracks Planks and Blocks.
- [ ] `LumberMill` converts Wood -> Planks when worked.
- [ ] `StoneMason` converts Stone -> Blocks when worked.
- [ ] Production stops if input resources are missing.
- [ ] Production caps at `max_planks` / `max_blocks`.
- [ ] Test coverage > 85%.

## 7. Technical Guidance
- **Batch Processing:** Unlike farms (continuous trickle), refineries should work in batches (1 unit input -> 1 unit output). This feels more impactful and easier to balance.
- **Resource Locking:** Be careful not to deduct resources *before* work is complete, or vice-versa. Simplest approach: Deduct and Add simultaneously at completion. Only check availability during progress.
- **Work Speed:** Default to 1.0 progress per worker per tick. Configurable `max_progress` (e.g., 20 ticks = 1 plank).
