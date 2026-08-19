#[allow(unused_imports)]
use bevy::prelude::*;

#[derive(Event)]
pub struct DefectorArrivalEvent {
    pub leader_name: String,
    pub tech_bonus_value: f32,
    pub colony_id: Entity,
}

#[derive(Component)]
pub struct TechProgress {
    pub points: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ideology {
    Egalitarian,
    Authoritarian,
}

#[derive(Component)]
pub struct DefectorState {
    pub regret_level: f32,
    pub ideology: Ideology,
}

#[derive(Component)]
pub struct ColonyIdeology {
    pub ideology: Ideology,
}

#[derive(Event)]
pub struct SabotageEvent {}

pub fn process_defector_arrival_system(
    mut events: EventReader<DefectorArrivalEvent>,
    mut colonies: Query<&mut TechProgress>,
) {
    for event in events.read() {
        if let Ok(mut progress) = colonies.get_mut(event.colony_id) {
            progress.points += event.tech_bonus_value;
        }
    }
}

pub fn defector_values_clash_system(
    mut defectors: Query<&mut DefectorState>,
    colonies: Query<&ColonyIdeology>,
) {
    // Just using the first colony for MVP as tests don't tie defector to colony yet
    if let Some(colony) = colonies.iter().next() {
        for mut defector in defectors.iter_mut() {
            if defector.ideology != colony.ideology {
                defector.regret_level += 1.0;
            }
        }
    }
}

pub fn defector_sabotage_system(
    defectors: Query<&DefectorState>,
    mut events: EventWriter<SabotageEvent>,
) {
    for defector in defectors.iter() {
        if defector.regret_level >= 100.0 {
            events.send(SabotageEvent {});
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use bevy::prelude::*;

    #[test]
    fn test_defector_arrival_grants_tech_boost() {
        let mut app = App::new();
        app.add_event::<DefectorArrivalEvent>();
        app.add_systems(Update, process_defector_arrival_system);

        // Setup initial tech level
        let colony = app.world_mut().spawn(TechProgress { points: 100.0 }).id();

        app.world_mut().send_event(DefectorArrivalEvent {
            leader_name: "Admiral Thrawn".to_string(),
            tech_bonus_value: 500.0,
            colony_id: colony,
        });

        app.update();

        // Check if tech points increased
        let progress = app.world().get::<TechProgress>(colony).unwrap();
        assert_eq!(
            progress.points, 600.0,
            "Defector arrival should grant an immediate tech bonus."
        );
    }

    #[test]
    fn test_defector_accumulates_values_clash() {
        let mut app = App::new();
        app.add_systems(Update, defector_values_clash_system);

        // Spawn a defector with incompatible values
        let defector = app
            .world_mut()
            .spawn(DefectorState {
                regret_level: 0.0,
                ideology: Ideology::Egalitarian,
            })
            .id();

        // Spawn a local colony with conflicting values
        app.world_mut().spawn(ColonyIdeology {
            ideology: Ideology::Authoritarian,
        });

        app.update();

        // Regret level should increase due to the clash
        let state = app.world().get::<DefectorState>(defector).unwrap();
        assert!(
            state.regret_level > 0.0,
            "Regret level should increase when values clash."
        );
    }

    #[test]
    fn test_high_regret_triggers_sabotage() {
        let mut app = App::new();
        app.add_event::<SabotageEvent>();
        app.add_systems(Update, defector_sabotage_system);

        // Spawn a defector with max regret
        app.world_mut().spawn(DefectorState {
            regret_level: 100.0,
            ideology: Ideology::Egalitarian,
        });

        app.update();

        // Sabotage event should be triggered
        let sabotage_events = app.world().resource::<Events<SabotageEvent>>();
        #[allow(deprecated)]
        {
            assert!(
                sabotage_events.get_reader().len(sabotage_events) > 0,
                "High regret should trigger sabotage."
            );
        }
    }
}
