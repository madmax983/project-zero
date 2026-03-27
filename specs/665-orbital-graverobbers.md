# Orbital Graverobbers (Spec 665)

## 1. Overview
The vultures of the void don't care if you're still using the ship. "Scavenger" fleets (Layer 2) are attracted to high-conflict systems. They don't attack combatants; they wait until a ship is disabled and immediately swoop in to strip the hull, ignoring the fact that the crew might still be alive inside. They will occasionally try to scavenge damaged, but active, orbital infrastructure.

## 2. Dependencies
- `src/layer2/fleets.rs` (Ship and fleet entities, combat states)
- `src/layer2/combat.rs` (Ship damage and destruction events)
- `src/layer2/logistics.rs` (Resource extraction from ships)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_scavenger_fleet_spawning() {
        let mut app = App::new();
        app.add_systems(Update, spawn_scavengers_on_conflict_system);

        let mut system_data = app.world_mut().spawn(SystemConflictLevel { level: 0.0 }).id();

        // Act: Combat happens, raising conflict level
        app.world_mut().send_event(ShipDestroyedEvent { system_id: system_data });
        app.update();

        // Assert: Conflict level increases, eventually spawning a Scavenger fleet
        let conflict = app.world().get::<SystemConflictLevel>(system_data).unwrap();
        assert!(conflict.level > 0.0);

        // Assuming the threshold is met for this test
        app.world_mut().send_event(ShipDestroyedEvent { system_id: system_data });
        app.world_mut().send_event(ShipDestroyedEvent { system_id: system_data });
        app.update();

        let scavenger_query = app.world().query_filtered::<Entity, With<ScavengerFleet>>();
        assert!(scavenger_query.iter(app.world()).count() > 0);
    }

    #[test]
    fn test_scavenging_disabled_ship() {
        let mut app = App::new();
        app.add_systems(Update, scavenge_disabled_ship_system);

        let disabled_ship = app.world_mut().spawn((
            Ship { is_disabled: true, resources: 100 },
            Position { x: 10, y: 10 },
        )).id();

        let scavenger = app.world_mut().spawn((
            ScavengerFleet { capacity: 50 },
            Position { x: 10, y: 10 }, // Arrived at target
        )).id();

        // Act: Scavenger extracts resources from the disabled ship
        app.world_mut().send_event(ScavengeEvent {
            scavenger,
            target: disabled_ship,
        });
        app.update();

        // Assert: Disabled ship loses resources, Scavenger gains them
        let ship = app.world().get::<Ship>(disabled_ship).unwrap();
        assert_eq!(ship.resources, 50); // Scavenged half
    }

    #[test]
    fn test_scavenger_hostility() {
        let mut app = App::new();
        app.add_systems(Update, resolve_scavenger_conflict_system);

        let disabled_ship = app.world_mut().spawn((
            Ship { is_disabled: true, resources: 100 },
        )).id();

        let scavenger = app.world_mut().spawn(ScavengerFleet { capacity: 50 }).id();

        // Player attempts to defend their disabled ship
        app.world_mut().send_event(PlayerAttackScavengerEvent {
            scavenger,
        });
        app.update();

        // Scavengers become hostile
        let hostile = app.world().get::<HostileEntity>(scavenger);
        assert!(hostile.is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct SystemConflictLevel {
    pub level: f32,
}

#[derive(Event)]
pub struct ShipDestroyedEvent {
    pub system_id: Entity,
}

#[derive(Component, Default)]
pub struct ScavengerFleet {
    pub capacity: u32,
}

#[derive(Component, Default)]
pub struct Ship {
    pub is_disabled: bool,
    pub resources: u32,
}

#[derive(Component, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Event)]
pub struct ScavengeEvent {
    pub scavenger: Entity,
    pub target: Entity,
}

#[derive(Event)]
pub struct PlayerAttackScavengerEvent {
    pub scavenger: Entity,
}

#[derive(Component)]
pub struct HostileEntity;

pub fn spawn_scavengers_on_conflict_system(
    mut events: EventReader<ShipDestroyedEvent>,
    mut query: Query<&mut SystemConflictLevel>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok(mut conflict) = query.get_mut(event.system_id) {
            conflict.level += 1.0;
            if conflict.level >= 3.0 {
                commands.spawn(ScavengerFleet { capacity: 100 });
                conflict.level = 0.0; // Reset after spawning
            }
        }
    }
}

pub fn scavenge_disabled_ship_system(
    mut events: EventReader<ScavengeEvent>,
    mut query: Query<&mut Ship>,
) {
    for event in events.read() {
        if let Ok(mut ship) = query.get_mut(event.target) {
            if ship.is_disabled && ship.resources > 0 {
                let amount = ship.resources.min(50); // Hardcoded extraction rate for now
                ship.resources -= amount;
            }
        }
    }
}

pub fn resolve_scavenger_conflict_system(
    mut events: EventReader<PlayerAttackScavengerEvent>,
    mut commands: Commands,
) {
    for event in events.read() {
        commands.entity(event.scavenger).insert(HostileEntity);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding & Targeting:** Scavengers need AI to seek out disabled ships and active but damaged infrastructure. This logic is missing and needs proper integration with Layer 2 pathfinding.
- **Resource Transfer:** Currently, resources are simply deleted from the target ship. They should be added to the Scavenger fleet's inventory or immediately converted into a "salvage" resource item.
- **Hostility Logic:** Becoming hostile shouldn't just be a tag; it needs to integrate with the combat system so the Scavenger fleet actually fights back or attempts to flee when attacked.
- **Crew Survival:** The spec mentions ignoring live crew. There should be a chance for crew to die or be "extracted" (kidnapped/lost) when a ship is scavenged.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] High conflict in a system triggers Scavenger fleet spawns.
- [ ] Scavengers extract resources from disabled ships.
- [ ] Scavengers become hostile when attacked by the player.
- [ ] Correctly integrates with Layer 2 movement and resource systems.

## 7. Technical Guidance
- **Layer 2 AI:** The Scavenger fleets are a new type of non-player entity in Layer 2. Ensure their AI behavior (seeking scrap, avoiding healthy warships unless attacked) is robust and doesn't interfere with main faction logic.
- **Combat Events:** Hook into the existing `ShipDamageEvent` or `ShipDisabledEvent` (if they exist) rather than relying solely on `ShipDestroyedEvent` to attract Scavengers, as they want the crippled ships, not just the dust.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
