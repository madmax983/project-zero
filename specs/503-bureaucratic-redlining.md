# 503 - Bureaucratic Redlining

## 1. Overview
The cold calculus of abandoning the poor to balance the planetary budget. Players can "Dezone" struggling sectors of their Layer 1 colony to remove them from the power/water grid, instantly saving massive upkeep costs. However, the Pops living there are not evicted; they become "Stateless". They stop paying taxes and start forming an autonomous, hostile squatter faction that slowly expands its territory, raiding your infrastructure for resources.

## 2. Dependencies
- `056` Designated Zones
- `042` Energy System
- `068` Pop Factions

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;
    // Mock imports for existing modules
    // use crate::layer1::zones::{Zone, ZoneStatus};
    // use crate::layer1::pop::{Pop, Faction};
    // use crate::layer1::energy::GridNode;

    #[derive(Component)]
    struct Zone {
        pub id: u32,
        pub is_dezoned: bool,
    }

    #[derive(Component)]
    struct GridNode {
        pub active: bool,
        pub zone_id: u32,
    }

    #[derive(Component)]
    struct Pop {
        pub zone_id: u32,
    }

    #[derive(Component)]
    struct Faction {
        pub name: String,
    }

    fn dezone_system(
        mut commands: Commands,
        mut zones: Query<&mut Zone>,
        mut nodes: Query<&mut GridNode>,
        mut pops: Query<(Entity, &Pop, Option<&mut Faction>)>,
    ) {
        // Implementation omitted for RED phase. Should fail tests.
    }

    #[test]
    fn test_dezoning_disables_grid_and_creates_stateless_faction() {
        let mut app = App::new();

        let zone_entity = app.world_mut().spawn(Zone { id: 1, is_dezoned: true }).id();
        let node_entity = app.world_mut().spawn(GridNode { active: true, zone_id: 1 }).id();
        let pop_entity = app.world_mut().spawn(Pop { zone_id: 1 }).id();

        app.add_systems(Update, dezone_system);
        app.update();

        let node = app.world().get::<GridNode>(node_entity).unwrap();
        assert!(!node.active, "Grid node should be disabled in dezoned zone");

        let faction = app.world().get::<Faction>(pop_entity);
        assert!(faction.is_some(), "Pop should have a faction assigned");
        assert_eq!(faction.unwrap().name, "Stateless", "Pop faction should be 'Stateless'");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Zone {
    pub id: u32,
    pub is_dezoned: bool,
}

#[derive(Component)]
pub struct GridNode {
    pub active: bool,
    pub zone_id: u32,
}

#[derive(Component)]
pub struct Pop {
    pub zone_id: u32,
}

#[derive(Component)]
pub struct Faction {
    pub name: String,
}

pub fn dezone_system(
    mut commands: Commands,
    mut zones: Query<&mut Zone>,
    mut nodes: Query<&mut GridNode>,
    mut pops: Query<(Entity, &Pop, Option<&mut Faction>)>,
) {
    for mut zone in zones.iter_mut() {
        if zone.is_dezoned {
            for mut node in nodes.iter_mut() {
                if node.zone_id == zone.id {
                    node.active = false;
                }
            }

            for (entity, pop, faction_opt) in pops.iter_mut() {
                if pop.zone_id == zone.id {
                    if let Some(mut faction) = faction_opt {
                        faction.name = "Stateless".to_string();
                    } else {
                        commands.entity(entity).insert(Faction { name: "Stateless".to_string() });
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event-Driven Architecture**: Refactor `dezone_system` to react to a `DezoneEvent` rather than continuously checking all zones every frame.
- **Faction Handling**: Properly integrate with `068 Pop Factions` to handle the transition smoothly. Pops shouldn't just instantly change faction; perhaps it requires a few ticks or causes a massive drop in loyalty first.
- **Resource Savings**: Ensure that `042 Energy System` calculates the upkeep savings immediately when `node.active = false`.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Dezoning a zone disables its grid nodes.
- [ ] Pops living in a dezoned area are assigned to the "Stateless" faction.

## 7. Technical Guidance
- Integrate with the existing Zone UI so the player can toggle the "Dezone" status.
- Ensure that pops in the "Stateless" faction are ignored by tax collectors and standard Utility AI work assignments, and instead use bandit/scavenger AI.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
