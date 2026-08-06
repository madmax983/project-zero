use crate::layer1::core::map::GridPosition;
use crate::layer1::physics::gravity_plating::PowerNode;
use crate::layer1::shields::DamageEvent;
use bevy::prelude::*;

#[derive(Component)]
pub struct PocketDimension {
    pub is_stable: bool,
    pub external_position: GridPosition,
}

#[derive(Component)]
pub struct InsidePocket {
    pub pocket: Entity,
}

#[derive(Event)]
pub struct PocketCollapseEvent {
    pub pocket: Entity,
}

pub fn monitor_pocket_power_system(
    mut query: Query<(Entity, &mut PocketDimension, &PowerNode)>,
    mut collapse_events: EventWriter<PocketCollapseEvent>,
) {
    for (entity, mut dim, power) in query.iter_mut() {
        if dim.is_stable && power.current_power < power.required_power {
            dim.is_stable = false;
            collapse_events.send(PocketCollapseEvent { pocket: entity });
        }
    }
}

pub fn process_pocket_collapse_system(
    mut commands: Commands,
    mut collapse_events: EventReader<PocketCollapseEvent>,
    dim_query: Query<&PocketDimension>,
    mut contents_query: Query<(Entity, &InsidePocket, &mut GridPosition)>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    for event in collapse_events.read() {
        if let Ok(dim) = dim_query.get(event.pocket) {
            for (ent, inside, mut pos) in contents_query.iter_mut() {
                if inside.pocket == event.pocket {
                    // Eject to external position
                    pos.x = dim.external_position.x;
                    pos.y = dim.external_position.y;

                    // Remove InsidePocket component
                    commands.entity(ent).remove::<InsidePocket>();

                    // Apply ejection trauma
                    damage_events.send(DamageEvent {
                        target: ent,
                        amount: 50.0,
                        velocity: 50.0,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::core::map::GridPosition;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::physics::gravity_plating::PowerNode;
    use crate::layer1::shields::DamageEvent;

    #[test]
    fn test_power_failure_collapses_pocket_dimension() {
        let mut app = App::new();
        app.add_event::<PocketCollapseEvent>();
        app.add_systems(Update, monitor_pocket_power_system);

        let pocket = app
            .world_mut()
            .spawn((
                PocketDimension {
                    is_stable: true,
                    external_position: GridPosition { x: 5, y: 5 },
                },
                PowerNode {
                    current_power: 0,
                    required_power: 100,
                }, // Underpowered
            ))
            .id();

        app.update();

        let collapse_events = app.world().resource::<Events<PocketCollapseEvent>>();
        let mut reader = collapse_events.get_cursor();
        let mut found = false;
        for event in reader.read(collapse_events) {
            if event.pocket == pocket {
                found = true;
            }
        }

        assert!(
            found,
            "An underpowered pocket dimension should trigger a collapse event."
        );
        let dim = app.world().get::<PocketDimension>(pocket).unwrap();
        assert!(
            !dim.is_stable,
            "Pocket dimension should be marked unstable."
        );
    }

    #[test]
    fn test_collapse_ejects_and_damages_contents() {
        let mut app = App::new();
        app.add_event::<PocketCollapseEvent>();
        app.add_event::<DamageEvent>();
        app.add_systems(Update, process_pocket_collapse_system);

        let pocket = app
            .world_mut()
            .spawn((PocketDimension {
                is_stable: false,
                external_position: GridPosition { x: 5, y: 5 },
            },))
            .id();

        // Spawn a pop "inside" the pocket
        let trapped_pop = app
            .world_mut()
            .spawn((
                Pop,
                InsidePocket { pocket },
                GridPosition { x: 1, y: 1 }, // Internal coordinates
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<PocketCollapseEvent>>()
            .send(PocketCollapseEvent { pocket });

        app.update();

        // Check ejection position
        let pop_pos = app.world().get::<GridPosition>(trapped_pop).unwrap();
        assert_eq!(
            pop_pos.x, 5,
            "Ejected pop should match external pocket X coordinate."
        );
        assert_eq!(
            pop_pos.y, 5,
            "Ejected pop should match external pocket Y coordinate."
        );

        // Check damage
        let damage_events = app.world().resource::<Events<DamageEvent>>();
        let mut reader = damage_events.get_cursor();
        let mut found_damage = false;
        for event in reader.read(damage_events) {
            if event.target == trapped_pop {
                found_damage = true;
            }
        }
        assert!(
            found_damage,
            "Pops ejected from a collapsing pocket dimension should take damage."
        );

        // Check InsidePocket removal
        let inside_component = app.world().get::<InsidePocket>(trapped_pop);
        assert!(
            inside_component.is_none(),
            "InsidePocket component should be removed on ejection"
        );
    }
}
