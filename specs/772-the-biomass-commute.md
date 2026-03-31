# 772: The Biomass Commute

## 1. Overview
**Layer:** 1
**Fantasy:** Building your infrastructure out of living, breathing tissue that requires feeding, but offers incredible efficiency.
**Mechanic:** A late-game biological alternative to conveyor belts or mass transit. "Vein Tubes" transport resources and pops instantly across the map. However, the network itself has a Hunger need. If it isn't fed excess organic matter, it begins digesting whatever is inside it—including raw resources, finished goods, or commuting Pops.

## 2. Dependencies
- Core Layer 1 ECS (Entities, Components, Systems)
- Need/Resource system
- Pop and Item entities

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_biomass_network_consumes_upkeep() {
        let mut app = App::new();
        app.add_systems(Update, process_biomass_network_hunger);

        // Add a biomass network entity with a hunger tracker
        let network_entity = app.world_mut().spawn((
            BiomassNetwork {
                hunger: 50.0,
                max_hunger: 100.0,
                consumption_rate: 10.0,
            },
        )).id();

        app.update();

        let network = app.world().get::<BiomassNetwork>(network_entity).unwrap();
        assert_eq!(network.hunger, 60.0); // Hunger increased by consumption rate
    }

    #[test]
    fn test_starving_network_digests_contents() {
        let mut app = App::new();
        app.add_systems(Update, (process_biomass_network_hunger, digest_transit_contents));

        let network_entity = app.world_mut().spawn((
            BiomassNetwork {
                hunger: 100.0, // Fully starving
                max_hunger: 100.0,
                consumption_rate: 10.0,
            },
        )).id();

        // Add an item in transit
        let item_entity = app.world_mut().spawn((
            InTransit { network: network_entity },
            ResourceYield { amount: 20.0, resource_type: ResourceType::Organic },
        )).id();

        app.update();

        // The item should be consumed/digested, meaning it despawns or changes state
        assert!(app.world().get_entity(item_entity).is_none(), "Item should be digested and despawned");

        // Network hunger should decrease due to digestion
        let network = app.world().get::<BiomassNetwork>(network_entity).unwrap();
        assert!(network.hunger < 100.0, "Hunger should be reduced after digesting contents");
    }

    #[test]
    fn test_fed_network_safely_transports() {
        let mut app = App::new();
        app.add_systems(Update, (process_biomass_network_hunger, digest_transit_contents));

        let network_entity = app.world_mut().spawn((
            BiomassNetwork {
                hunger: 0.0, // Sated
                max_hunger: 100.0,
                consumption_rate: 10.0,
            },
        )).id();

        // Add a pop in transit
        let pop_entity = app.world_mut().spawn((
            InTransit { network: network_entity },
            Pop { name: "Commuter".to_string() },
        )).id();

        app.update();

        // The pop should survive because the network is not starving
        assert!(app.world().get_entity(pop_entity).is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct BiomassNetwork {
    pub hunger: f32,
    pub max_hunger: f32,
    pub consumption_rate: f32,
}

#[derive(Component)]
pub struct InTransit {
    pub network: Entity,
}

#[derive(Component)]
pub struct ResourceYield {
    pub amount: f32,
    pub resource_type: ResourceType,
}

#[derive(PartialEq)]
pub enum ResourceType {
    Organic,
    Inorganic,
}

#[derive(Component)]
pub struct Pop {
    pub name: String,
}

pub fn process_biomass_network_hunger(
    mut query: Query<&mut BiomassNetwork>,
) {
    for mut network in query.iter_mut() {
        network.hunger = (network.hunger + network.consumption_rate).min(network.max_hunger);
    }
}

pub fn digest_transit_contents(
    mut commands: Commands,
    mut networks: Query<&mut BiomassNetwork>,
    transit_query: Query<(Entity, &InTransit)>,
) {
    for (entity, transit) in transit_query.iter() {
        if let Ok(mut network) = networks.get_mut(transit.network) {
            if network.hunger >= network.max_hunger {
                // Digest!
                commands.entity(entity).despawn();
                network.hunger = (network.hunger - 20.0).max(0.0);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**: The digestion logic could be expanded into different systems handling pops versus resources differently (e.g. popping a `PopDigestedEvent`).
- **Performance**: Instead of iterating all items in transit every tick, items could trigger a check only when first entering the network, or the network can maintain a list of internal entities.
- **API Improvements**: Use events for digestion so that UI and logging (or morale systems) can react to a commuter being eaten.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified: Hunger increases, starving networks eat contents, fed networks do not.

## 7. Technical Guidance
- Ensure that the biomass network ties into the overall logistics system so that normal items are routed through it.
- Keep the `BiomassNetwork` component lightweight.
- Consider emitting an event when a Pop is digested to handle morale penalties.

## 8. Questions
*Builder: add questions here if spec is unclear.*
