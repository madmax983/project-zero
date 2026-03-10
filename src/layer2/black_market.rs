use bevy_ecs::prelude::*;

#[derive(Resource, Default, Debug, Clone)]
pub struct ColonyStats {
    pub unmet_luxury: u32,
    pub corruption: f32,
}

#[derive(Component)]
pub struct Smuggler;

pub fn black_market_spawn_system(mut commands: Commands, stats: Option<Res<ColonyStats>>) {
    if let Some(s) = stats {
        if s.unmet_luxury >= 100 {
            // Spawn a smuggler entity
            commands.spawn(Smuggler);
        }
    }
}

pub fn smuggler_trade_system(stats: Option<ResMut<ColonyStats>>, query: Query<&Smuggler>) {
    let smuggler_count = query.iter().count();
    if smuggler_count > 0 {
        if let Some(mut s) = stats {
            s.unmet_luxury = s.unmet_luxury.saturating_sub((10 * smuggler_count) as u32);
            s.corruption += 1.0 * smuggler_count as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_unmet_needs_spawn_smugglers() {
        // Arrange
        let mut world = World::new();
        // Setup colony with massive unmet luxury needs
        world.insert_resource(ColonyStats {
            unmet_luxury: 100,
            ..Default::default()
        });

        // Act
        world.run_system_once(black_market_spawn_system).unwrap();

        // Assert
        let smugglers = world.query::<&Smuggler>().iter(&world).count();
        assert_eq!(smugglers, 1, "A smuggler should spawn due to unmet needs");
    }

    #[test]
    fn test_smuggler_increases_corruption() {
        // Arrange
        let mut world = World::new();
        world.insert_resource(ColonyStats {
            corruption: 0.0,
            ..Default::default()
        });
        let _smuggler = world.spawn(Smuggler).id();

        // Act
        world.run_system_once(smuggler_trade_system).unwrap(); // Mock trade

        // Assert
        let stats = world.get_resource::<ColonyStats>().unwrap();
        assert!(
            stats.corruption > 0.0,
            "Corruption should increase after smuggler trade"
        );
    }
}
