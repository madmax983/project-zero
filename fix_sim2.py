import re

with open('src/lib.rs', 'r') as f:
    content = f.read()

content = content.replace(
'''    app.add_event::<GridOverloadEvent>();''',
'''    app.add_event::<GridOverloadEvent>();
    app.add_event::<crate::layer1::gravitational_debt::DebtReleaseEvent>();'''
)

with open('src/lib.rs', 'w') as f:
    f.write(content)


with open('src/layer1/building.rs', 'r') as f:
    content = f.read()

content = content.replace(
'''        // Type cycling
        app.world_mut().send_event(GameKeyEvent {
            key: GameKeyCode::Tab,
        });
        app.update();

        let mode = app.world().resource::<BuildMode>();
        assert_eq!(mode.selected, BuildingType::AntiGravGenerator);''',
'''        // Type cycling
        app.world_mut().send_event(GameKeyEvent {
            key: GameKeyCode::Tab,
        });
        app.update();

        let mode = app.world().resource::<BuildMode>();
        assert_eq!(mode.selected, BuildingType::AntiGravGenerator);

        app.world_mut().send_event(GameKeyEvent {
            key: GameKeyCode::Tab,
        });
        app.update();

        let mode = app.world().resource::<BuildMode>();
        assert_eq!(mode.selected, BuildingType::Hospital);'''
)

with open('src/layer1/building.rs', 'w') as f:
    f.write(content)
