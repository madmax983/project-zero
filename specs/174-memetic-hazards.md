# 174: Memetic Hazards

**Layer:** 1
**Status:** In Progress
**Fantasy:** Knowledge is dangerous. Some ideas are viruses that compel their hosts to spread them.
**Mechanic:** Researching "Hazardous" technologies risks infecting researchers with a `MemeticCarrier` component. Carriers stop productive work to scrawl "Memetic Sigils" (Graffiti) on walls. Observing these sigils can infect other Pops.

---

## 1. Overview

Memetic Hazards introduce a risk/reward dynamic to high-tier research. Instead of purely costing resources, some knowledge costs sanity.

-   **Vector 1 (Patient Zero):** Unlocking a Hazardous Tech has a % chance to infect a random Pop working in a Library.
-   **Vector 2 (Contagion):** Infected Pops prioritize the `ScrawlMemeticSigil` action, placing `GraffitiType::MemeticSigil` on walls.
-   **Vector 3 (Observation):** Pops viewing a `MemeticSigil` have a chance to become infected.

Infected Pops:
-   Do not work normal jobs.
-   Do not sleep/eat efficiently (lower priority).
-   Obsessively create more Sigils.
-   Can be cured by `Medical` intervention (Psychotherapy) or decay over time (if non-terminal).

## 2. Dependencies

-   `011` Tech Tree Backend (for `unlock_tech` hook).
-   `144` Graffiti and Signage (for `GraffitiType` and placement).
-   `003` Pop Entity (for `MemeticCarrier` component).
-   `013` Utility AI (for `ActionType::ScrawlMemeticSigil`).

## 3. RED Phase: Tests First

These tests define the feature and MUST fail initially.

```rust
#[test]
fn test_unlocking_hazardous_tech_infects_researcher() {
    let mut world = World::new();
    // Setup TechState, Resources, and a Researcher Pop
    // Need to register necessary components

    let researcher = world.spawn((
        Pop,
        AssignedTo { assignment_type: AssignmentType::LibraryWorker, ..Default::default() },
        GridPosition::default()
    )).id();

    // Define a hazardous tech (e.g. VoidWhispers)
    // Unlock it
    let result = unlock_tech(&mut world, Tech::VoidWhispers);

    // Assert success
    assert!(result);
    // Assert researcher is infected
    assert!(world.get::<MemeticCarrier>(researcher).is_some());
}

#[test]
fn test_infected_pop_scrawls_sigil() {
    let mut world = World::new();
    // Setup infected pop
    let pop = world.spawn((
        Pop,
        MemeticCarrier,
        GridPosition { x: 5, y: 5 },
        UtilityWeights::default(),
        PopAction::default(),
        Needs::default()
    )).id();

    // Setup wall at (5,6) to scrawl on
    world.spawn((
        Building { building_type: BuildingType::Wall },
        GridPosition { x: 5, y: 6 },
        OccupiedTiles::default() // Mock or setup correctly
    ));

    // Insert necessary resources (UtilityConfig, etc)
    world.insert_resource(UtilityConfig::default());

    // Evaluate actions
    evaluate_actions_system(&mut world);

    // Assert Pop chose ScrawlMemeticSigil
    let action = world.get::<PopAction>(pop).unwrap();
    assert_eq!(action.current, ActionType::ScrawlMemeticSigil);
}

#[test]
fn test_observing_sigil_spreads_infection() {
    let mut world = World::new();
    // Setup clean pop
    let victim = world.spawn((
        Pop,
        GridPosition { x: 5, y: 5 },
        Morale::default()
    )).id();

    // Place Memetic Sigil at (5,6)
    let mut map = GraffitiMap::default();
    map.markings.insert(
        (5, 6),
        Graffiti {
            graffiti_type: GraffitiType::MemeticSigil,
            decay: 100.0,
            modifier: -0.1
        }
    );
    world.insert_resource(map);

    // Run observation system
    graffiti_observation_system(&mut world);

    // Assert victim is infected
    assert!(world.get::<MemeticCarrier>(victim).is_some());
}
```

## 4. GREEN Phase: Minimal Implementation

1.  **Tech Hazard**:
    -   Add `hazardous() -> bool` to `Tech` enum.
    -   Modify `unlock_tech` to check `hazardous()`.
    -   If true, query `LibraryWorker` entities and add `MemeticCarrier` component to one (randomly).

2.  **Memetic Carrier**:
    -   Define `#[derive(Component)] struct MemeticCarrier;`.

3.  **Scrawl Action**:
    -   Add `ActionType::ScrawlMemeticSigil`.
    -   Add `evaluate_scrawl_memetic_sigil` to AI. It should have **very high** utility for Carriers (override work/needs).

4.  **Graffiti Expansion**:
    -   Add `GraffitiType::MemeticSigil`.
    -   Update `graffiti_observation_system`: If observing `MemeticSigil`, roll chance to add `MemeticCarrier`.

## 5. REFACTOR Phase

-   **Configurable Risk**: Move infection chance to `MemeticConfig` resource.
-   **Immunity**: Add `Trait::IronWill` or `Trait::Oblivious` that resists infection.
-   **Cure**: Add `Medical` job to remove `MemeticCarrier`.
-   **Decay**: Make `MemeticCarrier` have a duration or severity.

## 6. Acceptance Criteria

-   [ ] New `Tech` variants marked as Hazardous exist.
-   [ ] Researching them triggers infection.
-   [ ] Infected pops prioritize placing Sigils.
-   [ ] Sigils spread infection to observers.
-   [ ] Tests pass.

## 7. Technical Guidance

-   **Integration**: Hook into `unlock_tech` in `src/layer1/tech.rs`.
-   **AI**: Create `src/layer1/actions/scrawl_sigil.rs` for the evaluation logic.
-   **Graffiti**: Extend `src/layer1/graffiti.rs` carefully.
