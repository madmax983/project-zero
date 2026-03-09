# 186 Bio-Architecture

**Layer:** 1 (Colony)
**Status:** Draft
**Type:** Mechanic

## 1. Overview

Bio-Architecture introduces "grown" buildings that function as living organisms. Unlike traditional structures constructed from dead materials (Stone, Metal), Bio-Structures are cultivated.

**Core Mechanics:**
1.  **Regeneration:** Bio-Structures automatically repair damage over time (`Health` increases) if their needs are met.
2.  **Hunger (Upkeep):** Instead of maintenance repair kits, Bio-Structures consume `Food` (representing Nutrient Paste) and `Water` from the colony stockpile periodically.
3.  **Starvation:** If upkeep fails, Bio-Structures take damage over time and stop regenerating.
4.  **Sickness:** Bio-Structures can become `Sick`, halting regeneration and reducing efficiency.
5.  **Infection:** Extreme neglect or specific events can cause `Infection`, turning the building hostile (dealing damage to occupants) or converting it to a threat.

## 2. Dependencies

*   `004` Building System (Basic `Structure` component)
*   `017` Structure Decay (Bio-Structures replace standard decay with Starvation)
*   `112` Maintenance Debt (Bio-Structures opt-out of standard maintenance debt in favor of upkeep)
*   `032` Entropy/Spoilage (Food/Water availability)

## 3. RED Phase: Tests First

These tests define the behavior of `BioStructure` components.

```rust
#[test]
fn test_bio_structure_regeneration() {
    // Arrange: A damaged BioStructure with full upkeep available
    let mut world = World::new();
    let bio_entity = world.spawn((
        Structure { max_hp: 100.0, hp: 50.0 }, // Damaged
        BioStructure {
            state: BioState::Healthy,
            hunger: 0.0, // Not hungry yet
            upkeep_cost: BioUpkeep { food: 1.0, water: 1.0 },
        },
        BioRegenRate(5.0), // Heals 5 HP per tick
    )).id();

    // Setup resources
    let mut resources = ColonyResources::default();
    resources.food = 100.0;
    resources.water = 100.0;
    world.insert_resource(resources);

    // Act: Run regeneration system
    let mut schedule = Schedule::new();
    schedule.add_systems(bio_regeneration_system);
    schedule.run(&mut world);

    // Assert: HP increased
    let structure = world.get::<Structure>(bio_entity).unwrap();
    assert_eq!(structure.hp, 55.0);
}

#[test]
fn test_bio_structure_consumes_upkeep() {
    // Arrange: A BioStructure that is "Hungry" (timer/threshold trigger)
    let mut world = World::new();
    let bio_entity = world.spawn((
        Structure { max_hp: 100.0, hp: 100.0 },
        BioStructure {
            state: BioState::Healthy,
            hunger: 10.0, // Threshold to eat is 10.0
            upkeep_cost: BioUpkeep { food: 2.0, water: 1.0 },
        },
    )).id();

    let mut resources = ColonyResources::default();
    resources.food = 10.0;
    resources.water = 10.0;
    world.insert_resource(resources);

    // Act: Run upkeep system
    let mut schedule = Schedule::new();
    schedule.add_systems(bio_upkeep_system);
    schedule.run(&mut world);

    // Assert: Resources consumed, hunger reset
    let resources = world.resource::<ColonyResources>();
    assert_eq!(resources.food, 8.0);
    assert_eq!(resources.water, 9.0);

    let bio = world.get::<BioStructure>(bio_entity).unwrap();
    assert_eq!(bio.hunger, 0.0);
}

#[test]
fn test_bio_structure_starvation_damage() {
    // Arrange: A BioStructure that is "Hungry" but NO resources available
    let mut world = World::new();
    let bio_entity = world.spawn((
        Structure { max_hp: 100.0, hp: 100.0 },
        BioStructure {
            state: BioState::Healthy,
            hunger: 10.0, // Hungry
            upkeep_cost: BioUpkeep { food: 5.0, water: 5.0 },
        },
    )).id();

    // Empty resources
    world.insert_resource(ColonyResources::zeroed());

    // Act: Run upkeep system
    let mut schedule = Schedule::new();
    schedule.add_systems(bio_upkeep_system);
    schedule.run(&mut world);

    // Assert: HP decreased (Starvation damage)
    let structure = world.get::<Structure>(bio_entity).unwrap();
    assert!(structure.hp < 100.0);

    // Assert: State might change to Starving?
    let bio = world.get::<BioStructure>(bio_entity).unwrap();
    // Assuming Starvation adds damage, maybe state change is separate
}

#[test]
fn test_bio_structure_sickness_stops_regen() {
    // Arrange: A Sick BioStructure
    let mut world = World::new();
    let bio_entity = world.spawn((
        Structure { max_hp: 100.0, hp: 50.0 },
        BioStructure {
            state: BioState::Sick, // SICK
            hunger: 0.0,
            upkeep_cost: BioUpkeep { food: 1.0, water: 1.0 },
        },
        BioRegenRate(5.0),
    )).id();

    world.insert_resource(ColonyResources::default()); // Has resources

    // Act: Run regen system
    let mut schedule = Schedule::new();
    schedule.add_systems(bio_regeneration_system);
    schedule.run(&mut world);

    // Assert: HP did NOT increase
    let structure = world.get::<Structure>(bio_entity).unwrap();
    assert_eq!(structure.hp, 50.0);
}

#[test]
fn test_bio_structure_infection_hostility() {
    // Arrange: An Infected BioStructure and a Pop nearby
    let mut world = World::new();
    let bio_entity = world.spawn((
        Structure { max_hp: 100.0, hp: 100.0 },
        BioStructure {
            state: BioState::Infected, // INFECTED
            hunger: 0.0,
            upkeep_cost: BioUpkeep::default(),
        },
        GridPosition { x: 5, y: 5 },
    )).id();

    let pop_entity = world.spawn((
        Pop,
        GridPosition { x: 5, y: 5 }, // Inside the building
        Health { current: 100.0, max: 100.0 },
    )).id();

    // Act: Run infection logic
    let mut schedule = Schedule::new();
    schedule.add_systems(bio_infection_system);
    schedule.run(&mut world);

    // Assert: Pop took damage
    let health = world.get::<Health>(pop_entity).unwrap();
    assert!(health.current < 100.0);
}
```

## 4. GREEN Phase: Minimal Implementation

### Components

```rust
#[derive(Component, Debug, Clone, PartialEq)]
pub enum BioState {
    Healthy,
    Starving,
    Sick,
    Infected,
    Dead,
}

#[derive(Debug, Clone, Default)]
pub struct BioUpkeep {
    pub food: f32,
    pub water: f32,
}

#[derive(Component, Debug, Clone)]
pub struct BioStructure {
    pub state: BioState,
    pub hunger: f32, // Accumulates over time (e.g. +1 per tick)
    pub hunger_threshold: f32, // At this value, attempts to consume resources
    pub upkeep_cost: BioUpkeep,
}

#[derive(Component, Debug, Clone)]
pub struct BioRegenRate(pub f32); // HP per tick
```

### Systems

1.  **`bio_upkeep_system`**:
    *   Increments `BioStructure.hunger`.
    *   If `hunger >= hunger_threshold`:
        *   Try to deduct `upkeep_cost` from `ColonyResources`.
        *   If success: Reset `hunger` to 0. State -> `Healthy` (if was Starving).
        *   If failure: Apply "Starvation Damage" to `Structure.hp`. State -> `Starving`.
2.  **`bio_regeneration_system`**:
    *   Query `(BioStructure, BioRegenRate, Structure)`.
    *   If `state == Healthy` AND `Structure.hp < Structure.max_hp`:
        *   `Structure.hp += BioRegenRate`.
3.  **`bio_infection_system`**:
    *   Query `(BioStructure, GridPosition)` where `state == Infected`.
    *   Query Pops at same position.
    *   Apply damage to Pops.

## 5. REFACTOR Phase

*   **Integration with Maintenance Debt**: Bio-Structures should explicitly *exclude* `MaintenanceDebt` component or have a system that sets their debt to 0, preventing double jeopardy.
*   **Visual Feedback**: Sick/Starving buildings should change color/texture.
*   **Construction**: Bio-Structures might start as a "Seed" and grow (HP increases from 1 to Max) rather than being built by workers hammering. This could reuse `MiningProgress` logic but reversed (`GrowthProgress`?).
*   **Balancing**: Upkeep cost must be lower than manual repair cost in labor, but higher in resource (Food/Water).
*   **Events**: "Infection" could be triggered by `Event` (e.g. "Space Plague") or random tick if `Sick` for too long.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] Bio-Structures regenerate HP when healthy.
- [ ] Bio-Structures consume Food/Water from colony resources.
- [ ] Starvation damages the structure.
- [ ] Sick/Infected states function as expected.
- [ ] `cargo test` passes.
- [ ] `cargo clippy` passes.

## 7. Technical Guidance

*   Use `crate::layer1::resources::ColonyResources` for upkeep.
*   Reuse `Structure` component for HP.
*   Ensure `bio_upkeep_system` runs less frequently than every tick (maybe once per "hour" or "day") to avoid spamming resource deduction, or accumulate fractional hunger.
*   Infection damage should use `crate::layer1::combat::Attack` logic or direct health modification if simple environment hazard.

## 8. Questions

- **Q:** Should Bio-Structures produce resources (e.g. Oxygen, specialized items)?
- *Architect:* Not in this spec. Keep it to the "Building Type" mechanic first. Specific bio-buildings (e.g. "Living Quarters") can be added later.
- **Q:** How do we build them?
- *Architect:* Standard construction for now, but costing `Biomass` (or Wood/Food). Future spec can handle "Growth" construction phase.
