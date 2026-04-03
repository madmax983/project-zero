#!/usr/bin/perl
use strict;
use warnings;

my $file = 'src/layer1/science.rs';
open my $in, '<', $file or die "Cannot open $file: $!";
my $content = do { local $/; <$in> };
close $in;

my $new_process_scan_system = <<'REPLACE_END';
#[allow(clippy::type_complexity)]
fn collect_scanners(
    world: &mut World,
) -> Vec<(Entity, Entity)> {
    let striking_factions: std::collections::HashSet<crate::layer1::factions::FactionId> = world
        .get_resource::<crate::layer1::factions::Factions>()
        .map(|f| {
            f.map
                .iter()
                .filter(|(_, d)| d.state == crate::layer1::factions::FactionState::Striking)
                .map(|(id, _)| *id)
                .collect()
        })
        .unwrap_or_default();

    let mut scanners = Vec::new();
    let mut query = world.query_filtered::<(
        Entity,
        &MovementTarget,
        Option<&crate::layer1::factions::FactionMember>,
    ), With<AtTarget>>();

    for (entity, mt, faction_member) in query.iter(world) {
        if mt.for_action == ActionType::Explore {
            let is_striking = faction_member
                .and_then(|m| m.faction_id)
                .is_some_and(|fid| striking_factions.contains(&fid));

            if !is_striking {
                scanners.push((entity, mt.target_entity));
            }
        }
    }
    scanners
}

fn complete_anomaly_scan(world: &mut World, pop_entity: Entity, anomaly_entity: Entity) {
    let (anomaly_type, reward) = if let Some(anomaly) = world.get::<Anomaly>(anomaly_entity) {
        (anomaly.anomaly_type, anomaly.reward_amount)
    } else {
        cleanup_pop_explore_state(world, pop_entity);
        return;
    };

    let pos = world.get::<GridPosition>(anomaly_entity).copied();

    match anomaly_type {
        AnomalyType::Ruins => {
            world
                .resource_mut::<ColonyResources>()
                .add_knowledge(reward);
            world.resource_mut::<MessageLog>().add(format!(
                "Discovery: Scanned ruins yielded {reward:.0} Knowledge."
            ));
        }
        AnomalyType::StrangeFlora => {
            world.resource_mut::<ColonyResources>().add_food(reward);
            world.resource_mut::<MessageLog>().add(format!(
                "Discovery: Strange flora yielded {reward:.0} Food."
            ));
            if let Some(pos) = pos {
                world.spawn((
                    ResourceItem {
                        resource_type: ResourceType::Food,
                        amount: 10.0,
                    },
                    pos,
                ));
            }
        }
        AnomalyType::Geode => {
            let mut rng = rand::thread_rng();
            if rng.gen_bool(0.5) {
                world.resource_mut::<ColonyResources>().add_stone(reward);
                world
                    .resource_mut::<MessageLog>()
                    .add(format!("Discovery: Geode yielded {reward:.0} Stone."));
                if let Some(pos) = pos {
                    world.spawn((
                        ResourceItem {
                            resource_type: ResourceType::Stone,
                            amount: 10.0,
                        },
                        pos,
                    ));
                }
            } else {
                world.resource_mut::<ColonyResources>().add_ore(reward);
                world
                    .resource_mut::<MessageLog>()
                    .add(format!("Discovery: Geode yielded {reward:.0} Ore."));
                if let Some(pos) = pos {
                    world.spawn((
                        ResourceItem {
                            resource_type: ResourceType::Ore,
                            amount: 10.0,
                        },
                        pos,
                    ));
                }
            }
        }
    }

    world.despawn(anomaly_entity);
    cleanup_pop_explore_state(world, pop_entity);
}

/// Processes the scanning action for pops.
///
/// # Panics
///
/// Panics if the anomaly entity exists but lacks the `Anomaly` component.
pub fn process_scan_system(world: &mut World) {
    let scanners = collect_scanners(world);

    for (pop_entity, anomaly_entity) in scanners {
        let scan_amount = 1.0;

        if world.get_entity(anomaly_entity).is_err() {
            cleanup_pop_explore_state(world, pop_entity);
            continue;
        }

        let is_complete = if let Some(mut progress) = world.get_mut::<ScanProgress>(anomaly_entity) {
            progress.current += scan_amount;
            progress.is_complete()
        } else {
            cleanup_pop_explore_state(world, pop_entity);
            continue;
        };

        if is_complete {
            complete_anomaly_scan(world, pop_entity, anomaly_entity);
        }
    }
}
REPLACE_END

$content =~ s/^\Q\/\/\/ Processes the scanning action for pops.\E.*?(?=\nfn cleanup_pop_explore_state)/$new_process_scan_system/ms
  or die "Could not find process_scan_system to replace";

open my $out, '>', $file or die "Cannot open $file for writing: $!";
print $out $content;
close $out;
