import re

with open('src/layer1/building.rs', 'r') as f:
    content = f.read()

# Add AntiGravGenerator to BuildingType enum
content = content.replace(
'''    /// Decorative statue (Beauty +10).
    Statue,''',
'''    /// Decorative statue (Beauty +10).
    Statue,
    /// Generates anti-gravity field but builds up debt.
    AntiGravGenerator,'''
)

content = content.replace(
'''            Self::HoloProjector => false,''',
'''            Self::HoloProjector => false,
            Self::AntiGravGenerator => false,'''
)

content = content.replace(
'''            Self::HoloProjector => "Holo Projector",''',
'''            Self::HoloProjector => "Holo Projector",
            Self::AntiGravGenerator => "Anti-Grav Generator",'''
)

content = content.replace(
'''            Self::BulletinBoard => 'B',''',
'''            Self::BulletinBoard => 'B',
            Self::AntiGravGenerator => 'A','''
)

content = content.replace(
'''            Self::Lander => ColonyResources::zeroed(),''',
'''            Self::Lander => ColonyResources::zeroed(),
            Self::AntiGravGenerator => ColonyResources {
                metal: 100.0,
                knowledge: 50.0,
                ..ColonyResources::zeroed()
            },'''
)

content = content.replace(
'''        BuildingType::PersonalShed
        | BuildingType::PersonalGarden
        | BuildingType::PersonalShrine => {
            // Logic handled by components added in system
        }
    }''',
'''        BuildingType::PersonalShed
        | BuildingType::PersonalGarden
        | BuildingType::PersonalShrine => {
            // Logic handled by components added in system
        }
        BuildingType::AntiGravGenerator => {
            entity.insert((
                crate::layer1::gravitational_debt::AntiGravGenerator::default(),
                crate::layer1::gravitational_debt::GravitationalDebt::default(),
                crate::layer1::energy::PowerConsumer { demand: 50.0, active: false }
            ));
        }
    }'''
)

with open('src/layer1/building.rs', 'w') as f:
    f.write(content)


with open('src/ui/map.rs', 'r') as f:
    ui_content = f.read()

ui_content = ui_content.replace(
'''        BuildingType::HoloProjector => "O",''',
'''        BuildingType::HoloProjector => "O",
        BuildingType::AntiGravGenerator => "⇪",'''
)

ui_content = ui_content.replace(
'''            BuildingType::HoloProjector => Color::Rgb(200, 200, 255),''',
'''            BuildingType::HoloProjector => Color::Rgb(200, 200, 255),
            BuildingType::AntiGravGenerator => Color::Magenta,'''
)

with open('src/ui/map.rs', 'w') as f:
    f.write(ui_content)
