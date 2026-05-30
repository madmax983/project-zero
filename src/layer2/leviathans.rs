use bevy_ecs::prelude::*;
use crate::layer1::economy::resources::ColonyResources;

#[derive(Component)]
pub struct Leviathan {
    pub hunger: u32,
    pub home_base: Entity,
}

#[derive(Component)]
pub struct Tamed {}

pub fn process_leviathan_hunger(
    mut commands: bevy_ecs::system::Commands,
    mut colonies: bevy_ecs::system::Query<&mut ColonyResources>,
    mut leviathans: Query<(Entity, &mut Leviathan, Option<&Tamed>)>,
) {
    for (entity, mut leviathan, tamed_opt) in leviathans.iter_mut() {
        if tamed_opt.is_none() {
            continue; // Already feral
        }

        if let Ok(mut resources) = colonies.get_mut(leviathan.home_base) {
            if resources.food >= leviathan.hunger as f32 {
                resources.food -= leviathan.hunger as f32;
                leviathan.hunger = 0;
            } else {
                // Not enough food to satisfy the beast
                commands.entity(entity).remove::<Tamed>();
            }
        } else {
            // Home base invalid, go feral
            commands.entity(entity).remove::<Tamed>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_leviathan_consumes_resources_and_remains_docile() {
        let mut app = App::new();
        app.add_systems(Update, process_leviathan_hunger);

        let colony = app.world_mut().spawn(ColonyResources {
            food: 10000.0, // Massive amount of livestock/flora
            ..Default::default()
        }).id();

        let leviathan = app.world_mut().spawn((
            Leviathan { hunger: 500, home_base: colony },
            Tamed {},
        )).id();

        app.update();

        let resources = app.world().get::<ColonyResources>(colony).unwrap();
        let lev = app.world().get::<Leviathan>(leviathan).unwrap();

        assert_eq!(resources.food, 9500.0, "Leviathan should consume massive food per tick");
        assert_eq!(lev.hunger, 0, "Leviathan hunger should be satisfied");
        assert!(app.world().get::<Tamed>(leviathan).is_some(), "Leviathan remains tamed");
    }

    #[test]
    fn test_leviathan_goes_feral_when_starved() {
        let mut app = App::new();
        app.add_systems(Update, process_leviathan_hunger);

        let colony = app.world_mut().spawn(ColonyResources {
            food: 0.0, // No food
            ..Default::default()
        }).id();

        let leviathan = app.world_mut().spawn((
            Leviathan { hunger: 500, home_base: colony },
            Tamed {},
        )).id();

        app.update();

        let lev = app.world().get::<Leviathan>(leviathan).unwrap();
        assert_eq!(lev.hunger, 500, "Hunger is not satisfied");
        assert!(app.world().get::<Tamed>(leviathan).is_none(), "Leviathan should lose Tamed component and go feral");
    }
}
