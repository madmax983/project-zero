with open("src/layer1/administration/bureaucracy_of_sleep.rs", "r") as f:
    content = f.read()

content = content.replace(
    "let mut schedule = Schedule::default();\n/// schedule.add_systems(process_sleep_deprivation_system);",
    "world.insert_resource(bevy::time::Time::<()>::default());\n/// let mut schedule = Schedule::default();\n/// schedule.add_systems(process_sleep_deprivation_system);"
)

with open("src/layer1/administration/bureaucracy_of_sleep.rs", "w") as f:
    f.write(content)
