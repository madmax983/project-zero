use crate::layer1::building::{Building, BuildingType};
use crate::layer1::control::{DoorControl, DoorState};
use crate::layer1::factions::{FactionId, FactionMember};
use crate::layer1::gastronomy::WorkSpeedBuff;
use crate::layer1::map::GridPosition;
use crate::layer1::morale::Morale;
use crate::layer1::pop::Pop;
use crate::shared::state::GameOverEvent;
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct SymbiontInfection {
    pub spread_timer: u32,
}

#[derive(Component)]
pub struct SymbiontFactionMember;

// System to apply buffs to infected pops
pub fn apply_symbiont_buffs(
    mut query: Query<(&mut Morale, Option<&mut WorkSpeedBuff>), With<SymbiontInfection>>,
) {
    for (mut morale, work_speed) in query.iter_mut() {
        morale.value = 1.0; // Euphoria

        if let Some(mut ws) = work_speed {
            ws.multiplier = 2.0; // Overclocked
            ws.duration = 200; // maintain duration
        }
    }
}

pub fn ensure_work_speed_buff(
    query: Query<Entity, (With<SymbiontInfection>, Without<WorkSpeedBuff>)>,
    mut commands: Commands,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(WorkSpeedBuff {
            multiplier: 2.0,
            duration: 200,
        });
    }
}

pub fn assign_symbiont_faction(
    mut query: Query<(Entity, &mut FactionMember), Added<SymbiontInfection>>,
    mut commands: Commands,
) {
    for (entity, mut member) in query.iter_mut() {
        member.faction_id = Some(FactionId::Symbiont);
        commands.entity(entity).insert(SymbiontFactionMember);
    }
}

// System to spread infection
pub fn spread_symbiont_spores(
    mut infected_query: Query<(&mut SymbiontInfection, &GridPosition)>,
    uninfected_query: Query<(Entity, &GridPosition), (With<Pop>, Without<SymbiontInfection>)>,
    mut commands: Commands,
) {
    let uninfected_positions: Vec<(Entity, GridPosition)> =
        uninfected_query.iter().map(|(e, pos)| (e, *pos)).collect();

    for (mut infection, pos) in infected_query.iter_mut() {
        if infection.spread_timer > 0 {
            infection.spread_timer -= 1;
        }

        if infection.spread_timer == 0 {
            // Infect a nearby pop
            for (uninfected_entity, uninfected_pos) in &uninfected_positions {
                if ((pos.x as f32 - uninfected_pos.x as f32).powi(2)
                    + (pos.y as f32 - uninfected_pos.y as f32).powi(2))
                .sqrt()
                    <= 5.0
                {
                    commands
                        .entity(*uninfected_entity)
                        .insert(SymbiontInfection {
                            spread_timer: rand::thread_rng().gen_range(100..500),
                        });
                    break;
                }
            }
            infection.spread_timer = rand::thread_rng().gen_range(100..500);
        }
    }
}

// System to check critical mass
pub fn check_symbiont_critical_mass(
    infected_query: Query<(), With<SymbiontFactionMember>>,
    total_pops_query: Query<(), With<Pop>>,
    mut airlock_query: Query<(&Building, &mut DoorControl)>,
    mut game_over_events: EventWriter<GameOverEvent>,
) {
    let infected_count = infected_query.iter().count() as f32;
    let total_count = total_pops_query.iter().count() as f32;

    if total_count > 0.0 && (infected_count / total_count) >= 0.80 {
        // Open all airlocks
        for (building, mut door_control) in airlock_query.iter_mut() {
            if building.building_type == BuildingType::Airlock {
                door_control.state = DoorState::Open;
            }
        }
        // Fire Game Over
        game_over_events.send(GameOverEvent::SymbiontAssimilation);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 1. Spore Infection Boosts
    #[test]
    fn test_spore_infection_grants_buffs() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                },
                SymbiontInfection { spread_timer: 100 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems((ensure_work_speed_buff, apply_symbiont_buffs).chain());
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();
        assert_eq!(morale.value, 1.0, "Morale should be maxed (1.0)");

        let work_speed = world.get::<WorkSpeedBuff>(pop).unwrap();
        assert_eq!(
            work_speed.multiplier, 2.0,
            "WorkSpeed multiplier should be 2.0"
        );
    }

    // 2. Faction Assignment
    #[test]
    fn test_infected_pops_join_symbiont_faction() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::MinersGuild),
                },
                SymbiontInfection { spread_timer: 100 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(assign_symbiont_faction);
        schedule.run(&mut world);

        let member = world.get::<FactionMember>(pop).unwrap();
        assert_eq!(
            member.faction_id,
            Some(FactionId::Symbiont),
            "Pop should be assigned to Symbiont faction"
        );
        assert!(
            world.get::<SymbiontFactionMember>(pop).is_some(),
            "Pop should have SymbiontFactionMember marker"
        );
    }

    // 3. Spore Spreading Behavior
    #[test]
    fn test_symbiont_faction_spreads_infection() {
        let mut world = World::new();
        let infected = world
            .spawn((
                Pop,
                SymbiontInfection { spread_timer: 1 },
                SymbiontFactionMember,
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let uninfected = world.spawn((Pop, GridPosition { x: 5, y: 6 })).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(spread_symbiont_spores);
        schedule.run(&mut world);

        let infected_comp = world.get::<SymbiontInfection>(infected).unwrap();
        // The timer resets to a random range 100..500, but since spread_timer started at 1,
        // it decremented to 0, fired infection, and was reset to gen_range(100..500), so it should be >= 100.
        assert!(
            infected_comp.spread_timer >= 100,
            "Timer should reset to a positive number"
        );

        assert!(
            world.get::<SymbiontInfection>(uninfected).is_some(),
            "Uninfected pop should be infected"
        );
    }

    // 4. Critical Mass Trigger (The End)
    #[test]
    fn test_symbiont_critical_mass_opens_airlocks() {
        let mut world = World::new();
        world.init_resource::<Events<GameOverEvent>>();

        // Spawn 4 infected pops (80%)
        for _ in 0..4 {
            world.spawn((Pop, SymbiontFactionMember));
        }

        // Spawn 1 uninfected pop (20%)
        world.spawn(Pop);

        let airlock = world
            .spawn((
                Building {
                    building_type: BuildingType::Airlock,
                },
                DoorControl {
                    state: DoorState::Auto,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(check_symbiont_critical_mass);
        schedule.run(&mut world);

        let door_control = world.get::<DoorControl>(airlock).unwrap();
        assert_eq!(
            door_control.state,
            DoorState::Open,
            "Airlock should be forced open"
        );

        let events = world.resource::<Events<GameOverEvent>>();
        let mut reader = events.get_cursor();
        let evts: Vec<_> = reader.read(events).collect();
        assert_eq!(evts.len(), 1, "Should fire exactly 1 GameOverEvent");
        assert_eq!(
            *evts[0],
            GameOverEvent::SymbiontAssimilation,
            "Event should be SymbiontAssimilation"
        );
    }
}
