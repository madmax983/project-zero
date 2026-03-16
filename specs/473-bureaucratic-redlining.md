# Bureaucratic Redlining (Spec 473)

## Overview
This feature allows the player to "Dezone" struggling sectors of the Layer 1 colony to remove them from the power/water grid, instantly saving massive upkeep costs. However, the Pops living there are not evicted; they become "Stateless," stop paying taxes, and form an autonomous, hostile squatter faction that slowly expands its territory.

## Dependencies
- `056` Designated Zones (Implemented)
- `042` Energy System (Implemented)
- `068` Pop Factions (Implemented)

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use super::*;

    #[test]
    fn test_dezoning_removes_upkeep_and_creates_stateless_faction() {
        let mut world = World::new();

        // Arrange
        let zone = world.spawn((
            Zone { id: 1, is_redlined: false },
            PowerGridNode { upkeep: 50.0 },
        )).id();

        let pop = world.spawn((
            PopBundle::default(),
            HomeZone { zone_id: 1 },
            FactionMember { faction_id: Faction::Colony },
        )).id();

        // Act
        // Player triggers a redlining action on zone 1
        world.resource_mut::<Events<RedlineZoneEvent>>().send(RedlineZoneEvent { zone: zone });
        world.run_system_once(execute_redlining_system).unwrap();

        // Assert
        let zone_data = world.get::<Zone>(zone).unwrap();
        assert!(zone_data.is_redlined);

        let power_node = world.get::<PowerGridNode>(zone);
        assert!(power_node.is_none()); // Removed from the grid

        let pop_faction = world.get::<FactionMember>(pop).unwrap();
        assert_eq!(pop_faction.faction_id, Faction::Stateless); // Pop is now stateless
    }

    #[test]
    fn test_stateless_faction_spreads_to_adjacent_zones() {
        let mut world = World::new();

        // Arrange
        let redlined_zone = world.spawn((
            Zone { id: 1, is_redlined: true },
            GridPosition { x: 10, y: 10 }, // Assuming Zones have or map to positions
        )).id();

        let adjacent_zone = world.spawn((
            Zone { id: 2, is_redlined: false },
            GridPosition { x: 10, y: 11 },
        )).id();

        world.insert_resource(Time::new()); // Mock time
        world.insert_resource(ColonySecurity { level: 0.1 }); // Low security allows faster spread

        // Act
        world.run_system_once(stateless_expansion_system).unwrap();

        // Assert
        let adj_zone_data = world.get::<Zone>(adjacent_zone).unwrap();
        assert!(adj_zone_data.is_redlined); // The squatter faction expanded
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Event)]
pub struct RedlineZoneEvent {
    pub zone: Entity,
}

#[derive(Component)]
pub struct Stateless;

pub fn execute_redlining_system(
    mut commands: Commands,
    mut events: EventReader<RedlineZoneEvent>,
    mut zones: Query<&mut Zone>,
    mut pops: Query<(Entity, &HomeZone, &mut FactionMember)>,
) {
    for event in events.read() {
        if let Ok(mut zone) = zones.get_mut(event.zone) {
            zone.is_redlined = true;

            // Disconnect from grid (remove grid components)
            commands.entity(event.zone).remove::<PowerGridNode>();
            // Also remove water, etc. if they exist

            // Convert residents to Stateless faction
            for (pop_entity, home, mut faction) in pops.iter_mut() {
                if home.zone_id == zone.id {
                    faction.faction_id = Faction::Stateless;
                    commands.entity(pop_entity).insert(Stateless);
                }
            }
        }
    }
}

pub fn stateless_expansion_system(
    mut commands: Commands,
    security: Res<ColonySecurity>,
    redlined_zones: Query<(&Zone, &GridPosition)>,
    mut normal_zones: Query<(Entity, &mut Zone, &GridPosition), Without<Stateless>>, // Assuming Stateless flag applies to zones too
) {
    let mut rng = rand::thread_rng();
    let spread_chance = 0.01 * (1.0 - security.level); // Base 1% chance, modified by low security

    for (_, r_pos) in redlined_zones.iter().filter(|(z, _)| z.is_redlined) {
        for (n_entity, mut n_zone, n_pos) in normal_zones.iter_mut().filter(|(_, z, _)| !z.is_redlined) {
            if r_pos.distance_manhattan(*n_pos) <= 1 && rng.gen::<f32>() < spread_chance {
                n_zone.is_redlined = true;
                commands.entity(n_entity).remove::<PowerGridNode>();
                // In reality, converting the pops here too would be needed
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design
- **Squatter Raids**: The `Stateless` faction should occasionally spawn small raiding parties targeting adjacent powered zones for batteries or food, hooking into the combat system.
- **Taxation Hook**: Ensure the economy tick explicitly ignores Pops in the `Stateless` faction so the player loses the tax revenue.
- **Visuals**: Redlined zones should visually decay (e.g., using darker UI colors or a "Squalor" overlay).

## Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Triggering Redlining instantly removes power upkeep from a zone.
- [ ] Pops in the redlined zone are moved to the `Stateless` faction and stop contributing taxes.
- [ ] The `Stateless` area has a probabilistic chance to slowly spread to adjacent vulnerable zones.

## Technical Guidance
- Zones in SCALE are typically logical groupings of tiles. Make sure the grid disconnection correctly targets the `PowerGridNode` entities located within that zone's logical bounds.
- Use `Faction::Stateless` (or add it if it doesn't exist) to integrate seamlessly with the existing `068 Pop Factions` and combat targeting logic.

## Questions
*Builder: add questions here if spec is unclear.*
