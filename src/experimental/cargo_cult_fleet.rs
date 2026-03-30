use crate::layer1::items::ItemType;
use crate::layer1::logistics::orbital_drop::OrbitalDropEvent;
use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::terrain::TerrainGrid;
use crate::layer2::system::OrbitalBody;
use crate::shared::log::MessageLog;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Marker component for the automated supply dreadnought.
#[derive(Component, Debug, Clone)]
pub struct CargoCultFleet {
    /// How many total deliveries this fleet has made.
    pub deliveries: u32,
}

/// The resource representing the open connection/demand from the fleet to the colony.
#[derive(Resource, Debug, Clone)]
pub struct CargoCultTether {
    /// What resource the fleet currently demands.
    pub demand_type: TetherDemandType,
    /// How much of the resource is demanded.
    pub demand_amount: f32,
    /// How much has been fed into the tether.
    pub current_fed: f32,
    /// Whether the tether is currently active.
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TetherDemandType {
    Water, // We'll map this to an arbitrary drain on the simulation, or just use Food as a proxy if Water isn't tracked. We'll track 'Water' conceptually but drain food for now if needed, or better, let's use actual resources: Wood, Stone, Metal.
    Wood,
    Stone,
    Metal,
}

impl Default for CargoCultTether {
    fn default() -> Self {
        Self {
            demand_type: TetherDemandType::Wood,
            demand_amount: 500.0,
            current_fed: 0.0,
            active: false,
        }
    }
}

/// A component marking a Pop as part of the Cult that worships the fleet.
#[derive(Component, Debug, Clone)]
pub struct CargoCultCultist;

/// System that detects if a CargoCultFleet should arrive (or spawns one randomly if none exist).
pub fn spawn_cargo_cult_fleet_system(
    mut commands: Commands,
    fleet_query: Query<&CargoCultFleet>,
    body_query: Query<Entity, With<OrbitalBody>>,
    tether: Option<ResMut<CargoCultTether>>,
    message_log: Option<ResMut<MessageLog>>,
    time: Option<Res<SimulationTime>>,
) {
    // Only spawn occasionally, e.g. tick 100 or something if not spawned.
    let current_tick = time.map(|t| t.tick).unwrap_or(0);
    if current_tick < 100 || fleet_query.iter().count() > 0 {
        return;
    }

    // Pick an orbital body to orbit
    let mut rng = rand::thread_rng();
    let bodies: Vec<Entity> = body_query.iter().collect();
    if bodies.is_empty() {
        return;
    }
    let target_body = bodies[rng.gen_range(0..bodies.len())];

    // Spawn the fleet
    commands.spawn((
        CargoCultFleet { deliveries: 0 },
        crate::layer2::system::Orbit {
            parent: target_body,
            radius: 8.0,
            speed: 0.02,
            angle: 0.0,
        },
        crate::layer2::system::OrbitalBody {
            name: "Ancient Supply Dreadnought".to_string(),
            radius: 0.5,
            color: ratatui::style::Color::Magenta,
            char: 'D',
        },
    ));

    if let Some(mut t) = tether {
        t.active = true;
        t.current_fed = 0.0;
        let demands = [
            TetherDemandType::Wood,
            TetherDemandType::Stone,
            TetherDemandType::Metal,
        ];
        t.demand_type = demands[rng.gen_range(0..demands.len())];
        t.demand_amount = rng.gen_range(300.0..1000.0);

        if let Some(mut log) = message_log {
            log.add(format!(
                "An ancient Dreadnought arrived in orbit. It opened a Tether demanding {}!",
                match t.demand_type {
                    TetherDemandType::Water => "Water",
                    TetherDemandType::Wood => "Wood",
                    TetherDemandType::Stone => "Stone",
                    TetherDemandType::Metal => "Metal",
                }
            ));
        }
    } else {
        // Initialize tether if it didn't exist
        let mut t = CargoCultTether {
            active: true,
            current_fed: 0.0,
            demand_amount: rng.gen_range(300.0..1000.0),
            ..Default::default()
        };
        let demands = [
            TetherDemandType::Wood,
            TetherDemandType::Stone,
            TetherDemandType::Metal,
        ];
        t.demand_type = demands[rng.gen_range(0..demands.len())];

        commands.insert_resource(t);

        if let Some(mut log) = message_log {
            log.add("An ancient Dreadnought arrived in orbit and opened a Tether!");
        }
    }
}

/// System that drains colony resources to feed the active tether and triggers rewards.
pub fn feed_cargo_cult_tether_system(
    tether_res: Option<ResMut<CargoCultTether>>,
    colony_resources: Option<ResMut<ColonyResources>>,
    mut drop_events: EventWriter<OrbitalDropEvent>,
    message_log: Option<ResMut<MessageLog>>,
    terrain_grid: Option<Res<TerrainGrid>>,
    mut fleet_query: Query<&mut CargoCultFleet>,
) {
    let Some(mut tether) = tether_res else { return };
    if !tether.active {
        return;
    }

    let Some(mut resources) = colony_resources else {
        return;
    };

    let feed_rate = 5.0; // Amount fed per tick
    let mut fed_this_tick = 0.0;

    match tether.demand_type {
        TetherDemandType::Wood => {
            if resources.wood >= feed_rate {
                resources.wood -= feed_rate;
                fed_this_tick = feed_rate;
            } else if resources.wood > 0.0 {
                fed_this_tick = resources.wood;
                resources.wood = 0.0;
            }
        }
        TetherDemandType::Stone => {
            if resources.stone >= feed_rate {
                resources.stone -= feed_rate;
                fed_this_tick = feed_rate;
            } else if resources.stone > 0.0 {
                fed_this_tick = resources.stone;
                resources.stone = 0.0;
            }
        }
        TetherDemandType::Metal => {
            if resources.metal >= feed_rate {
                resources.metal -= feed_rate;
                fed_this_tick = feed_rate;
            } else if resources.metal > 0.0 {
                fed_this_tick = resources.metal;
                resources.metal = 0.0;
            }
        }
        _ => {}
    }

    tether.current_fed += fed_this_tick;

    if tether.current_fed >= tether.demand_amount {
        tether.active = false; // Demand met!

        if let Some(mut log) = message_log {
            log.add("The ancient Dreadnought's demands were met. It is dropping something...");
        }

        for mut fleet in fleet_query.iter_mut() {
            fleet.deliveries += 1;
        }

        // Trigger OrbitalDropEvent with random rewards
        let mut rng = rand::thread_rng();
        let center_x = if let Some(ref grid) = terrain_grid {
            grid.width / 2
        } else {
            25
        };
        let center_y = if let Some(ref grid) = terrain_grid {
            grid.height / 2
        } else {
            25
        };

        let possible_rewards = [ItemType::LuxuryMeal,
            ItemType::AlienMeatA,
            ItemType::AlienMeatB,
            ItemType::Scrap,
            ItemType::Tool,
            ItemType::Prosthetic,
            ItemType::GlowMushroom,
            ItemType::ShadowCrystal];

        let mut reward_items = Vec::new();
        let num_items = rng.gen_range(3..10);
        for _ in 0..num_items {
            let item = possible_rewards[rng.gen_range(0..possible_rewards.len())];
            reward_items.push(item);
        }

        drop_events.send(OrbitalDropEvent {
            target: GridPosition {
                x: center_x as i32,
                y: center_y as i32,
            },
            items: reward_items,
            scatter_radius: 5,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::logistics::orbital_drop::OrbitalDropEvent;
    use crate::layer2::system::OrbitalBody;
    use crate::shared::log::MessageLog;
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_spawn_cargo_cult_fleet() {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 150,
            ..Default::default()
        });
        world.insert_resource(MessageLog::default());

        // Need an orbital body to orbit
        world.spawn(OrbitalBody::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(spawn_cargo_cult_fleet_system);
        schedule.run(&mut world);

        // Verify fleet spawned
        let fleet_count = world.query::<&CargoCultFleet>().iter(&world).count();
        assert_eq!(fleet_count, 1);

        // Verify tether created
        let tether = world.resource::<CargoCultTether>();
        assert!(tether.active);
        assert_eq!(tether.current_fed, 0.0);
        assert!(tether.demand_amount > 0.0);
    }

    #[test]
    fn test_feed_cargo_cult_tether_system() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            wood: 100.0,
            ..Default::default()
        });
        world.insert_resource(CargoCultTether {
            demand_type: TetherDemandType::Wood,
            demand_amount: 10.0,
            current_fed: 0.0,
            active: true,
        });
        world.init_resource::<Events<OrbitalDropEvent>>();
        world.insert_resource(MessageLog::default());

        world.spawn(CargoCultFleet { deliveries: 0 });

        let mut schedule = Schedule::default();
        schedule.add_systems(feed_cargo_cult_tether_system);

        // Tick 1
        schedule.run(&mut world);

        {
            let resources = world.resource::<ColonyResources>();
            assert_eq!(resources.wood, 95.0); // 5.0 fed
            let tether = world.resource::<CargoCultTether>();
            assert_eq!(tether.current_fed, 5.0);
            assert!(tether.active);
        }

        // Tick 2
        schedule.run(&mut world);

        {
            let resources = world.resource::<ColonyResources>();
            assert_eq!(resources.wood, 90.0); // 5.0 fed
            let tether = world.resource::<CargoCultTether>();
            assert_eq!(tether.current_fed, 10.0);
            assert!(!tether.active); // Demand met
        }

        // Verify drop event
        let events = world.resource::<Events<OrbitalDropEvent>>();
        assert_eq!(events.len(), 1);

        // Verify fleet deliveries updated
        let fleet = world.query::<&CargoCultFleet>().single(&world);
        assert_eq!(fleet.deliveries, 1);
    }
}
