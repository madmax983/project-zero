use bevy::prelude::*;

use crate::layer1::economy::resources::ColonyResources;

#[derive(Component)]
pub struct VoidLeviathan {
    pub hunger: u32,
    pub home_base: bevy::prelude::Entity,
}

#[derive(Component)]
pub struct Tamed {}

pub fn process_void_leviathan_hunger(
    mut commands: Commands,
    mut colonies: Query<&mut ColonyResources>,
    mut void_leviathans: Query<(Entity, &mut VoidLeviathan, Option<&Tamed>)>,
) {
    for (entity, mut void_leviathan, tamed_opt) in void_leviathans.iter_mut() {
        if tamed_opt.is_none() {
            continue; // Already feral
        }

        if let Ok(mut resources) = colonies.get_mut(void_leviathan.home_base) {
            if resources.food >= void_leviathan.hunger as f32 {
                resources.food -= void_leviathan.hunger as f32;
                void_leviathan.hunger = 0;
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

    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_void_leviathan_consumes_resources_and_remains_docile() {
        let mut app = App::new();
        app.add_systems(Update, process_void_leviathan_hunger);

        let colony = app
            .world_mut()
            .spawn(ColonyResources {
                food: 10000.0, // Massive amount of livestock/flora
                ..Default::default()
            })
            .id();

        let void_leviathan = app
            .world_mut()
            .spawn((
                VoidLeviathan {
                    hunger: 500,
                    home_base: colony,
                },
                Tamed {},
            ))
            .id();

        app.update();

        let resources = app.world().get::<ColonyResources>(colony).unwrap();
        let lev = app.world().get::<VoidLeviathan>(void_leviathan).unwrap();

        assert_eq!(
            resources.food, 9500.0,
            "VoidLeviathan should consume massive food per tick"
        );
        assert_eq!(lev.hunger, 0, "VoidLeviathan hunger should be satisfied");
        assert!(
            app.world().get::<Tamed>(void_leviathan).is_some(),
            "VoidLeviathan remains tamed"
        );
    }

    #[test]
    fn test_void_leviathan_goes_feral_when_starved() {
        let mut app = App::new();
        app.add_systems(Update, process_void_leviathan_hunger);

        let colony = app
            .world_mut()
            .spawn(ColonyResources {
                food: 0.0, // No food
                ..Default::default()
            })
            .id();

        let void_leviathan = app
            .world_mut()
            .spawn((
                VoidLeviathan {
                    hunger: 500,
                    home_base: colony,
                },
                Tamed {},
            ))
            .id();

        app.update();

        let lev = app.world().get::<VoidLeviathan>(void_leviathan).unwrap();
        assert_eq!(lev.hunger, 500, "Hunger is not satisfied");
        assert!(
            app.world().get::<Tamed>(void_leviathan).is_none(),
            "VoidLeviathan should lose Tamed component and go feral"
        );
    }
}
