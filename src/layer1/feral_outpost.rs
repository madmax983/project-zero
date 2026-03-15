use crate::layer1::building::{Building, BuildingType};
use crate::layer1::culture::CulturalTag;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct FeralOutpost;

#[derive(Component)]
pub struct SubFaction {
    pub name: String,
}

impl SubFaction {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[derive(Component)]
pub struct SubFactionMember {
    pub faction_entity: Entity,
}

#[derive(Component)]
pub struct FringeExposure {
    pub amount: f32,
}

impl Default for FringeExposure {
    fn default() -> Self {
        Self { amount: 0.0 }
    }
}

pub const FRINGE_EXPOSURE_THRESHOLD: f32 = 100.0;
pub const DISTANCE_THRESHOLD: u32 = 50;
pub const FRINGE_EXPOSURE_RATE: f32 = 1.0;

#[allow(clippy::type_complexity)]
pub fn update_cultural_drift_system(
    mut commands: Commands,
    cc_query: Query<(&GridPosition, &Building)>,
    mut pops_query: Query<
        (
            Entity,
            &GridPosition,
            &mut CulturalTag,
            Option<&mut FringeExposure>,
        ),
        With<Pop>,
    >,
) {
    let mut command_center_pos = None;
    for (pos, building) in cc_query.iter() {
        if building.building_type == BuildingType::CommandCenter {
            command_center_pos = Some(*pos);
            break; // Assume 1 core for MVP
        }
    }

    if let Some(core_pos) = command_center_pos {
        for (entity, pos, mut tags, exposure_opt) in pops_query.iter_mut() {
            let distance = core_pos.distance_chebyshev(*pos);
            if distance > DISTANCE_THRESHOLD {
                // Threshold distance
                if let Some(mut exposure) = exposure_opt {
                    exposure.amount += FRINGE_EXPOSURE_RATE;
                    if exposure.amount >= FRINGE_EXPOSURE_THRESHOLD && !tags.has_tag("Fringe") {
                        tags.add_tag("Fringe");
                    }
                } else {
                    commands.entity(entity).insert(FringeExposure {
                        amount: FRINGE_EXPOSURE_RATE,
                    });
                }
            } else if let Some(mut exposure) = exposure_opt {
                exposure.amount = (exposure.amount - FRINGE_EXPOSURE_RATE).max(0.0);
                if exposure.amount == 0.0 {
                    commands.entity(entity).remove::<FringeExposure>();
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn form_feral_outposts_system(
    mut commands: Commands,
    pops_query: Query<
        (Entity, &GridPosition, &CulturalTag),
        (With<Pop>, Without<SubFactionMember>),
    >,
) {
    // Collect fringe pops and their positions
    let mut fringe_pops = Vec::new();
    for (entity, pos, tags) in pops_query.iter() {
        if tags.has_tag("Fringe") {
            fringe_pops.push((entity, *pos));
        }
    }

    // Simplified clustering: if we have >= 3 fringe pops unassigned, form one faction
    // We should ensure they are somewhat close together.
    if fringe_pops.len() >= 3 {
        // Find a cluster
        let mut cluster = Vec::new();
        let mut assigned = vec![false; fringe_pops.len()];
        for i in 0..fringe_pops.len() {
            if assigned[i] {
                continue;
            }

            let mut potential_cluster = Vec::new();
            potential_cluster.push(fringe_pops[i].0);

            let mut potential_assigned = vec![false; fringe_pops.len()];
            potential_assigned[i] = true;

            for j in (i + 1)..fringe_pops.len() {
                if !assigned[j] && fringe_pops[i].1.distance_chebyshev(fringe_pops[j].1) <= 10 {
                    potential_cluster.push(fringe_pops[j].0);
                    potential_assigned[j] = true;
                }
            }

            if potential_cluster.len() >= 3 {
                cluster = potential_cluster;
                for j in 0..fringe_pops.len() {
                    if potential_assigned[j] {
                        assigned[j] = true;
                    }
                }
                break;
            }
        }

        if cluster.len() >= 3 {
            let faction_entity = commands
                .spawn((SubFaction::new("Feral Outpost"), FeralOutpost))
                .id();

            for pop_entity in cluster {
                commands
                    .entity(pop_entity)
                    .insert(SubFactionMember { faction_entity });
            }
        }
    }
}

pub fn can_assign_home_zone(
    faction_member_opt: Option<&SubFactionMember>,
    feral_outpost_query: &Query<&FeralOutpost>,
) -> bool {
    if let Some(faction_member) = faction_member_opt {
        if feral_outpost_query.get(faction_member.faction_entity).is_ok() {
            // Feral pops refuse orders far from their current home/outpost
            return false;
        }
    }

    // Default success for MVP
    true
}

pub fn assign_home_zone(
    world: &mut World,
    pop_entity: Entity,
    _target_pos: GridPosition,
) -> Result<(), &'static str> {
    if let Some(faction_member) = world.get::<SubFactionMember>(pop_entity) {
        if world
            .get::<FeralOutpost>(faction_member.faction_entity)
            .is_some()
        {
            // Feral pops refuse orders far from their current home/outpost
            // We could check distance, but for now we simply refuse relocation to core (or anywhere far).
            return Err("Pop is feral and refuses relocation");
        }
    }

    // Default success for MVP
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::culture::CulturalTag;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_pop_accumulates_fringe_tag_when_far_from_core() {
        let mut world = World::new();

        // Core at (10, 10)
        world.spawn((
            Building {
                building_type: BuildingType::CommandCenter,
                ..Default::default()
            },
            GridPosition { x: 10, y: 10 },
        ));

        // Pop at (80, 80) - distance is max(|80-10|, |80-10|) = 70 > 50
        let pop_entity = world
            .spawn((Pop, GridPosition { x: 80, y: 80 }, CulturalTag::default()))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_cultural_drift_system);

        // Run system over time
        for _ in 0..100 {
            schedule.run(&mut world);
        }

        let tags = world.get::<CulturalTag>(pop_entity).unwrap();
        assert!(
            tags.has_tag("Fringe"),
            "Pop should have acquired the Fringe tag due to distance"
        );
    }

    #[test]
    fn test_fringe_pops_form_feral_outpost_subfaction() {
        let mut world = World::new();

        // Spawn several pops with Fringe tag clustered together
        for i in 0..5 {
            world.spawn((
                Pop,
                GridPosition { x: 80 + i, y: 80 },
                CulturalTag::new(vec!["Fringe".to_string()]),
            ));
        }

        let mut schedule = Schedule::default();
        schedule.add_systems(form_feral_outposts_system);
        schedule.run(&mut world);

        // Check if FeralOutpost faction was formed
        let mut query = world.query_filtered::<&SubFaction, With<FeralOutpost>>();
        assert_eq!(
            query.iter(&world).count(),
            1,
            "A Feral Outpost subfaction should have formed"
        );
    }

    #[test]
    fn test_feral_pops_refuse_relocation() {
        let mut world = World::new();

        let feral_faction = world
            .spawn((SubFaction::new("The Outlanders"), FeralOutpost))
            .id();

        let pop_entity = world
            .spawn((
                Pop,
                GridPosition { x: 80, y: 80 },
                SubFactionMember {
                    faction_entity: feral_faction,
                },
            ))
            .id();

        // Attempt to reassign home zone to the core
        let result = assign_home_zone(&mut world, pop_entity, GridPosition { x: 10, y: 10 });
        assert!(
            result.is_err(),
            "Feral pops should refuse relocation orders to the core"
        );
    }

    #[test]
    fn test_feral_pops_accept_relocation_if_no_faction() {
        let mut world = World::new();

        let pop_entity = world
            .spawn((
                Pop,
                GridPosition { x: 80, y: 80 },
            ))
            .id();

        // Attempt to reassign home zone to the core (should succeed)
        let result = assign_home_zone(&mut world, pop_entity, GridPosition { x: 10, y: 10 });

        assert!(
            result.is_ok(),
            "Non-feral pops should accept relocation orders to the core"
        );
    }

    #[test]
    fn test_feral_pops_accept_relocation_if_not_feral() {
        let mut world = World::new();

        let not_feral_faction = world
            .spawn((SubFaction::new("Normal faction"),))
            .id();

        let pop_entity = world
            .spawn((
                Pop,
                GridPosition { x: 80, y: 80 },
                SubFactionMember { faction_entity: not_feral_faction },
            ))
            .id();

        let result = assign_home_zone(&mut world, pop_entity, GridPosition { x: 10, y: 10 });

        assert!(
            result.is_ok(),
            "Non-feral pops should accept relocation orders to the core"
        );
    }
}
