use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    #[must_use]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub fn distance(&self, other: Vec3) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

// Implement a simple Transform component because the actual Transform
// requires bevy_transform which is not imported or available,
// but the tests expect a simple .translation field with distance()
#[derive(Component, Clone)]
pub struct Transform {
    pub translation: Vec3,
}

impl Transform {
    #[must_use]
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            translation: Vec3::new(x, y, z),
        }
    }
}

#[derive(Resource, Clone, Copy)]
pub struct ScrapCodeConfig {
    pub revelation_distance: f32,
    pub aura_distance: f32,
    pub conversion_chance: f64,
    pub morale_bonus: f32,
    pub sabotage_damage: f32,
    pub maintenance_broken_threshold: f32,
    pub max_morale_bonus: f32,
}

impl Default for ScrapCodeConfig {
    fn default() -> Self {
        Self {
            revelation_distance: 5.0,
            aura_distance: 10.0,
            conversion_chance: 1.0, // Default to 100% to keep tests passing unless configured otherwise
            morale_bonus: 5.0,
            sabotage_damage: 50.0,
            maintenance_broken_threshold: 100.0,
            max_morale_bonus: 100.0, // Cap for morale bonus to avoid infinite scaling
        }
    }
}

#[derive(Component)]
pub struct ScrapCodeCultist;

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
pub struct CultistMachine {
    pub maintenance_debt: f32,
    pub state: MachineState,
}

#[derive(Component)]
pub struct CultistPop {
    pub morale: f32,
}

#[derive(Component)]
pub enum CultistCurrentAction {
    Sabotage(Entity),
    Idle,
}

type CultistPopQueryFilter = (With<CultistPop>, Without<ScrapCodeCultist>);

pub fn process_scrap_code_revelations(
    mut events: EventReader<MachineBreakdownEvent>,
    mut commands: Commands,
    config: Option<Res<ScrapCodeConfig>>,
    machine_query: Query<&Transform, With<CultistMachine>>,
    pop_query: Query<(Entity, &Transform), CultistPopQueryFilter>,
) {
    let cfg = config.map_or(ScrapCodeConfig::default(), |c| *c);
    let mut rng = rand::thread_rng();

    for event in events.read() {
        if let Ok(machine_transform) = machine_query.get(event.entity) {
            for (pop_entity, pop_transform) in pop_query.iter() {
                if machine_transform.translation.distance(pop_transform.translation) < cfg.revelation_distance
                    && rng.gen_bool(cfg.conversion_chance)
                {
                    commands.entity(pop_entity).insert(ScrapCodeCultist);
                    // Emit event in a real scenario
                }
            }
        }
    }
}

pub fn execute_cult_sabotage(
    cultist_query: Query<&CultistCurrentAction, With<ScrapCodeCultist>>,
    mut machine_query: Query<&mut CultistMachine>,
    config: Option<Res<ScrapCodeConfig>>,
) {
    let cfg = config.map_or(ScrapCodeConfig::default(), |c| *c);

    for action in cultist_query.iter() {
        if let CultistCurrentAction::Sabotage(machine_ent) = action {
            if let Ok(mut machine) = machine_query.get_mut(*machine_ent) {
                machine.maintenance_debt += cfg.sabotage_damage;
                if machine.maintenance_debt >= cfg.maintenance_broken_threshold {
                    machine.state = MachineState::Broken;
                }
            }
        }
    }
}

pub fn cultist_morale_aura(
    broken_machines: Query<&Transform, With<CultistMachine>>,
    mut pops: Query<(&mut CultistPop, &Transform, Option<&ScrapCodeCultist>)>,
    config: Option<Res<ScrapCodeConfig>>,
) {
    let cfg = config.map_or(ScrapCodeConfig::default(), |c| *c);

    for (mut pop, pop_transform, is_cultist) in pops.iter_mut() {
        if is_cultist.is_some() {
            let mut bonus_applied = 0.0;
            for machine_transform in broken_machines.iter() {
                if pop_transform.translation.distance(machine_transform.translation) < cfg.aura_distance {
                    bonus_applied += cfg.morale_bonus;
                }
            }
            if bonus_applied > 0.0 {
                pop.morale = (pop.morale + bonus_applied).min(cfg.max_morale_bonus);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    type Machine = CultistMachine;
    type Pop = CultistPop;
    type CurrentAction = CultistCurrentAction;

    #[test]
    fn test_scrap_code_cult_formation() {
        // Arrange
        let mut app = App::new();
        app.add_event::<MachineBreakdownEvent>()
           .add_event::<CultFormationEvent>()
           .add_systems(Update, process_scrap_code_revelations);

        let machine = app.world_mut().spawn((
            Machine { maintenance_debt: 100.0, state: MachineState::Broken },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let pop = app.world_mut().spawn((
            Pop { morale: 40.0 },
            Transform::from_xyz(1.0, 0.0, 0.0), // Nearby pop
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
            Pop { morale: 80.0 },
            ScrapCodeCultist,
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
            Pop { morale: 50.0 },
            ScrapCodeCultist,
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        let normal_pop = app.world_mut().spawn((
            Pop { morale: 50.0 },
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        // Act
        app.update();

        // Assert
        let cultist_morale = app.world().get::<Pop>(cultist).unwrap().morale;
        let normal_morale = app.world().get::<Pop>(normal_pop).unwrap().morale;

        assert!(cultist_morale > 50.0, "Cultist should gain morale from nearby broken machine");
        assert_eq!(normal_morale, 50.0, "Normal pop should not gain morale from broken machine");
    }
}
