import sys

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    content = content.replace("/// Drone Networks (Spec 116).\n\n", "/// Drone Networks (Spec 116).\n")

    with open(filepath, 'w') as f:
        f.write(content)

process_file("src/layer1/mod.rs")
