# Specification: The "Empty" Room

## 1. Overview
In a crowded station, space is the ultimate luxury. This feature allows players to designate "Sanctuary" zones that reduce Stress for visiting Pops, provided the zone remains entirely empty of furniture, machines, and clutter. If items are placed or dropped in the zone, the bonus is invalidated.

## 2. Dependencies
- Base Zone and Room Designation systems (Layer 1)
- Pop Stress and Mood systems
- Item dropping/clutter mechanics

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // Mock components for test phase
    #[derive(Component)]
    pub struct SanctuaryZone {
        pub active: bool,
    }

    #[derive(Component)]
    pub struct Clutter;

    #[derive(Component)]
    pub struct Stress {
        pub value: f32,
    }

    #[derive(Component)]
    pub struct InZone {
        pub zone_entity: Entity,
    }

    fn evaluate_sanctuary_emptiness(
        mut q_zones: Query<(Entity, &mut SanctuaryZone)>,
        q_clutter: Query<&InZone, With<Clutter>>,
    ) {
        // Implementation will go here
    }

    fn apply_sanctuary_stress_relief(
        q_zones: Query<&SanctuaryZone>,
        mut q_pops: Query<(&mut Stress, &InZone)>,
    ) {
        // Implementation will go here
    }

    #[test]
    fn test_sanctuary_active_when_empty() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_sanctuary_emptiness);

        let zone = app.world_mut().spawn(SanctuaryZone { active: false }).id();

        app.update();

        let sanctuary = app.world().get::<SanctuaryZone>(zone).unwrap();
        assert!(sanctuary.active, "Sanctuary should be active when no clutter is present");
    }

    #[test]
    fn test_sanctuary_deactivated_by_clutter() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_sanctuary_emptiness);

        let zone = app.world_mut().spawn(SanctuaryZone { active: true }).id();
        app.world_mut().spawn((Clutter, InZone { zone_entity: zone }));

        app.update();

        let sanctuary = app.world().get::<SanctuaryZone>(zone).unwrap();
        assert!(!sanctuary.active, "Sanctuary should deactivate if clutter is inside");
    }

    #[test]
    fn test_stress_relief_in_active_sanctuary() {
        let mut app = App::new();
        app.add_systems(Update, apply_sanctuary_stress_relief);

        let zone = app.world_mut().spawn(SanctuaryZone { active: true }).id();
        let pop = app.world_mut().spawn((Stress { value: 50.0 }, InZone { zone_entity: zone })).id();

        app.update();

        let stress = app.world().get::<Stress>(pop).unwrap();
        assert!(stress.value < 50.0, "Pop should lose stress in an active sanctuary");
    }

    #[test]
    fn test_no_stress_relief_in_inactive_sanctuary() {
        let mut app = App::new();
        app.add_systems(Update, apply_sanctuary_stress_relief);

        let zone = app.world_mut().spawn(SanctuaryZone { active: false }).id();
        let pop = app.world_mut().spawn((Stress { value: 50.0 }, InZone { zone_entity: zone })).id();

        app.update();

        let stress = app.world().get::<Stress>(pop).unwrap();
        assert_eq!(stress.value, 50.0, "Pop should not lose stress in an inactive sanctuary");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The minimal code required to pass the tests

fn evaluate_sanctuary_emptiness(
    mut q_zones: Query<(Entity, &mut SanctuaryZone)>,
    q_clutter: Query<&InZone, With<Clutter>>,
) {
    for (zone_entity, mut sanctuary) in q_zones.iter_mut() {
        let has_clutter = q_clutter.iter().any(|in_zone| in_zone.zone_entity == zone_entity);
        sanctuary.active = !has_clutter;
    }
}

fn apply_sanctuary_stress_relief(
    q_zones: Query<&SanctuaryZone>,
    mut q_pops: Query<(&mut Stress, &InZone)>,
) {
    for (mut stress, in_zone) in q_pops.iter_mut() {
        if let Ok(sanctuary) = q_zones.get(in_zone.zone_entity) {
            if sanctuary.active {
                stress.value = (stress.value - 1.0).max(0.0);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Optimize `evaluate_sanctuary_emptiness` to avoid `O(N*M)` complexity by either maintaining a clutter count on the `SanctuaryZone` via change detection/events, or building a spatial lookup grid if one isn't already used.
- Ensure "Clutter" checks consider dropped items as well as constructed furniture, integrating seamlessly with Layer 1's entity categorization.
- Implement an event when a Sanctuary is ruined by clutter, allowing UI notifications or narrative chronicle entries.
- Adjust stress relief values to be time-scaled using `Time::delta_seconds()`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Sanctuary zones correctly toggle their active state based on presence of items/furniture.
- [ ] Pops only receive stress relief in active sanctuaries.

## 7. Technical Guidance
- `SanctuaryZone` should be implemented as a component applied to designated area entities.
- "Clutter" refers to any entity with physical presence (machinery, storage, dropped resources) that shares the space. Ensure the querying logic checks for the actual Layer 1 representation of these objects (e.g., GridPosition overlaps).
- Pops leaving offerings (flowers, rocks) can be implemented via the Pop Utility AI as an edge-case behavior, scheduling a "Drop Item" task targeted at the Sanctuary tile when stress relief occurs.

## 8. Questions
*Builder: add questions here if spec is unclear.*
