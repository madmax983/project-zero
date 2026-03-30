with open("src/experimental/architectural_palimpsest.rs", "r") as f:
    content = f.read()

import re

# We will change the test to use an App so the system state (Local) is preserved.
new_test = """
    #[test]
    fn test_building_demolition_creates_shadow() {
        use bevy_app::{App, Update};
        let mut app = App::new();
        app.add_systems(Update, detect_building_demolitions);

        // Spawn a building that will generate squalor (e.g. Tavern)
        let building_entity = app.world_mut()
            .spawn((
                GridPosition { x: 5, y: 5 },
                Building {
                    building_type: BuildingType::Tavern,
                },
            ))
            .id();

        // Run detect_building_demolitions once to populate the Local tracker
        app.update();

        assert_eq!(app.world_mut().query::<&BuildingShadow>().iter(app.world_mut()).count(), 0);

        // Despawn the building (simulating demolition)
        app.world_mut().despawn(building_entity);

        // Run system again
        app.update();

        // Shadow should be created
        let mut shadow_q = app.world_mut().query::<(&GridPosition, &BuildingShadow)>();
        let mut shadow_count = 0;
        for (pos, shadow) in shadow_q.iter(app.world()) {
            assert_eq!(pos.x, 5);
            assert_eq!(pos.y, 5);
            assert!(shadow.squalor_memory > 0.0);
            shadow_count += 1;
        }
        assert_eq!(shadow_count, 1);
    }
"""

content = re.sub(
    r"    #\[test\]\n    fn test_building_demolition_creates_shadow\(\) \{.*?(?=    #\[test\])",
    new_test + "\n",
    content,
    flags=re.DOTALL
)

with open("src/experimental/architectural_palimpsest.rs", "w") as f:
    f.write(content)
