# 1247: Hyperlane Turbulence

## 1. Overview
**Layer:** 2 -> 3
**Fantasy:** The highways of the galaxy are not calm rivers; they are raging rapids.
**Mechanic:** Heavy fleet traffic (from you or other empires) on a specific hyperlane destabilizes it, creating "Turbulence." Navigating a turbulent lane causes severe hull damage and scatters fleets, causing them to arrive piecemeal over months.
**Emergence:** You build your capital at a major hyperlane intersection. The massive trade traffic causes permanent turbulence, turning your capital into an unassailable fortress that accidentally destroys any fleet trying to visit or invade, effectively cutting you off from the galaxy.
**Tension:** Centralize logistics for efficiency vs. spreading out to avoid destroying the very infrastructure you rely on.

## 2. Dependencies
- Layer 2 / Layer 3 Hyperlane Network
- Fleet Movement
- Hull Damage System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_hyperlane_turbulence_increases_with_traffic() {
        let mut app = App::new();
        app.add_systems(Update, process_hyperlane_traffic);

        // Spawn a hyperlane
        let hyperlane_id = app.world.spawn(Hyperlane {
            node_a: Entity::from_raw(1),
            node_b: Entity::from_raw(2),
            turbulence: 0.0,
            traffic_count: 0,
        }).id();

        // Simulate traffic
        app.world.insert_resource(Events::<FleetJumpEvent>::default());
        let mut events = app.world.get_resource_mut::<Events<FleetJumpEvent>>().unwrap();
        events.send(FleetJumpEvent { hyperlane: hyperlane_id, fleet: Entity::from_raw(10) });
        events.send(FleetJumpEvent { hyperlane: hyperlane_id, fleet: Entity::from_raw(11) });

        app.update();

        // Turbulence should increase
        let lane = app.world.get::<Hyperlane>(hyperlane_id).unwrap();
        assert!(lane.turbulence > 0.0, "Turbulence must increase when fleets jump through the hyperlane");
        assert_eq!(lane.traffic_count, 2, "Traffic count must be updated");
    }

    #[test]
    fn test_turbulent_hyperlane_damages_fleets() {
        let mut app = App::new();
        app.add_systems(Update, apply_turbulence_damage);

        // Spawn a turbulent hyperlane
        let hyperlane_id = app.world.spawn(Hyperlane {
            node_a: Entity::from_raw(1),
            node_b: Entity::from_raw(2),
            turbulence: 100.0, // High turbulence
            traffic_count: 100,
        }).id();

        // Spawn a fleet jumping through
        let fleet_id = app.world.spawn((Fleet, Hull { health: 100.0 })).id();

        app.world.insert_resource(Events::<FleetJumpEvent>::default());
        let mut events = app.world.get_resource_mut::<Events<FleetJumpEvent>>().unwrap();
        events.send(FleetJumpEvent { hyperlane: hyperlane_id, fleet: fleet_id });

        app.update();

        // Fleet hull should be damaged
        let hull = app.world.get::<Hull>(fleet_id).unwrap();
        assert!(hull.health < 100.0, "Fleets jumping through a turbulent hyperlane must take hull damage");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Hyperlane {
    pub node_a: Entity,
    pub node_b: Entity,
    pub turbulence: f32,
    pub traffic_count: u32,
}

#[derive(Component)]
pub struct Fleet;

#[derive(Component)]
pub struct Hull {
    pub health: f32,
}

#[derive(Event)]
pub struct FleetJumpEvent {
    pub hyperlane: Entity,
    pub fleet: Entity,
}

pub fn process_hyperlane_traffic(
    mut jump_events: EventReader<FleetJumpEvent>,
    mut hyperlanes: Query<&mut Hyperlane>,
) {
    for event in jump_events.read() {
        if let Ok(mut lane) = hyperlanes.get_mut(event.hyperlane) {
            lane.traffic_count += 1;
            // Turbulence increases per jump
            lane.turbulence += 5.0;
        }
    }
}

pub fn apply_turbulence_damage(
    mut jump_events: EventReader<FleetJumpEvent>,
    hyperlanes: Query<&Hyperlane>,
    mut fleets: Query<&mut Hull, With<Fleet>>,
) {
    for event in jump_events.read() {
        if let Ok(lane) = hyperlanes.get(event.hyperlane) {
            if lane.turbulence > 50.0 {
                if let Ok(mut hull) = fleets.get_mut(event.fleet) {
                    // Apply damage based on turbulence intensity
                    let damage = (lane.turbulence - 50.0) * 0.1;
                    hull.health -= damage;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Decay:** Turbulence should slowly decay over time if a hyperlane is not used, requiring a `decay_turbulence` system hooked up to the `Time` resource.
- **Scattering:** Instead of just damage, high turbulence should add a random delay to the fleet's arrival time, simulating the "scattered fleets" aspect.
- **Visualization:** Make the hyperlane edge visually throb or turn red on the Layer 2 map when turbulence is high.

## 6. Acceptance Criteria
- [ ] `Hyperlane` component tracks turbulence.
- [ ] Fleet jumps increase turbulence on the hyperlane.
- [ ] Jumps through turbulent hyperlanes apply Hull damage to fleets.
- [ ] All RED phase tests pass.
- [ ] Coverage >= 85%.
- [ ] 0 clippy warnings.

## 7. Technical Guidance
- `apply_turbulence_damage` should ideally run after `process_hyperlane_traffic` or be carefully ordered to apply the correct turbulence values (either before or after the jump's contribution).
- Consider making the threshold for damage configurable or derived from ship mass/size.

## 8. Questions
*Builder: Add any questions here.*
