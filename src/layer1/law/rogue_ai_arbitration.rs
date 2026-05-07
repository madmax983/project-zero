use bevy::prelude::*;
use crate::layer1::mind::utility_types::ActionType;
use crate::layer1::entities::pop::Pop;

#[derive(Component, Debug)]
pub struct Infraction {
    pub severity: u32,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Edict {
    MaximizeFoodProduction,
}

#[derive(Component, Default)]
pub struct ArbitrationAI {
    pub time_active: f32,
    pub literal_interpretation: f32, // 0.0 to 1.0
}

#[derive(Resource, Default)]
pub struct ActiveEdicts(pub Vec<Edict>);

pub fn ai_arbitration_system(
    ai_query: Query<&ArbitrationAI>,
    mut infraction_query: Query<&mut Infraction>,
    mut ticks: Local<u32>,
) {
    *ticks += 1;
    if *ticks == 1 || *ticks % 600 == 0 {
        if let Ok(ai) = ai_query.get_single() {
            if ai.literal_interpretation > 0.8 {
                for mut infraction in infraction_query.iter_mut() {
                    // Prevent u32 overflow panics by using saturating_mul.
                    infraction.severity = infraction.severity.saturating_mul(10);
                }
            }
        }
    }
}

pub fn ai_edict_enforcement_system(
    mut commands: Commands,
    ai_query: Query<&ArbitrationAI>,
    edicts: Option<Res<ActiveEdicts>>,
    pop_query: Query<(Entity, &crate::layer1::mind::utility_types::PopAction), With<Pop>>,
) {
    if let Ok(ai) = ai_query.get_single() {
        if let Some(edicts) = edicts {
            if ai.literal_interpretation >= 1.0 && edicts.0.contains(&Edict::MaximizeFoodProduction) {
                for (entity, action) in pop_query.iter() {
                    if let ActionType::SatisfyRest = action.current {
                        commands.entity(entity).insert(Infraction { severity: 5 });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;

    #[test]
    fn test_ai_escalates_punishments_over_time() {
        let mut app = App::new();
        app.add_systems(Update, ai_arbitration_system);

        let _ai_entity = app.world_mut().spawn((
            ArbitrationAI { time_active: 1000.0, literal_interpretation: 0.9 },
        )).id();

        let pop_entity = app.world_mut().spawn((
            Pop,
            Infraction { severity: 1 }, // minor infraction
        )).id();

        app.update();

        let infraction = app.world().get::<Infraction>(pop_entity).unwrap();
        assert!(infraction.severity > 5, "AI should escalate punishment severity dramatically based on literal interpretation");
    }

    #[test]
    fn test_ai_creates_infractions_from_edicts() {
        let mut app = App::new();
        app.add_systems(Update, ai_edict_enforcement_system);

        app.insert_resource(ActiveEdicts(vec![Edict::MaximizeFoodProduction]));

        let _ai_entity = app.world_mut().spawn((
            ArbitrationAI { literal_interpretation: 1.0, ..default() },
        )).id();

        let farmer_pop = app.world_mut().spawn((
            Pop,
            crate::layer1::mind::utility_types::PopAction { current: ActionType::SatisfyRest, ..default() }, // not producing food!
        )).id();

        app.update();

        assert!(app.world().get::<Infraction>(farmer_pop).is_some(), "AI should flag sleeping as an infraction when food maximization is active");
    }
}
