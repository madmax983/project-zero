use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
/// Colony-wide statistics tracking economy and law.
pub struct ColonyStats {
    /// Measure of how many luxury needs are currently unmet.
    pub unmet_luxury: u32,
    /// Total corruption level in the colony.
    pub corruption: f32,
}

#[derive(Component)]
/// Marker component for a pop acting as a smuggler.
pub struct Smuggler;

/// Spawns smugglers if unmet luxury needs are high.
pub fn black_market_spawn_system(mut commands: Commands, stats: Res<ColonyStats>) {
    if stats.unmet_luxury > 50 {
        commands.spawn(Smuggler);
    }
}

/// Increases corruption when smugglers are present.
pub fn smuggler_trade_system(mut stats: ResMut<ColonyStats>, query: Query<&Smuggler>) {
    if !query.is_empty() {
        stats.corruption += 1.0;
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
