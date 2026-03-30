import sys

def process_file(filepath):
    with open(filepath, 'r') as f:
        lines = f.readlines()

    out = []

    # We want to remove specific lines and NOT test attributes
    skip_next = False

    to_remove = [
        "pub mod pop;\n",
        "pub use pop::*;\n",
        "pub mod mascot;\n",
        "pub use mascot::*;\n",
        "pub mod vermin;\n",
        "pub use vermin::*;\n",
        "mod vermin_evolution_tests;\n",
        "pub mod fauna_gen;\n",
        "pub mod visitor;\n",
        "pub use visitor::*;\n",
        "pub mod the_visitor;\n",
        "pub use the_visitor::*;\n",
        "pub mod wild_child;\n",
        "pub use wild_child::*;\n",
        "pub mod blob;\n",
        "pub mod drone;\n",
        "pub use drone::*;\n",
        "mod drone_tests;\n",
        "pub mod pop_doppelganger;\n",
        "pub use pop_doppelganger::*;\n",
        "mod fauna_modular_tests;\n"
    ]

    for i, line in enumerate(lines):
        if skip_next:
            skip_next = False
            continue

        if line == "#[cfg(test)]\n" and i + 1 < len(lines):
            next_line = lines[i+1]
            if next_line in to_remove:
                # Skip both the attribute and the line
                skip_next = True
                continue

        if line in to_remove:
            continue

        out.append(line)

    # Insert new entities module at the top
    insert_idx = 0
    for i, line in enumerate(out):
        if line == "pub mod economy;\n":
            insert_idx = i
            break

    out.insert(insert_idx, "pub mod entities;\n")
    out.insert(insert_idx + 1, "pub use entities::*;\n")

    with open(filepath, 'w') as f:
        f.writelines(out)

process_file("src/layer1/mod.rs")
