use crate::layer1::building::Building;
use crate::layer1::building::BuildingType;
use crate::layer1::control::{DoorControl, DoorState};
use crate::layer1::events::GameOverEvent;
use crate::layer1::map::GridPosition;
use crate::layer1::morale::Morale;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;
use bevy_time::{Timer, TimerMode};

#[derive(Component)]
pub struct SymbiontInfection {
    pub spread_timer: Timer,
}

impl Default for SymbiontInfection {
    fn default() -> Self {
        Self {
            spread_timer: Timer::from_seconds(100.0, TimerMode::Repeating), // Placeholder default duration
        }
    }
}

#[derive(Component)]
pub struct SymbiontFactionMember;

// System to apply buffs to infected pops
pub fn apply_symbiont_buffs(mut query: Query<&mut Morale, With<SymbiontInfection>>) {
    for mut morale in query.iter_mut() {
        morale.value = 100.0; // Euphoria
    }
}

// System to spread infection
#[allow(clippy::type_complexity)]
pub fn spread_symbiont_spores(
    mut commands: Commands,
    time: Res<crate::shared::time::SimulationTime>,
    mut infected_query: Query<(&GridPosition, &mut SymbiontInfection)>,
    uninfected_query: Query<(Entity, &GridPosition), (With<Pop>, Without<SymbiontInfection>)>,
) {
    for (infected_pos, mut infection) in infected_query.iter_mut() {
        let speed_mult = match time.speed {
            crate::shared::time::SimSpeed::Paused => 0.0,
            crate::shared::time::SimSpeed::Normal => 1.0,
            crate::shared::time::SimSpeed::Fast => 3.0,
            crate::shared::time::SimSpeed::Faster => 5.0,
        };
        let delta = 0.1 * speed_mult; // 100ms per simulated tick is a standard proxy in SCALE
        infection
            .spread_timer
            .tick(std::time::Duration::from_secs_f32(delta));

        if infection.spread_timer.just_finished() {
            // Find a nearby pop
            for (uninfected_entity, uninfected_pos) in uninfected_query.iter() {
                let dist = ((infected_pos.x as f32 - uninfected_pos.x as f32).powi(2)
                    + (infected_pos.y as f32 - uninfected_pos.y as f32).powi(2))
                .sqrt();
                if dist <= 5.0 {
                    commands
                        .entity(uninfected_entity)
                        .insert(SymbiontInfection::default());
                    commands
                        .entity(uninfected_entity)
                        .insert(SymbiontFactionMember);
                    break; // Infect one at a time
                }
            }
        }
    }
}

// System to check critical mass
pub fn check_symbiont_critical_mass(
    infected_query: Query<(), With<SymbiontFactionMember>>,
    total_pops_query: Query<(), With<Pop>>,
    mut airlock_query: Query<(&mut DoorControl, &Building)>,
    mut game_over_events: EventWriter<GameOverEvent>,
) {
    let infected_count = infected_query.iter().count() as f32;
    let total_count = total_pops_query.iter().count() as f32;

    if total_count > 0.0 && (infected_count / total_count) >= 0.80 {
        // Open all airlocks
        for (mut door_control, building) in airlock_query.iter_mut() {
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

    use crate::layer1::execution::general_work::calculate_work_amount;
    use crate::layer1::factions::{update_faction_membership_system, FactionId, FactionMember};
    use crate::layer1::map::GridPosition;
    use crate::layer1::skills::Skills;
    use crate::layer1::DesignationType;
    use crate::shared::time::SimulationTime;

    // 1. Spore Infection Boosts
    #[test]
    fn test_spore_infection_grants_buffs() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                Morale {
                    value: 50.0,

                    ..Default::default()
                },
                SymbiontInfection::default(),
                Skills::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_symbiont_buffs);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop).unwrap();
        assert_eq!(morale.value, 100.0, "Morale is locked at max");

        let work_amount =
            calculate_work_amount(&world, pop, DesignationType::Mine, None, 100.0, 1.0, 1.0);

        world.entity_mut(pop).remove::<SymbiontInfection>();
        let uninfected_work_amount =
            calculate_work_amount(&world, pop, DesignationType::Mine, None, 100.0, 1.0, 1.0);

        assert!(
            work_amount > uninfected_work_amount * 1.5,
            "WorkSpeed multiplier applied"
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
                SymbiontInfection::default(),
                SymbiontFactionMember,
                Skills::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_faction_membership_system);
        schedule.run(&mut world);

        let faction = world.get::<FactionMember>(pop).unwrap();
        assert_eq!(
            faction.faction_id,
            Some(FactionId::Symbiont),
            "Pop is removed from old factions, added to Symbiont faction"
        );
    }

    // 3. Spore Spreading Behavior
    #[test]
    fn test_symbiont_faction_spreads_infection() {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            speed: crate::shared::time::SimSpeed::Normal,
            ..Default::default()
        });

        // 1 Infected Pop
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            SymbiontInfection {
                spread_timer: bevy_time::Timer::from_seconds(0.05, bevy_time::TimerMode::Repeating),
            },
        ));

        // 1 Uninfected Pop in same room
        let uninfected = world.spawn((Pop, GridPosition { x: 6, y: 5 })).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(spread_symbiont_spores);
        schedule.run(&mut world);

        assert!(
            world.get::<SymbiontInfection>(uninfected).is_some(),
            "Uninfected Pop becomes infected over time"
        );
    }

    // 4. Critical Mass Trigger (The End)
    #[test]
    fn test_symbiont_critical_mass_opens_airlocks() {
        let mut world = World::new();
        world.insert_resource(Events::<GameOverEvent>::default());

        // Colony where > 80% of Pops are in Symbiont Faction
        for _ in 0..9 {
            world.spawn((Pop, SymbiontFactionMember));
        }
        world.spawn((Pop,));

        // Plus sealed airlocks
        let airlock = world
            .spawn((
                DoorControl {
                    state: DoorState::Locked,
                },
                Building {
                    building_type: BuildingType::Airlock,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(check_symbiont_critical_mass);
        schedule.run(&mut world);

        let airlock_control = world.get::<DoorControl>(airlock).unwrap();
        assert_eq!(
            airlock_control.state,
            DoorState::Open,
            "Airlock states changed to Open"
        );

        let events = world.resource::<Events<GameOverEvent>>();
        let mut reader = events.get_cursor();
        let events_list: Vec<_> = reader.read(events).collect();

        assert_eq!(events_list.len(), 1);
        assert_eq!(
            events_list[0],
            &GameOverEvent::SymbiontAssimilation,
            "game over state triggered"
        );
    }
}
