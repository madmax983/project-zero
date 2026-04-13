def run():
    with open('src/layer1/systems/observation.rs', 'r') as f:
        lines = f.readlines()

    # Find where to insert our system. Let's create a new schedule.add_systems block entirely
    # just for our systems, right at the end of the `register` function, before the last `}`.

    # The last line should be `}`
    # Let's find the `clear_input_system` which is usually the last one.

    insert_idx = -1
    for i, line in enumerate(lines):
        if "clear_input_system.after(Layer1SystemSet::Observation)" in line:
            insert_idx = i
            break

    if insert_idx != -1:
        new_block = """
    schedule.add_systems(
        (
            crate::layer1::social::propaganda::process_redactions_system,
            crate::layer1::social::propaganda::apply_propaganda_effects_system,
        )
            .in_set(Layer1SystemSet::Observation),
    );
"""
        lines.insert(insert_idx, new_block)

    with open('src/layer1/systems/observation.rs', 'w') as f:
        f.writelines(lines)

run()
