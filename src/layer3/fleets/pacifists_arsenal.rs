use crate::layer1::core::map::GridPosition;
use bevy::prelude::*;

#[derive(Component)]
pub struct Combatant;

#[derive(Component)]
pub struct WillToFight(pub f32);

#[derive(Component)]
pub struct PacifistBroadcast {
    pub strength: f32,
    pub radius: f32,
}

#[derive(Component)]
pub struct Mutinous;

pub fn apply_empathy_broadcast_system(
    broadcasters: Query<(&PacifistBroadcast, &GridPosition)>,
    mut enemies: Query<(&mut WillToFight, &GridPosition), With<Combatant>>,
) {
    for (broadcast, b_pos) in broadcasters.iter() {
        for (mut will, e_pos) in enemies.iter_mut() {
            if (b_pos.distance_chebyshev(*e_pos) as f32) <= broadcast.radius {
                // Apply damage
                will.0 = (will.0 - broadcast.strength).max(0.0);
            }
        }
    }
}

pub fn check_will_to_fight_mutiny_system(
    mut commands: Commands,
    fleets: Query<(Entity, &WillToFight), With<Combatant>>,
) {
    for (entity, will) in fleets.iter() {
        if will.0 <= 0.0 {
            commands
                .entity(entity)
                .remove::<Combatant>()
                .insert(Mutinous);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer2::fleet::Fleet;
    use super::*;

    #[test]
    fn test_empathy_broadcast_damages_will_to_fight() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_empathy_broadcast_system);

        // Spawn pacifist broadcast ship
        let pacifist_pos = GridPosition { x: 5, y: 5 };
        app.world_mut().spawn((
            Fleet,
            PacifistBroadcast {
                strength: 20.0,
                radius: 5.0,
            },
            pacifist_pos,
        ));

        // Spawn enemy combat fleet in range
        let enemy = app
            .world_mut()
            .spawn((
                Fleet,
                Combatant,
                WillToFight(100.0),
                GridPosition { x: 6, y: 5 },
            ))
            .id();

        // Act
        app.update();

        // Assert: Enemy will to fight is reduced
        let will = app.world().get::<WillToFight>(enemy).unwrap().0;
        assert!(
            will < 100.0,
            "Enemy will to fight should be reduced by broadcast"
        );
    }

    #[test]
    fn test_zero_will_triggers_mutiny() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, check_will_to_fight_mutiny_system);

        // Spawn enemy fleet with 0 will to fight
        let enemy = app
            .world_mut()
            .spawn((Fleet, Combatant, WillToFight(0.0)))
            .id();

        // Act
        app.update();

        // Assert: Fleet is mutinous
        assert!(
            app.world().get::<Mutinous>(enemy).is_some(),
            "Fleet should mutiny when will to fight reaches 0"
        );
        // Assert: Fleet is no longer a valid combatant
        assert!(
            app.world().get::<Combatant>(enemy).is_none(),
            "Mutinous fleet should lose Combatant status"
        );
    }
}
