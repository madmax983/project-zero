use bevy_ecs::prelude::*;

/// Resource tracking the Scrapcode infection.
#[derive(Resource, Debug, Clone)]
pub struct Scrapcode {
    /// Whether the infection is currently active.
    pub active: bool,
    /// Cost multiplier (e.g., 1.5 for 50% increase).
    pub severity: f32,
    /// Duration of the infection in ticks.
    pub duration: u32,
}

impl Default for Scrapcode {
    fn default() -> Self {
        Self {
            active: false,
            severity: 1.0, // Default to 1.0 (no change) to avoid 0.0 cost bugs!
            duration: 0,
        }
    }
}

// --- Scrap-Code Cultists ---

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

#[derive(Component, PartialEq, Clone, Debug)]
pub enum MachineState {
    Working,
    Broken,
}

#[derive(Component)]
pub struct Machine {
    pub maintenance_debt: f32,
    pub state: MachineState,
}

/// Configuration for ScrapCode Cult mechanics.
#[derive(Resource)]
pub struct ScrapCodeConfig {
    /// Range within which a breakdown can cause revelation.
    pub revelation_range: f32,
    /// Range within which a broken machine provides morale to a cultist.
    pub aura_range: f32,
    /// The chance (0.0 - 1.0) a pop within range will become a cultist when a machine breaks down.
    pub formation_chance: f32,
    /// Amount of maintenance debt added per sabotage tick.
    pub sabotage_debt_per_tick: f32,
    /// Amount of morale gained by cultists near broken machines per tick.
    pub morale_bonus_per_tick: f32,
}

impl Default for ScrapCodeConfig {
    fn default() -> Self {
        Self {
            revelation_range: 5.0,
            aura_range: 10.0,
            formation_chance: 0.1, // 10% chance
            sabotage_debt_per_tick: 50.0,
            morale_bonus_per_tick: 5.0, // Simplified: actual implementation might decay over time.
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn process_scrap_code_revelations(
    mut events: EventReader<MachineBreakdownEvent>,
    mut commands: Commands,
    machine_query: Query<&bevy::prelude::Transform, With<Machine>>,
    pop_query: Query<(Entity, &bevy::prelude::Transform), (With<crate::layer1::pop::Pop>, Without<ScrapCodeCultist>)>,
    config: Option<Res<ScrapCodeConfig>>,
    mut event_writer: EventWriter<CultFormationEvent>,
) {
    let cfg_default = ScrapCodeConfig::default();
    let cfg = config.as_deref().unwrap_or(&cfg_default);
    let mut rng = rand::thread_rng();
    use rand::Rng;

    for event in events.read() {
        if let Ok(machine_transform) = machine_query.get(event.entity) {
            for (pop_entity, pop_transform) in pop_query.iter() {
                if machine_transform.translation.distance(pop_transform.translation) < cfg.revelation_range {
                    // Probabilistic chance to form cult
                    if rng.gen::<f32>() < cfg.formation_chance {
                        commands.entity(pop_entity).insert(ScrapCodeCultist);
                        event_writer.send(CultFormationEvent { pop: pop_entity });
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn execute_cult_sabotage(
    cultist_query: Query<(&crate::layer1::utility_ai::PopAction, &crate::layer1::actions::AssignedTo), With<ScrapCodeCultist>>,
    mut machine_query: Query<(Entity, &mut Machine)>,
    config: Option<Res<ScrapCodeConfig>>,
    mut breakdown_events: EventWriter<MachineBreakdownEvent>,
) {
    let cfg_default = ScrapCodeConfig::default();
    let cfg = config.as_deref().unwrap_or(&cfg_default);

    for (action, assigned_to) in cultist_query.iter() {
        if action.current == crate::layer1::utility_ai::ActionType::Vandalize {
            if let Ok((machine_entity, mut machine)) = machine_query.get_mut(assigned_to.entity) {
                machine.maintenance_debt += cfg.sabotage_debt_per_tick;
                if machine.maintenance_debt >= 100.0 && machine.state == MachineState::Working {
                    machine.state = MachineState::Broken;
                    breakdown_events.send(MachineBreakdownEvent { entity: machine_entity });
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn cultist_morale_aura(
    broken_machines: Query<(&bevy::prelude::Transform, &Machine)>,
    mut pops: Query<(&mut crate::layer1::morale::Morale, &bevy::prelude::Transform, Option<&ScrapCodeCultist>)>,
    config: Option<Res<ScrapCodeConfig>>,
) {
    let cfg_default = ScrapCodeConfig::default();
    let cfg = config.as_deref().unwrap_or(&cfg_default);

    for (mut morale, pop_transform, is_cultist) in pops.iter_mut() {
        if is_cultist.is_some() {
            for (machine_transform, machine) in broken_machines.iter() {
                if machine.state == MachineState::Broken && pop_transform.translation.distance(machine_transform.translation) < cfg.aura_range {
                    morale.value = (morale.value + cfg.morale_bonus_per_tick).min(100.0);
                    break;
                }
            }
        }
    }
}

/// Purges the scrapcode infection, resetting it to a dormant state.
pub fn perform_purge(world: &mut World) {
    if let Some(mut scrapcode) = world.get_resource_mut::<Scrapcode>() {
        scrapcode.active = false;
        scrapcode.severity = 1.0;
        scrapcode.duration = 0;

        if let Some(mut log) = world.get_resource_mut::<crate::shared::log::MessageLog>() {
            log.add("Scrapcode Purged!");
        }
    }
}

/// System to decay Scrapcode duration over time.
pub fn scrapcode_decay_system(mut scrapcode: ResMut<Scrapcode>) {
    if scrapcode.active && scrapcode.duration > 0 {
        scrapcode.duration = scrapcode.duration.saturating_sub(1);
        if scrapcode.duration == 0 {
            scrapcode.active = false;
            scrapcode.severity = 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{try_place_building, BuildingType, MaterialType};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup Terrain
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        // Setup Resources (Rich)
        world.insert_resource(ColonyResources {
            wood: 1000.0,
            stone: 1000.0,
            metal: 1000.0,
            ..Default::default()
        });
        // Setup OccupiedTiles
        world.insert_resource(crate::layer1::building::OccupiedTiles::default());
        // Setup BuildMode
        world.insert_resource(crate::layer1::building::BuildMode::default());
        // Setup MessageLog
        world.insert_resource(crate::shared::log::MessageLog::default());

        world
    }

    #[test]
    fn test_scrapcode_increases_building_cost() {
        let mut world = setup_world();

        // Activate Scrapcode
        world.insert_resource(Scrapcode {
            active: true,
            severity: 1.5, // 50% cost increase
            ..Default::default()
        });

        // Building Type: Housing
        let building = BuildingType::Housing;
        let normal_cost = building.cost(MaterialType::Wood).wood;

        // Attempt placement
        // Note: try_place_building subtracts cost. We check how much was subtracted.
        let initial_wood = world.resource::<ColonyResources>().wood;
        let success = try_place_building(&mut world, 5, 5, building);
        assert!(success);

        let final_wood = world.resource::<ColonyResources>().wood;
        let cost_paid = initial_wood - final_wood;

        // With 1.5 severity, cost should be increased
        // Note: Use a tolerance or integer math if needed, but for MVP float check:
        let expected_cost = (normal_cost * 1.5).ceil();
        assert!(
            (cost_paid - expected_cost).abs() < f32::EPSILON,
            "Scrapcode should increase cost by 50%. Paid: {}, Expected: {}",
            cost_paid,
            expected_cost
        );
    }

    #[test]
    fn test_purge_action_removes_scrapcode() {
        let mut world = setup_world();
        world.insert_resource(Scrapcode {
            active: true,
            severity: 1.0,
            duration: 100,
        });

        // Run Purge System (simulating action completion)
        perform_purge(&mut world);

        let scrapcode = world.resource::<Scrapcode>();
        assert!(!scrapcode.active, "Purge should deactivate Scrapcode");
        assert_eq!(scrapcode.severity, 1.0); // Reset to baseline or 0?
    }

    #[test]
    fn test_scrapcode_decay_system() {
        let mut world = setup_world();
        world.insert_resource(Scrapcode {
            active: true,
            severity: 1.5,
            duration: 1,
        });

        // Run decay logic directly (since we can't easily run system function with ResMut directly outside world context easily, or use system scheduler)
        // Actually, we can schedule it.
        let mut schedule = Schedule::default();
        schedule.add_systems(super::scrapcode_decay_system);
        schedule.run(&mut world);

        let scrapcode = world.resource::<Scrapcode>();
        assert!(!scrapcode.active, "Scrapcode should decay and deactivate");
        assert_eq!(scrapcode.duration, 0);
        assert_eq!(scrapcode.severity, 1.0);
    }

    use bevy::prelude::*;

    #[test]
    fn test_scrap_code_cult_formation() {
        // Arrange
        let mut app = App::new();
        app.add_event::<super::MachineBreakdownEvent>()
           .add_event::<super::CultFormationEvent>()
           .add_systems(Update, super::process_scrap_code_revelations);

        app.world_mut().insert_resource(super::ScrapCodeConfig {
            formation_chance: 1.0, // Force formation for test
            ..Default::default()
        });

        let machine = app.world_mut().spawn((
            super::Machine { maintenance_debt: 100.0, state: super::MachineState::Broken },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let pop = app.world_mut().spawn((
            crate::layer1::pop::Pop,
            crate::layer1::morale::Morale { value: 40.0, ..Default::default() },
            Transform::from_xyz(1.0, 0.0, 0.0), // Nearby pop
        )).id();

        // Act
        app.world_mut().send_event(super::MachineBreakdownEvent { entity: machine });
        app.update();

        // Assert
        assert!(app.world().get::<super::ScrapCodeCultist>(pop).is_some(), "Pop near broken machine should have a chance to become a cultist");
    }

    #[test]
    fn test_cultist_sabotage() {
        // Arrange
        let mut app = App::new();
        app.add_event::<super::MachineBreakdownEvent>()
           .add_systems(Update, super::execute_cult_sabotage);

        let machine = app.world_mut().spawn((
            super::Machine { maintenance_debt: 0.0, state: super::MachineState::Working },
        )).id();

        app.world_mut().spawn((
            crate::layer1::pop::Pop,
            crate::layer1::morale::Morale { value: 80.0, ..Default::default() },
            super::ScrapCodeCultist,
            crate::layer1::utility_ai::PopAction {
                current: crate::layer1::utility_ai::ActionType::Vandalize,
                ..Default::default()
            },
            crate::layer1::actions::AssignedTo {
                entity: machine,
                assignment_type: crate::layer1::utility_types::AssignmentType::FarmWorker, // Dummy assignment type for test compilation
            }
        ));

        // Act
        app.update();

        // Assert
        let machine_data = app.world().get::<super::Machine>(machine).unwrap();
        assert!(machine_data.maintenance_debt > 0.0, "Cultist should increase maintenance debt to break the machine");
    }

    #[test]
    fn test_cultist_morale_from_broken_machines() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, super::cultist_morale_aura);

        app.world_mut().spawn((
            super::Machine { maintenance_debt: 100.0, state: super::MachineState::Broken },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        app.world_mut().spawn((
            super::Machine { maintenance_debt: 0.0, state: super::MachineState::Working },
            Transform::from_xyz(0.0, 100.0, 0.0),
        ));

        let cultist_near_broken = app.world_mut().spawn((
            crate::layer1::pop::Pop,
            crate::layer1::morale::Morale { value: 50.0, ..Default::default() },
            super::ScrapCodeCultist,
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        let cultist_near_working = app.world_mut().spawn((
            crate::layer1::pop::Pop,
            crate::layer1::morale::Morale { value: 50.0, ..Default::default() },
            super::ScrapCodeCultist,
            Transform::from_xyz(1.0, 100.0, 0.0),
        )).id();

        let normal_pop = app.world_mut().spawn((
            crate::layer1::pop::Pop,
            crate::layer1::morale::Morale { value: 50.0, ..Default::default() },
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        // Act
        app.update();

        // Assert
        let cultist_broken_morale = app.world().get::<crate::layer1::morale::Morale>(cultist_near_broken).unwrap().value;
        let cultist_working_morale = app.world().get::<crate::layer1::morale::Morale>(cultist_near_working).unwrap().value;
        let normal_morale = app.world().get::<crate::layer1::morale::Morale>(normal_pop).unwrap().value;

        assert!(cultist_broken_morale > 50.0, "Cultist should gain morale from nearby broken machine");
        assert_eq!(cultist_working_morale, 50.0, "Cultist should not gain morale from nearby working machine");
        assert_eq!(normal_morale, 50.0, "Normal pop should not gain morale from broken machine");
    }

    #[test]
    fn test_execute_cult_sabotage_emits_breakdown() {
        // Arrange
        let mut app = App::new();
        app.add_event::<super::MachineBreakdownEvent>()
           .add_systems(Update, super::execute_cult_sabotage);

        let machine = app.world_mut().spawn((
            super::Machine { maintenance_debt: 60.0, state: super::MachineState::Working },
        )).id();

        app.world_mut().spawn((
            crate::layer1::pop::Pop,
            super::ScrapCodeCultist,
            crate::layer1::utility_ai::PopAction {
                current: crate::layer1::utility_ai::ActionType::Vandalize,
                ..Default::default()
            },
            crate::layer1::actions::AssignedTo {
                entity: machine,
                assignment_type: crate::layer1::utility_types::AssignmentType::FarmWorker,
            }
        ));

        // Act
        app.update();

        // Assert
        let machine_data = app.world().get::<super::Machine>(machine).unwrap();
        assert_eq!(machine_data.state, super::MachineState::Broken, "Machine should be broken now");

        let events = app.world().resource::<Events<super::MachineBreakdownEvent>>();
        assert_eq!(events.get_cursor().len(events), 1, "Should have emitted exactly 1 breakdown event");

        // Act again to ensure no redundant events
        app.update();
        let events = app.world().resource::<Events<super::MachineBreakdownEvent>>();
        assert_eq!(events.get_cursor().len(events), 1, "Should not emit again if already broken");
    }
}
