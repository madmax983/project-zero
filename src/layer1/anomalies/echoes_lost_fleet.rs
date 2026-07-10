use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct GhostDreadnought {
    pub is_broadcasting: bool,
    pub ticks_remaining: u32,
}

#[derive(Component)]
pub struct LuredByGhostFleet;

pub fn lure_militaristic_pops_system(
    mut commands: Commands,
    dreadnoughts: Query<&GhostDreadnought>,
    pops: Query<(Entity, &Traits), Without<LuredByGhostFleet>>,
) {
    let mut is_broadcasting = false;
    for dreadnought in dreadnoughts.iter() {
        if dreadnought.is_broadcasting && dreadnought.ticks_remaining > 0 {
            is_broadcasting = true;
            break;
        }
    }

    if !is_broadcasting {
        return;
    }

    for (entity, traits) in pops.iter() {
        if traits.has(Trait::Militaristic) {
            commands.entity(entity).insert(LuredByGhostFleet);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::psychology::traits::{Trait, Traits};
    use bevy_app::prelude::*;

    #[test]
    fn test_lost_fleet_broadcast_lures_militaristic_pops() {
        let mut app = App::new();
        app.add_systems(Update, lure_militaristic_pops_system);

        // Setup the dreadnought anomaly
        app.world_mut().spawn(GhostDreadnought {
            is_broadcasting: true,
            ticks_remaining: 100,
        });

        // Setup a pop with the Militaristic trait
        let mut traits = Traits::default();
        traits.add(Trait::Militaristic);
        let pop_id = app
            .world_mut()
            .spawn((Pop, traits, GridPosition { x: 10, y: 10 }))
            .id();

        // Run the luring system
        app.update();

        // Pop should receive a "Lured" component or be despawned/marked for escape
        let has_lured = app.world().get::<LuredByGhostFleet>(pop_id).is_some();
        assert!(
            has_lured,
            "Militaristic pop should be lured by the ghost fleet broadcast"
        );
    }

    #[test]
    fn test_non_militaristic_pops_ignore_broadcast() {
        let mut app = App::new();
        app.add_systems(Update, lure_militaristic_pops_system);

        // Setup the dreadnought anomaly
        app.world_mut().spawn(GhostDreadnought {
            is_broadcasting: true,
            ticks_remaining: 100,
        });

        // Setup a pop without the Militaristic trait
        let pop_id = app
            .world_mut()
            .spawn((Pop, Traits::default(), GridPosition { x: 10, y: 10 }))
            .id();

        // Run the luring system
        app.update();

        let has_lured = app.world().get::<LuredByGhostFleet>(pop_id).is_some();
        assert!(
            !has_lured,
            "Non-militaristic pop should ignore the ghost fleet broadcast"
        );
    }
}
