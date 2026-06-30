import re
import os

files_to_fix = [
    "tests/integration/binge_resources.rs"
]

for file_path in files_to_fix:
    if os.path.exists(file_path):
        with open(file_path, "r") as f:
            content = f.read()

        # Add the missing event resource properly using init_resource
        content = re.sub(
            r'(app\.add_event::<scale::layer1::UnequipFailedEvent>\(\);)',
            r'app.world_mut().init_resource::<bevy_ecs::event::Events<scale::layer1::UnequipFailedEvent>>();',
            content
        )

        if "init_resource::<bevy_ecs::event::Events<scale::layer1::UnequipFailedEvent>>" not in content:
            content = re.sub(
                r'(let mut app = App::new\(\);)',
                r'\1\n        app.world_mut().init_resource::<bevy_ecs::event::Events<scale::layer1::UnequipFailedEvent>>();',
                content
            )

        with open(file_path, "w") as f:
            f.write(content)
