import sys

def main():
    with open('src/layer1/core/integration.rs', 'r') as f:
        content = f.read()

    new_code = """

// Predatory Weather Integration (969)
pub fn predatory_weather_emission_bridge_system(
    mut commands: bevy_ecs::prelude::Commands,
    power_sources: bevy_ecs::prelude::Query<&crate::layer1::energy::PowerSource>,
    heat_sources: bevy_ecs::prelude::Query<&crate::layer1::nature::temperature::HeatSource>,
    mut targets: bevy_ecs::prelude::Query<&mut crate::layer2::weather::AggroTarget>,
) {
    let mut total_energy = 0.0;
    for power in power_sources.iter() {
        if power.active {
            total_energy += power.output;
        }
    }

    let mut total_heat = 0.0;
    for heat in heat_sources.iter() {
        total_heat += heat.output;
    }

    if let Some(mut target) = targets.iter_mut().next() {
        target.energy_emission = total_energy;
        target.heat_signature = total_heat;
    } else {
        commands.spawn(crate::layer2::weather::AggroTarget {
            position: bevy::math::Vec2::new(0.0, 0.0),
            energy_emission: total_energy,
            heat_signature: total_heat,
        });
    }
}

pub fn predatory_weather_impact_bridge_system(
    mut events: bevy_ecs::prelude::EventReader<crate::layer2::weather::StormImpactEvent>,
    mut structures: bevy_ecs::prelude::Query<&mut crate::layer1::architecture::structure::Structure>,
    mut chronicle_events: bevy_ecs::prelude::EventWriter<crate::layer1::chronicle::AddChronicleEvent>,
) {
    for event in events.read() {
        for mut structure in structures.iter_mut() {
            structure.current_hp -= event.damage;
        }

        chronicle_events.send(crate::layer1::chronicle::AddChronicleEvent {
            importance: crate::layer1::chronicle::EventImportance::Major,
            text: format!("A massive planetary storm impacted the colony, dealing {} damage to our infrastructure.", event.damage),
        });
    }
}
"""
    if "predatory_weather_emission_bridge_system" not in content:
        content += new_code
        with open('src/layer1/core/integration.rs', 'w') as f:
            f.write(content)
        print("Patched src/layer1/core/integration.rs")
    else:
        print("Already patched")

if __name__ == "__main__":
    main()
