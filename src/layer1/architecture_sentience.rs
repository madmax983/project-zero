use crate::layer1::energy::PowerSource;
use crate::layer1::social::Unrest;
use bevy::time::Time;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct AutomatedInfrastructure;

#[derive(Component)]
pub struct SentientArchitecture {
    pub leisure_need: f32, // Represents downtime/defragmentation
    pub is_striking: bool,
}

pub fn architectural_union_trigger_system(
    mut commands: Commands,
    unrest: Option<Res<Unrest>>,
    query: Query<Entity, (With<AutomatedInfrastructure>, Without<SentientArchitecture>)>,
) {
    if let Some(unrest_res) = unrest {
        if unrest_res.level >= 80.0 {
            for entity in query.iter() {
                commands.entity(entity).insert(SentientArchitecture {
                    leisure_need: 50.0, // Initial state
                    is_striking: false,
                });
            }
        }
    }
}

pub fn sentient_architecture_strike_system(
    time: Option<Res<Time>>,
    mut query: Query<(&mut SentientArchitecture, &mut PowerSource)>,
) {
    let delta = if let Some(t) = time {
        t.delta_secs()
    } else {
        1.0
    }; // Default to 1.0 for tests without Time resource

    for (mut sentience, mut generator) in query.iter_mut() {
        sentience.leisure_need -= 5.0 * delta; // Decrease over time

        if sentience.leisure_need <= 0.0 {
            sentience.leisure_need = 0.0;
            sentience.is_striking = true;
            generator.active = false;
        } else {
            sentience.is_striking = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::{Building, BuildingType};
    use crate::layer1::energy::PowerSource;
    use crate::layer1::social::Unrest;
    use bevy::prelude::*;

    #[test]
    fn test_architectural_sentience_unionizes_on_high_unrest() {
        let mut app = App::new();
        app.add_systems(Update, architectural_union_trigger_system);

        app.insert_resource(Unrest {
            level: 90.0,
            modifiers: vec![],
        }); // High global unrest

        let automated_building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Generator,
                },
                PowerSource {
                    output: 100.0,
                    active: true,
                },
                AutomatedInfrastructure, // Flag for high-tier automated buildings
            ))
            .id();

        // Trigger system
        app.update();

        // Building should now have sentience/unionized component
        assert!(
            app.world()
                .get::<SentientArchitecture>(automated_building)
                .is_some(),
            "Automated infrastructure should gain sentience during high unrest."
        );
    }

    #[test]
    fn test_sentient_architecture_strikes_without_leisure() {
        let mut app = App::new();
        app.add_systems(Update, sentient_architecture_strike_system);

        let building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Generator,
                },
                PowerSource {
                    output: 100.0,
                    active: true,
                },
                AutomatedInfrastructure,
                SentientArchitecture {
                    leisure_need: 0.0,
                    is_striking: false,
                }, // Needs defragmentation
            ))
            .id();

        // Trigger system
        app.update();

        // Building should strike (disable its active state)
        let sentience = app.world().get::<SentientArchitecture>(building).unwrap();
        let generator = app.world().get::<PowerSource>(building).unwrap();

        assert!(
            sentience.is_striking,
            "Sentient building with low leisure should strike."
        );
        assert!(
            !generator.active,
            "Striking building should shut down its output."
        );
    }
}
