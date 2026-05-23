use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct ColonyRenown {
    pub score: f32, // 0.0 to 100.0
}

#[derive(Component, PartialEq, Eq, Debug, Clone, Copy)]
pub enum AestheticLevel {
    Rotting,
    Normal,
    Idealized,
}

#[derive(Component)]
pub struct AestheticState {
    pub level: AestheticLevel,
}

pub fn update_living_score_aesthetics(
    renown: Res<ColonyRenown>,
    mut query: Query<&mut AestheticState>,
) {
    let target_level = if renown.score >= 80.0 {
        AestheticLevel::Idealized
    } else if renown.score <= 20.0 {
        AestheticLevel::Rotting
    } else {
        AestheticLevel::Normal
    };

    for mut state in query.iter_mut() {
        if state.level != target_level {
            state.level = target_level;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_renown_improves_aesthetics() {
        // Arrange
        let mut app = bevy_ecs::world::World::new();
        app.insert_resource(ColonyRenown { score: 95.0 });

        let entity = app
            .spawn(AestheticState {
                level: AestheticLevel::Normal,
            })
            .id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(update_living_score_aesthetics);
        schedule.run(&mut app);

        // Assert: High renown should shift state to Idealized
        let state = app.get::<AestheticState>(entity).unwrap();
        assert_eq!(state.level, AestheticLevel::Idealized);
    }

    #[test]
    fn test_low_renown_causes_rot() {
        // Arrange
        let mut app = bevy_ecs::world::World::new();
        app.insert_resource(ColonyRenown { score: 10.0 });

        let entity = app
            .spawn(AestheticState {
                level: AestheticLevel::Normal,
            })
            .id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(update_living_score_aesthetics);
        schedule.run(&mut app);

        // Assert: Low renown should shift state to Rotting
        let state = app.get::<AestheticState>(entity).unwrap();
        assert_eq!(state.level, AestheticLevel::Rotting);
    }
}
