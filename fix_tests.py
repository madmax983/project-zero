with open("src/layer1/fauna/mod.rs", "r") as f:
    content = f.read()

# Replace manual fauna_behavior_system(&mut world) with a schedule
old_test_call = "fauna_behavior_system(&mut world);"
new_test_call = """let mut schedule = Schedule::default();
        schedule.add_systems(fauna_behavior_system);
        schedule.run(&mut world);"""

content = content.replace(old_test_call, new_test_call)

with open("src/layer1/fauna/mod.rs", "w") as f:
    f.write(content)
