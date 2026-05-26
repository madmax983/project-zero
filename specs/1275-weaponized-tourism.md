# 1275: Weaponized Tourism

## 1. Overview
**Layer:** Cross-layer (Layer 1 -> 3)
**Fantasy:** Killing them with kindness.
**Mechanic:** You send your own "Tourist" pops to rival colonies. They pay well but are programmed to be "Difficult" (complain, break things, spread contrary Ethics). If the rival harms them, you get a Casus Belli.
**Emergence:** You send a wave of "Food Critics" to a starving enemy colony. They eat the reserve rations and complain about the texture, causing a riot that topples the enemy government.
**Tension:** Soft Power (culture victory) vs. Hostage risk (your pops are there).

## 2. Dependencies
- Layer 1 `Pop` mechanics and `Needs` (especially Food/Morale).
- Layer 3 Diplomacy/Casus Belli systems.
- Inter-colony travel (Freighters/Passenger Ships).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_tourist_drains_resources_and_spreads_unrest() {
        let mut app = App::new();
        app.add_systems(Update, weaponized_tourist_behavior_system);

        let colony = app.world_mut().spawn(ColonyResources {
            food: 100,
            morale: 50,
        }).id();

        let tourist = app.world_mut().spawn((
            Pop,
            WeaponizedTourist {
                origin_faction: Entity::PLACEHOLDER,
                difficulty_level: 5,
            },
            LocatedAt { location: colony },
        )).id();

        app.update();

        let resources = app.world().get::<ColonyResources>(colony).unwrap();
        // Tourist consumed food aggressively
        assert!(resources.food < 100);
        // Tourist complained, lowering morale
        assert!(resources.morale < 50);
    }

    #[test]
    fn test_tourist_death_grants_casus_belli() {
        let mut app = App::new();
        app.add_systems(Update, tourist_death_casus_belli_system);
        app.add_event::<PopDeathEvent>();

        let origin_faction = app.world_mut().spawn(Faction { casus_belli: vec![] }).id();
        let host_colony = app.world_mut().spawn(Colony { faction: Entity::PLACEHOLDER }).id();

        let tourist = app.world_mut().spawn((
            Pop,
            WeaponizedTourist {
                origin_faction,
                difficulty_level: 5,
            },
            LocatedAt { location: host_colony },
        )).id();

        app.world_mut().send_event(PopDeathEvent {
            pop: tourist,
            location: host_colony,
        });

        app.update();

        let origin = app.world().get::<Faction>(origin_faction).unwrap();
        assert!(origin.casus_belli.len() > 0, "Death of tourist must grant a casus belli");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct LocatedAt {
    pub location: Entity,
}

#[derive(Component)]
pub struct WeaponizedTourist {
    pub origin_faction: Entity,
    pub difficulty_level: i32,
}

#[derive(Component)]
pub struct ColonyResources {
    pub food: i32,
    pub morale: i32,
}

#[derive(Component)]
pub struct Colony {
    pub faction: Entity,
}

#[derive(Component, Default)]
pub struct Faction {
    pub casus_belli: Vec<Entity>, // list of target factions
}

#[derive(Event)]
pub struct PopDeathEvent {
    pub pop: Entity,
    pub location: Entity,
}

pub fn weaponized_tourist_behavior_system(
    tourist_query: Query<(&WeaponizedTourist, &LocatedAt)>,
    mut colony_query: Query<&mut ColonyResources>,
) {
    for (tourist, located_at) in tourist_query.iter() {
        if let Ok(mut resources) = colony_query.get_mut(located_at.location) {
            // Difficult tourists consume double food and lower morale
            resources.food -= 2;
            resources.morale -= tourist.difficulty_level;
        }
    }
}

pub fn tourist_death_casus_belli_system(
    mut events: EventReader<PopDeathEvent>,
    tourist_query: Query<&WeaponizedTourist>,
    colony_query: Query<&Colony>,
    mut faction_query: Query<&mut Faction>,
) {
    for event in events.read() {
        if let Ok(tourist) = tourist_query.get(event.pop) {
            if let Ok(colony) = colony_query.get(event.location) {
                // If we find the origin faction, add casus belli against the colony's faction
                if let Ok(mut origin_faction) = faction_query.get_mut(tourist.origin_faction) {
                    origin_faction.casus_belli.push(colony.faction);
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Action System:** Instead of direct subtractions in a global system, tourists should use the `PopDecider` to schedule "Complain" or "Gorge on Food" actions, hooking into standard AI behavior.
- **Economic Benefit:** Tourists should also deposit credits into the host colony's economy to simulate the "they pay well" aspect, creating the tension for the host player.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Tourists actively drain resources and morale from their location.
- [ ] Tourist death yields a casus belli for their origin faction.

## 7. Technical Guidance
- The `PopDeathEvent` must trigger before the entity is fully despawned so we can read its `WeaponizedTourist` component, or the component data must be cloned into the event itself.

## 8. Questions
*Builder: add questions here if spec is unclear.*
