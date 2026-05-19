use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Lifeboat {
    pub capacity: usize,
    pub occupants: Vec<Entity>,
    pub launch_triggered: bool,
}

#[derive(Component)]
pub struct DistressSignal {
    pub occupants: Vec<Entity>,
}

pub fn process_lifeboat_launches(
    mut commands: Commands,
    query: Query<(Entity, &Lifeboat)>,
) {
    for (entity, lifeboat) in query.iter() {
        if lifeboat.launch_triggered {
            // Spawn Distress Signal in Orbit
            commands.spawn(DistressSignal {
                occupants: lifeboat.occupants.clone(),
            });

            // Remove Lifeboat from surface
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use crate::layer1::entities::pop::Pop;
    use bevy_app::Update;

    #[test]
    fn test_lifeboat_launch_transfers_pops_to_orbit() {
        // Arrange
        let mut app = App::new();
        // app.add_plugins(MinimalPlugins);
        app.add_systems(Update, process_lifeboat_launches);

        // Spawn Pops
        let pop1 = app.world_mut().spawn(Pop).id();
        let pop2 = app.world_mut().spawn(Pop).id();

        // Spawn a Lifeboat that has been triggered to launch
        let lifeboat = app.world_mut().spawn(Lifeboat {
            capacity: 2,
            occupants: vec![pop1, pop2],
            launch_triggered: true,
        }).id();

        // Act
        app.update();

        // Assert
        // The surface Lifeboat should be destroyed (launched)
        assert!(app.world().get::<Lifeboat>(lifeboat).is_none());

        // A new DistressSignal should appear in orbit with the pops
        let mut q = app.world_mut().query::<&DistressSignal>();
        let signals: Vec<&DistressSignal> = q.iter(app.world()).collect();
        assert_eq!(signals.len(), 1, "Expected one distress signal in orbit");
        assert_eq!(signals[0].occupants.len(), 2);
        assert!(signals[0].occupants.contains(&pop1));
        assert!(signals[0].occupants.contains(&pop2));
    }

    #[test]
    fn test_lifeboat_does_not_launch_if_not_triggered() {
        let mut app = App::new();
        app.add_systems(Update, process_lifeboat_launches);

        let lifeboat = app.world_mut().spawn(Lifeboat {
            capacity: 2,
            occupants: vec![],
            launch_triggered: false,
        }).id();

        app.update();

        assert!(app.world().get::<Lifeboat>(lifeboat).is_some(), "Lifeboat should remain on surface if not triggered");
    }
}
