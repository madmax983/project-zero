use bevy::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::social::morale::Morale;
use rand::Rng;

#[derive(Resource)]
pub struct ScrapCodeConfig {
    pub revelation_distance: f32,
    pub aura_distance: f32,
    pub cult_formation_chance: f32,
    pub sabotage_debt_increase: f32,
    pub max_morale_bonus: f32,
}

impl Default for ScrapCodeConfig {
    fn default() -> Self {
        Self {
            revelation_distance: 5.0,
            aura_distance: 10.0,
            cult_formation_chance: 0.1, // 10% chance
            sabotage_debt_increase: 50.0,
            max_morale_bonus: 20.0,
        }
    }
}

#[derive(Component)]
pub struct ScrapCodeCultist {
    pub current_morale_bonus: f32,
}

impl Default for ScrapCodeCultist {
    fn default() -> Self {
        Self {
            current_morale_bonus: 0.0,
        }
    }
}

#[derive(Event)]
pub struct MachineBreakdownEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct CultFormationEvent {
    pub pop: Entity,
}

#[derive(Component, PartialEq, Clone)]
pub enum MachineState {
    Working,
    Broken,
}

#[derive(Component)]
pub struct Machine {
    pub maintenance_debt: f32,
    pub state: MachineState,
}

#[derive(Component)]
pub enum CurrentAction {
    Sabotage(Entity),
    Idle,
}

#[allow(clippy::type_complexity)]
pub fn process_scrap_code_revelations(
    mut events: EventReader<MachineBreakdownEvent>,
    mut commands: Commands,
    machine_query: Query<&Transform, With<Machine>>,
    pop_query: Query<(Entity, &Transform), (With<Pop>, Without<ScrapCodeCultist>)>,
    config: Option<Res<ScrapCodeConfig>>,
) {
    let default_config = ScrapCodeConfig::default();
    let config = config.as_deref().unwrap_or(&default_config);
    let mut rng = rand::thread_rng();

    for event in events.read() {
        if let Ok(machine_transform) = machine_query.get(event.entity) {
            for (pop_entity, pop_transform) in pop_query.iter() {
                if machine_transform.translation.distance(pop_transform.translation) < config.revelation_distance {
                    if rng.gen::<f32>() < config.cult_formation_chance {
                        commands.entity(pop_entity).insert(ScrapCodeCultist::default());
                    }
                }
            }
        }
    }
}

pub fn execute_cult_sabotage(
    cultist_query: Query<&CurrentAction, With<ScrapCodeCultist>>,
    mut machine_query: Query<&mut Machine>,
    config: Option<Res<ScrapCodeConfig>>,
) {
    let default_config = ScrapCodeConfig::default();
    let config = config.as_deref().unwrap_or(&default_config);

    for action in cultist_query.iter() {
        if let CurrentAction::Sabotage(machine_ent) = action {
            if let Ok(mut machine) = machine_query.get_mut(*machine_ent) {
                machine.maintenance_debt += config.sabotage_debt_increase;
                if machine.maintenance_debt >= 100.0 {
                    machine.state = MachineState::Broken;
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn cultist_morale_aura(
    broken_machines: Query<(&Machine, &Transform)>,
    mut pops: Query<(&mut Morale, &Transform, Option<&mut ScrapCodeCultist>)>,
    config: Option<Res<ScrapCodeConfig>>,
) {
    let default_config = ScrapCodeConfig::default();
    let config = config.as_deref().unwrap_or(&default_config);

    for (mut morale, pop_transform, is_cultist) in pops.iter_mut() {
        if let Some(mut cultist) = is_cultist {
            let mut near_broken = false;
            for (machine, machine_transform) in broken_machines.iter() {
                if machine.state == MachineState::Broken {
                    if pop_transform.translation.distance(machine_transform.translation) < config.aura_distance {
                        near_broken = true;
                        break;
                    }
                }
            }

            if near_broken {
                if cultist.current_morale_bonus < config.max_morale_bonus {
                    morale.value += 5.0; // Minimal increase logic
                    cultist.current_morale_bonus += 5.0;
                }
            } else if cultist.current_morale_bonus > 0.0 {
                // Decay buffer when away from machines
                morale.value -= 1.0;
                cultist.current_morale_bonus -= 1.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrap_code_cult_formation() {
        // Arrange
        let mut app = App::new();

        let config = ScrapCodeConfig {
            cult_formation_chance: 1.0, // Force formation for test
            ..Default::default()
        };
        app.insert_resource(config);

        app.add_event::<MachineBreakdownEvent>()
           .add_event::<CultFormationEvent>()
           .add_systems(Update, process_scrap_code_revelations);

        let machine = app.world_mut().spawn((
            Machine { maintenance_debt: 100.0, state: MachineState::Broken },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let pop = app.world_mut().spawn((
            Pop,
            Morale { value: 40.0, ..Default::default() },
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        // Act
        app.world_mut().send_event(MachineBreakdownEvent { entity: machine });
        app.update();

        // Assert
        assert!(app.world().get::<ScrapCodeCultist>(pop).is_some(), "Pop near broken machine should have a chance to become a cultist");
    }

    #[test]
    fn test_cultist_sabotage() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, execute_cult_sabotage);

        let machine = app.world_mut().spawn((
            Machine { maintenance_debt: 0.0, state: MachineState::Working },
        )).id();

        app.world_mut().spawn((
            Pop,
            Morale { value: 80.0, ..Default::default() },
            ScrapCodeCultist::default(),
            CurrentAction::Sabotage(machine),
        ));

        // Act
        app.update();

        // Assert
        let machine_data = app.world().get::<Machine>(machine).unwrap();
        assert!(machine_data.maintenance_debt > 0.0, "Cultist should increase maintenance debt to break the machine");
    }

    #[test]
    fn test_cultist_morale_from_broken_machines() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, cultist_morale_aura);

        app.world_mut().spawn((
            Machine { maintenance_debt: 100.0, state: MachineState::Broken },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        let cultist = app.world_mut().spawn((
            Pop,
            Morale { value: 50.0, ..Default::default() },
            ScrapCodeCultist::default(),
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        let normal_pop = app.world_mut().spawn((
            Pop,
            Morale { value: 50.0, ..Default::default() },
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        // Act
        app.update();

        // Assert
        let cultist_morale = app.world().get::<Morale>(cultist).unwrap().value;
        let normal_morale = app.world().get::<Morale>(normal_pop).unwrap().value;

        assert!(cultist_morale > 50.0, "Cultist should gain morale from nearby broken machine");
        assert_eq!(normal_morale, 50.0, "Normal pop should not gain morale from broken machine");
    }

    #[test]
    fn test_cultist_morale_decay() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, cultist_morale_aura);

        // No machines nearby

        let cultist = app.world_mut().spawn((
            Pop,
            Morale { value: 55.0, ..Default::default() },
            ScrapCodeCultist { current_morale_bonus: 5.0 },
            Transform::from_xyz(100.0, 0.0, 0.0),
        )).id();

        // Act
        app.update();

        // Assert
        let cultist_morale = app.world().get::<Morale>(cultist).unwrap().value;
        let cultist_comp = app.world().get::<ScrapCodeCultist>(cultist).unwrap();

        assert_eq!(cultist_morale, 54.0, "Cultist should lose morale when away from broken machines");
        assert_eq!(cultist_comp.current_morale_bonus, 4.0);
    }

    #[test]
    fn test_cultist_ignores_working_machines() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, cultist_morale_aura);

        // A working machine nearby
        app.world_mut().spawn((
            Machine { maintenance_debt: 0.0, state: MachineState::Working },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        let cultist = app.world_mut().spawn((
            Pop,
            Morale { value: 50.0, ..Default::default() },
            ScrapCodeCultist::default(),
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        // Act
        app.update();

        // Assert
        let cultist_morale = app.world().get::<Morale>(cultist).unwrap().value;
        let cultist_comp = app.world().get::<ScrapCodeCultist>(cultist).unwrap();

        assert_eq!(cultist_morale, 50.0, "Cultist should NOT gain morale from working machines");
        assert_eq!(cultist_comp.current_morale_bonus, 0.0);
    }
}
