# The Exiled Architects (Spec 795)

## 1. Overview
When Pops are banished or rebel, they do not necessarily despawn immediately. Instead, they can form hidden "Exile Camps" in the fog of war (Layer 1). These camps can scavenge resources and construct crude but dangerous guerrilla infrastructure (e.g., siphon pipelines, scrap catapults) that damage or drain resources from the primary colony.

This creates tension: does the player spend military resources clearing out the exiles, or endure their parasitic sabotage?

## 2. Dependencies
- `src/layer1/population/mod.rs` (Pop components)
- `src/layer1/faction.rs` (Factions and banishment mechanics)
- `src/layer1/map.rs` (TerrainGrid and Fog of War)
- `src/layer1/building.rs` (Infrastructure)

## 3. RED Phase: Tests First

```rust
// tests/layer1/test_exiled_architects.rs

use bevy::prelude::*;
use scale::layer1::population::{Pop, TraitExiled};
use scale::layer1::faction::{BanishEvent, ExileCamp};
use scale::layer1::map::{TerrainGrid, FogOfWar};
use scale::layer1::building::{Building, ResourceSiphon};
use scale::layer1::resources::ColonyResources;

#[test]
fn test_exile_camp_formation_on_banish() {
    let mut app = App::new();
    // ... setup plugins and grid ...

    // Arrange: A pop is banished
    let pop = app.world_mut().spawn((Pop::new("Traitor"),)).id();
    app.world_mut().resource_mut::<Events<BanishEvent>>().send(BanishEvent { entity: pop });

    app.update();

    // Assert: The pop now has TraitExiled, and an ExileCamp entity is spawned in the fog of war
    assert!(app.world().entity(pop).contains::<TraitExiled>());

    let camp_query = app.world_mut().query::<&ExileCamp>().iter(app.world()).count();
    assert_eq!(camp_query, 1);
}

#[test]
fn test_exile_camp_constructs_siphon() {
    let mut app = App::new();
    // ... setup ...

    // Arrange: An exile camp exists near the main colony
    let camp = app.world_mut().spawn(ExileCamp { resources: 100 }).id();

    // Act: Advance simulation enough for the camp to build a siphon
    for _ in 0..100 {
        app.update();
    }

    // Assert: A ResourceSiphon building exists
    let siphon_query = app.world_mut().query::<&ResourceSiphon>().iter(app.world()).count();
    assert!(siphon_query > 0);
}

#[test]
fn test_siphon_drains_resources() {
    let mut app = App::new();
    // ... setup ...

    app.world_mut().insert_resource(ColonyResources { energy: 1000, ..default() });
    app.world_mut().spawn(ResourceSiphon { drain_rate: 10 });

    // Act
    app.update(); // Tick the siphon system

    // Assert
    let resources = app.world().resource::<ColonyResources>();
    assert_eq!(resources.energy, 990);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/exile.rs

use bevy::prelude::*;
use crate::layer1::population::Pop;
use crate::layer1::faction::BanishEvent;
use crate::layer1::map::FogOfWar;
use crate::layer1::resources::ColonyResources;

#[derive(Component)]
pub struct TraitExiled;

#[derive(Component)]
pub struct ExileCamp {
    pub resources: u32,
}

#[derive(Component)]
pub struct ResourceSiphon {
    pub drain_rate: u32,
}

pub fn handle_banishment_system(
    mut commands: Commands,
    mut banish_events: EventReader<BanishEvent>,
    fog_of_war: Res<FogOfWar>,
) {
    for event in banish_events.read() {
        commands.entity(event.entity).insert(TraitExiled);

        // Find a spot in the fog of war
        if let Some(pos) = fog_of_war.find_hidden_tile() {
            commands.spawn((
                ExileCamp { resources: 50 },
                Transform::from_translation(pos.extend(0.0)),
            ));
        }
    }
}

pub fn exile_camp_construction_system(
    mut commands: Commands,
    mut camps: Query<(Entity, &mut ExileCamp, &Transform)>,
) {
    for (entity, mut camp, transform) in camps.iter_mut() {
        if camp.resources >= 100 {
            camp.resources -= 100;
            commands.spawn((
                ResourceSiphon { drain_rate: 10 },
                Transform::from_translation(transform.translation), // Build nearby
            ));
        } else {
            camp.resources += 1; // Passive scavenging
        }
    }
}

pub fn siphon_drain_system(
    siphons: Query<&ResourceSiphon>,
    mut resources: ResMut<ColonyResources>,
) {
    for siphon in siphons.iter() {
        if resources.energy >= siphon.drain_rate {
            resources.energy -= siphon.drain_rate;
        } else {
            resources.energy = 0;
        }
    }
}

pub struct ExilePlugin;
impl Plugin for ExilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            handle_banishment_system,
            exile_camp_construction_system,
            siphon_drain_system,
        ));
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**:
    - The `exile_camp_construction_system` should probably use a proper Utility AI or action queue instead of just a raw resource counter.
    - Siphons should target specific colony infrastructure (e.g., finding the nearest power conduit) rather than just draining global resources directly.
    - Add a system for the player's military to discover and destroy `ExileCamp` and `ResourceSiphon` entities.
- **Code Smells**: Hardcoded values (100 resources to build, 10 drain rate). These should be configurable via resources or constants.
- **Performance**: Ensure `find_hidden_tile` isn't too expensive if many pops are banished at once.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Banished pops form exile camps in the fog of war
- [ ] Exile camps build guerrilla infrastructure (like siphons)
- [ ] Siphons drain resources from the colony

## 7. Technical Guidance
- **Integration Points**: Register the `ExilePlugin`. Ensure `BanishEvent` is emitted correctly when the player banishes a pop (or a faction secedes).
- **Gotchas**: Make sure `ExileCamp` and `ResourceSiphon` are despawned if the fog of war is cleared and military units attack them.

## 8. Questions
*Builder: add questions here if spec is unclear.*
