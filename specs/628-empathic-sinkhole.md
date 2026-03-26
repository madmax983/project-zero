# Spec 628: The Empathic Sinkhole

## 1. Overview
The Empathic Sinkhole is a unique Layer 1 terrain feature. It acts as an emotional sponge for colonists. Pops with high Stress will naturally pathfind to the sinkhole to "Vent," instantly reducing their stress to zero. However, the sinkhole stores this absorbed stress. Once its hidden internal capacity is reached, it violently erupts, applying a massive "Despair" debuff to all Pops in a large radius, proportional to the stress it absorbed. It creates a tension between a free, immediate solution to stress and a localized emotional time bomb.

## 2. Dependencies
- **Layer 1 Needs System** (Stress/Mood tracking)
- **Layer 1 Pathfinding & Utility AI** (Pops must want to visit the sinkhole)
- **Layer 1 Terrain/Grid** (The Sinkhole must exist in the world)

## 3. RED Phase: Tests First

```rust
// specs/628-empathic-sinkhole.md

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // Mock components and resources
    #[derive(Component)]
    struct Pop {
        stress: f32,
    }

    #[derive(Component)]
    struct EmpathicSinkhole {
        absorbed_stress: f32,
        capacity: f32,
    }

    #[derive(Component)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Component)]
    struct DespairDebuff {
        duration: f32,
        intensity: f32,
    }

    #[derive(Event)]
    struct VentStressEvent {
        pop_entity: Entity,
        sinkhole_entity: Entity,
    }

    #[test]
    fn test_venting_stress_transfers_stress_to_sinkhole() {
        // Arrange
        let mut app = App::new();
        app.add_event::<VentStressEvent>();
        app.add_systems(Update, process_stress_venting);

        let pop = app.world.spawn(Pop { stress: 50.0 }).id();
        let sinkhole = app.world.spawn(EmpathicSinkhole { absorbed_stress: 0.0, capacity: 100.0 }).id();

        // Act
        app.world.send_event(VentStressEvent { pop_entity: pop, sinkhole_entity: sinkhole });
        app.update();

        // Assert
        let pop_data = app.world.get::<Pop>(pop).unwrap();
        assert_eq!(pop_data.stress, 0.0, "Pop's stress should be reduced to 0");
        let sinkhole_data = app.world.get::<EmpathicSinkhole>(sinkhole).unwrap();
        assert_eq!(sinkhole_data.absorbed_stress, 50.0, "Sinkhole should absorb the pop's stress");
    }

    #[test]
    fn test_sinkhole_erupts_when_capacity_reached() {
        // Arrange
        let mut app = App::new();
        app.add_event::<VentStressEvent>();
        app.add_systems(Update, (process_stress_venting, trigger_sinkhole_eruption).chain());

        let pop1 = app.world.spawn((Pop { stress: 60.0 }, Position { x: 5.0, y: 5.0 })).id();
        let pop2 = app.world.spawn((Pop { stress: 60.0 }, Position { x: 6.0, y: 5.0 })).id();
        let pop_far = app.world.spawn((Pop { stress: 10.0 }, Position { x: 100.0, y: 100.0 })).id();

        let sinkhole = app.world.spawn((
            EmpathicSinkhole { absorbed_stress: 0.0, capacity: 100.0 },
            Position { x: 0.0, y: 0.0 },
        )).id();

        // Act
        app.world.send_event(VentStressEvent { pop_entity: pop1, sinkhole_entity: sinkhole });
        app.world.send_event(VentStressEvent { pop_entity: pop2, sinkhole_entity: sinkhole });
        app.update();

        // Assert
        let sinkhole_data = app.world.get::<EmpathicSinkhole>(sinkhole).unwrap();
        assert_eq!(sinkhole_data.absorbed_stress, 0.0, "Sinkhole stress should reset after eruption");

        // Check debuffs
        let pop1_debuff = app.world.get::<DespairDebuff>(pop1);
        assert!(pop1_debuff.is_some(), "Nearby pop should receive despair debuff");

        let pop_far_debuff = app.world.get::<DespairDebuff>(pop_far);
        assert!(pop_far_debuff.is_none(), "Faraway pop should not receive despair debuff");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop {
    pub stress: f32,
}

#[derive(Component)]
pub struct EmpathicSinkhole {
    pub absorbed_stress: f32,
    pub capacity: f32,
}

#[derive(Component, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct DespairDebuff {
    pub duration: f32,
    pub intensity: f32,
}

#[derive(Event)]
pub struct VentStressEvent {
    pub pop_entity: Entity,
    pub sinkhole_entity: Entity,
}

pub fn process_stress_venting(
    mut events: EventReader<VentStressEvent>,
    mut pops: Query<&mut Pop>,
    mut sinkholes: Query<&mut EmpathicSinkhole>,
) {
    for event in events.read() {
        if let Ok(mut pop) = pops.get_mut(event.pop_entity) {
            if let Ok(mut sinkhole) = sinkholes.get_mut(event.sinkhole_entity) {
                sinkhole.absorbed_stress += pop.stress;
                pop.stress = 0.0;
            }
        }
    }
}

pub fn trigger_sinkhole_eruption(
    mut commands: Commands,
    mut sinkholes: Query<(&mut EmpathicSinkhole, &Position)>,
    pops: Query<(Entity, &Position), With<Pop>>,
) {
    let eruption_radius = 20.0;

    for (mut sinkhole, sinkhole_pos) in sinkholes.iter_mut() {
        if sinkhole.absorbed_stress >= sinkhole.capacity {
            let intensity = sinkhole.absorbed_stress;
            sinkhole.absorbed_stress = 0.0; // Reset

            // Apply debuff to nearby pops
            for (pop_entity, pop_pos) in pops.iter() {
                let dx = sinkhole_pos.x - pop_pos.x;
                let dy = sinkhole_pos.y - pop_pos.y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq <= eruption_radius * eruption_radius {
                    commands.entity(pop_entity).insert(DespairDebuff {
                        duration: 60.0,
                        intensity,
                    });
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Action / Utility AI Integration**: Add a new `ActionType` (e.g., `VentAtSinkhole`) and score it highly when a Pop's stress is dangerously high and a Sinkhole is reachable.
- **Particle/Visual Effects**: When the sinkhole erupts, spawn a visual indicator (like a shockwave or dark pulse) to give players feedback on what just happened.
- **Chronicle Event**: Log the eruption to the Chronicle so the player understands the source of the sudden, massive colony-wide despair.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops successfully transfer their stress to the sinkhole.
- [ ] Sinkhole stress accumulation triggers an eruption when capacity is reached.
- [ ] Eruption resets the sinkhole and applies `DespairDebuff` to Pops within the defined radius.

## 7. Technical Guidance
- Ensure distance checks use the actual `TerrainGrid` distance or a reliable continuous coordinate system (like Bevy's `Transform`) depending on how spatial queries are managed in Layer 1.
- You will need to implement a system that processes the `DespairDebuff`, preventing it from ticking down or applying continuous mood penalties over its duration.

## 8. Questions
*Builder: add questions here if spec is unclear.*
