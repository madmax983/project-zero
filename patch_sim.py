import re

content = open('src/simulation.rs').read()

content = re.sub(
    r'    if !world.contains_resource::<Events<crate::layer2::phantom::SpawnGhostFleetEvent>>\(\) \{\n        world.init_resource::<Events<crate::layer2::phantom::SpawnGhostFleetEvent>>\(\);\n    \}',
    r'''    if !world.contains_resource::<Events<crate::layer2::phantom::SpawnGhostFleetEvent>>() {
        world.init_resource::<Events<crate::layer2::phantom::SpawnGhostFleetEvent>>();
    }

    if !world.contains_resource::<Events<crate::layer1::nanite_fabrication::ContainmentBreachEvent>>() {
        world.init_resource::<Events<crate::layer1::nanite_fabrication::ContainmentBreachEvent>>();
    }''',
    content
)

content = re.sub(
    r'        world.init_resource::<Events<crate::layer2::phantom::SpawnGhostFleetEvent>>\(\);\n\n        let schedule',
    r'''        world.init_resource::<Events<crate::layer2::phantom::SpawnGhostFleetEvent>>();
        world.init_resource::<Events<crate::layer1::nanite_fabrication::ContainmentBreachEvent>>();

        let schedule''',
    content
)

open('src/simulation.rs', 'w').write(content)
