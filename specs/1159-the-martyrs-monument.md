# 1159: The Martyr's Monument

## Overview

If a highly populated Layer 1 colony is completely wiped out by a dramatic event (e.g., orbital bombardment, catastrophic plague, exploding reactor), the ruined planet becomes a "Martyr Site." Pops across the galaxy (Layer 3) who shared the colony's ideology or culture develop a massive, unyielding desire to undertake a pilgrimage to the ruins. This forces thousands of citizens from within rival empires to migrate en masse, bringing resources, sparking religious riots against their own government, and effectively handing the system back to the player as a heavily fortified cult stronghold.

## Dependencies

- None (Base Layer 1 destruction and Layer 3 migration mechanics assumed)

## RED Phase: Tests First

```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_colony_destruction_creates_martyr_site() {
    // Arrange: Setup a highly populated Layer 1 colony with a specific ideology
    // Act: Trigger a catastrophic destruction event
    // Assert: The planet entity gains a `MartyrSite` component with the colony's ideology
}

#[test]
fn test_martyr_site_triggers_pilgrimage_desire() {
    // Arrange: A `MartyrSite` exists, and a rival empire has Pops sharing the ideology
    // Act: Advance simulation
    // Assert: The rival empire's Pops gain a `PilgrimageDesire` component targeting the site
}

#[test]
fn test_pilgrims_migrate_and_fortify_site() {
    // Arrange: Rival Pops have `PilgrimageDesire`
    // Act: Advance simulation enough for migration
    // Assert: Pops move to the `MartyrSite`, and the site's defense/fortification level increases
}

#[test]
fn test_pilgrimage_causes_unrest_in_origin_empire() {
    // Arrange: Rival empire is losing Pops to pilgrimage
    // Act: Advance simulation
    // Assert: The rival empire experiences a spike in `Unrest` due to religious riots
}
```

## GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN

pub fn create_martyr_site_system(
    mut commands: Commands,
    colonies: Query<(Entity, &ColonyIdeology, &Population), With<Destroyed>>,
) {
    for (entity, ideology, pop) in colonies.iter() {
        if pop.count > 10_000 { // Highly populated threshold
            commands.entity(entity).insert(MartyrSite {
                ideology: ideology.clone(),
                defense_level: 0.0,
            });
        }
    }
}

pub fn trigger_pilgrimage_system(
    mut commands: Commands,
    martyr_sites: Query<&MartyrSite>,
    mut rival_pops: Query<(Entity, &PopIdeology), Without<PilgrimageDesire>>,
) {
    for site in martyr_sites.iter() {
        for (pop_entity, pop_ideology) in rival_pops.iter_mut() {
            if pop_ideology == &site.ideology {
                commands.entity(pop_entity).insert(PilgrimageDesire);
            }
        }
    }
}

pub fn execute_pilgrimage_system(
    mut commands: Commands,
    mut pilgrims: Query<(Entity, &mut Location), With<PilgrimageDesire>>,
    mut martyr_sites: Query<(Entity, &mut MartyrSite)>,
    mut empires: Query<&mut Unrest>,
) {
    // Simplified: move pilgrims instantly for the test
    for (site_entity, mut site) in martyr_sites.iter_mut() {
        for (pilgrim_entity, mut location) in pilgrims.iter_mut() {
            location.target = site_entity;
            site.defense_level += 1.0; // Fortify
            commands.entity(pilgrim_entity).remove::<PilgrimageDesire>();

            // Cause unrest in origin
            for mut unrest in empires.iter_mut() {
                unrest.level += 5.0;
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization:** The pilgrimage desire check should not iterate over every single pop in the galaxy every tick. Use event-driven triggers when a `MartyrSite` is created to assign the desire, or batch process it.
- **Code Smell:** The `execute_pilgrimage_system` instantly teleports pops. It should hook into the actual Layer 3 migration/travel systems so the pilgrimage takes time and uses ships.
- **Integration:** Generate a Chronicle event when a world falls and becomes a Martyr Site to alert the player.
- **Gameplay Balance:** Ensure there is a way for rival empires to attempt to block or suppress the pilgrimage, providing counter-play.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code in `src/layer3/diplomacy/martyr_site.rs`
- [ ] Chronicle templates are added for both the site creation and the resulting riots.

## Technical Guidance

### Components

```rust
#[derive(Component)]
pub struct MartyrSite {
    pub ideology: IdeologyType,
    pub defense_level: f32,
}

#[derive(Component)]
pub struct PilgrimageDesire;
```

### Integration Points

- **Chronicle:** Use templates from `lore/TEMPLATES.md` to record the tragedy and subsequent holy migration.
- **Migration System:** Hook the `PilgrimageDesire` into the existing logic that handles pop movement between planets on Layer 3.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
