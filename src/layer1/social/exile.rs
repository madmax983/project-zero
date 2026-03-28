//! Exile system
//!
//! Handles the Banishment of Pops to the outside, removing them from the colony
//! and potentially returning them later.

use bevy_ecs::prelude::*;
use crate::shared::time::SimulationTime;
use crate::layer1::pop::Pop;

#[derive(Component)]
pub struct Crime {
    pub severity: u32,
}

#[derive(Component)]
pub enum BanishmentState {
    Pending,
}

#[derive(Component)]
pub struct ExiledPop {
    pub exiled_at_tick: u64,
    pub base_crime_severity: u32,
    pub has_returned: bool,
    pub return_role: Option<String>,
}

#[allow(clippy::type_complexity)]
pub fn process_banishments(
    mut commands: Commands,
    query: Query<(Entity, &Crime), (With<BanishmentState>, With<Pop>)>,
    sim_time: Res<SimulationTime>,
) {
    for (entity, crime) in query.iter() {
        commands.entity(entity).remove::<Pop>();
        commands.entity(entity).remove::<BanishmentState>();
        commands.entity(entity).remove::<Crime>();
        commands.entity(entity).insert(ExiledPop {
            exiled_at_tick: sim_time.tick,
            base_crime_severity: crime.severity,
            has_returned: false,
            return_role: None,
        });
    }
}

pub fn evaluate_exile_returns(
    mut query: Query<&mut ExiledPop>,
    sim_time: Res<SimulationTime>,
) {
    for mut exiled_pop in query.iter_mut() {
        if !exiled_pop.has_returned {
            let elapsed = sim_time.tick.saturating_sub(exiled_pop.exiled_at_tick);
            if elapsed >= 1_000_000 {
                exiled_pop.has_returned = true;
                exiled_pop.return_role = Some("Pirate".to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::shared::time::SimulationTime;
    use bevy_app::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(SimulationTime::default());
        app.add_systems(Update, (process_banishments, evaluate_exile_returns));
        app
    }

    #[test]
    fn test_banish_pop_removes_from_layer1() {
        let mut app = setup_app();

        let pop_entity = app.world_mut().spawn((
            Pop,
            Crime { severity: 5 },
            BanishmentState::Pending,
        )).id();

        app.update();

        // Assert: The pop entity should no longer have the `Pop` component,
        // or should have been moved out of the active simulation pool and tagged as `ExiledPop`.
        assert!(app.world().get::<Pop>(pop_entity).is_none());
        assert!(app.world().get::<ExiledPop>(pop_entity).is_some());
    }

    #[test]
    fn test_exiled_pop_returns_after_years() {
        let mut app = setup_app();

        let pop_entity = app.world_mut().spawn((
            ExiledPop {
                exiled_at_tick: 0,
                base_crime_severity: 5,
                has_returned: false,
                return_role: None,
            },
        )).id();

        // Fast forward simulation time by several years (ticks)
        let mut sim_time = app.world_mut().resource_mut::<SimulationTime>();
        sim_time.tick = 1_000_000; // Simulated time lapse

        app.update();

        // Assert: A return event or component flag should indicate the pop is trying to return
        let exiled_pop = app.world().get::<ExiledPop>(pop_entity).unwrap();
        assert!(exiled_pop.has_returned);
        assert!(exiled_pop.return_role.is_some()); // e.g., Pirate, Merchant
    }
}
