#!/usr/bin/perl
use strict;
use warnings;

my $file = 'src/layer1/building.rs';
open my $in, '<', $file or die "Cannot open $file: $!";
my $content = do { local $/; <$in> };
close $in;

my $new_configure_refining_buildings = <<'REPLACE_END';
fn configure_basic_refiners(entity: &mut EntityWorldMut, building_type: BuildingType) {
    match building_type {
        BuildingType::Smokehouse => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 4.0,
                    intensity: 0.5,
                    color: (200, 200, 200), // Smoky white/grey
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::LumberMill => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 4.0,
                    intensity: 0.5,
                    color: (200, 180, 100), // Dim Wood light
                },
                SeismicSource {
                    intensity: 0.5,
                    radius: 3.0,
                },
                NoiseSource {
                    radius: 6.0,
                    intensity: 0.8,
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::StoneMason | BuildingType::Weaver | BuildingType::Tailor => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                ShiftSchedule::default(),
            ));
        }
        _ => {}
    }
}

fn configure_advanced_refiners(entity: &mut EntityWorldMut, building_type: BuildingType) {
    match building_type {
        BuildingType::Smelter => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 5.0,
                    intensity: 0.9,
                    color: (255, 50, 0), // Red/Fire
                },
                SeismicSource {
                    intensity: 0.5,
                    radius: 3.0,
                },
                NoiseSource {
                    radius: 8.0,
                    intensity: 1.0,
                },
                PowerConsumer {
                    demand: 5.0,
                    active: false,
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::Smithy => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                LightSource {
                    is_outdoor: true,
                    radius: 4.0,
                    intensity: 0.7,
                    color: (255, 100, 0), // Orange/Fire
                },
                SeismicSource {
                    intensity: 0.5,
                    radius: 3.0,
                },
                NoiseSource {
                    radius: 6.0,
                    intensity: 0.9,
                },
                PowerConsumer {
                    demand: 2.0,
                    active: false,
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::Refinery => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 20.0, // Slower process
                },
                LightSource {
                    is_outdoor: true,
                    radius: 6.0,
                    intensity: 0.8,
                    color: (100, 200, 255), // Chemical blue
                },
                SeismicSource {
                    intensity: 0.8,
                    radius: 6.0,
                },
                NoiseSource {
                    radius: 10.0,
                    intensity: 1.0,
                },
                ShiftSchedule::default(),
            ));
        }
        BuildingType::AncientFabricator => {
            entity.insert((
                RefiningProgress {
                    current: 0.0,
                    max: 1.0,
                },
                AncientStructure,
                MachineSpirit::default(),
                LightSource {
                    is_outdoor: true,
                    radius: 6.0,
                    intensity: 0.8,
                    color: (0, 255, 255), // Cyan
                },
                ShiftSchedule::default(),
            ));
            if let Some(mut structure) = entity.get_mut::<crate::layer1::structure::Structure>() {
                structure.max_hp = 1000.0;
                structure.current_hp = 1000.0;
            }
        }
        _ => {}
    }
}

fn configure_refining_buildings(entity: &mut EntityWorldMut, building_type: BuildingType) {
    configure_basic_refiners(entity, building_type);
    configure_advanced_refiners(entity, building_type);
}
REPLACE_END

$content =~ s/\#\[allow\(clippy::too_many_lines\)\]\nfn configure_refining_buildings.*?\}\n\}//ms
  or die "Could not find configure_refining_buildings to replace";

$content =~ s/(fn configure_production.*)/$new_configure_refining_buildings\n\n$1/ms;

open my $out, '>', $file or die "Cannot open $file for writing: $!";
print $out $content;
close $out;
