import re

files_to_patch = {
    'src/ui/input.rs': [
        "let mut world = setup_world_for_test();",
        "let mut world = setup_world();",
        "let mut app = App::new();\n        app.world_mut().insert_resource(world);"
    ],
    'src/ui/shell/plugins/mod.rs': [
        "let mut world = setup_world();"
    ],
    'src/ui/shell/runtime.rs': [
        "let mut world = setup_world();",
        "let shared_world = Rc::new(RefCell::new(setup_world()));"
    ],
    'src/ui/tech.rs': [
        "let mut world = setup_world();"
    ]
}

def apply_patch(filepath):
    try:
        with open(filepath, 'r') as f:
            content = f.read()

        # Custom replacements
        if filepath == 'src/ui/input.rs':
            content = content.replace("let mut world = setup_world_for_test();", "let mut world = setup_world_for_test();\n        crate::ui::setup_ui_resources(&mut world);")
            content = content.replace("let mut world = setup_world();", "let mut world = setup_world();\n        crate::ui::setup_ui_resources(&mut world);")

        if filepath == 'src/ui/shell/runtime.rs':
            content = content.replace("let shared_world = Rc::new(RefCell::new(setup_world()));", "let mut raw_world = setup_world();\n        crate::ui::setup_ui_resources(&mut raw_world);\n        let shared_world = Rc::new(RefCell::new(raw_world));")

        if filepath == 'src/ui/shell/plugins/mod.rs':
            content = content.replace("let mut world = setup_world();", "let mut world = setup_world();\n        crate::ui::setup_ui_resources(&mut world);")
            content = content.replace("let shared_world = Rc::new(RefCell::new(setup_world()));", "let mut raw_world = setup_world();\n        crate::ui::setup_ui_resources(&mut raw_world);\n        let shared_world = Rc::new(RefCell::new(raw_world));")

        if filepath == 'src/ui/tech.rs':
             content = content.replace("let mut world = setup_world();", "let mut world = setup_world();\n        crate::ui::setup_ui_resources(&mut world);")
             content = content.replace("let shared_world = Rc::new(RefCell::new(setup_world()));", "let mut raw_world = setup_world();\n        crate::ui::setup_ui_resources(&mut raw_world);\n        let shared_world = Rc::new(RefCell::new(raw_world));")

        with open(filepath, 'w') as f:
            f.write(content)
        print(f"Patched {filepath}")
    except FileNotFoundError:
        print(f"File {filepath} not found")

for file in files_to_patch.keys():
    apply_patch(file)
