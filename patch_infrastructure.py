import os
import re

dir_path = 'src/'

def process_file(filepath):
    if not filepath.endswith('.rs'):
        return

    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    modified = False

    replacements = [
        (r'crate::layer1::building\b', 'crate::layer1::infrastructure::building'),
        (r'crate::layer1::structure\b', 'crate::layer1::infrastructure::structure'),
        (r'crate::layer1::housing\b', 'crate::layer1::infrastructure::housing'),
        (r'crate::layer1::room_quality\b', 'crate::layer1::infrastructure::room_quality'),
        (r'crate::layer1::window\b', 'crate::layer1::infrastructure::window'),
        (r'crate::layer1::parasitic_architecture\b', 'crate::layer1::infrastructure::parasitic_architecture'),
        (r'crate::layer1::spontaneous_architecture\b', 'crate::layer1::infrastructure::spontaneous_architecture'),
        (r'crate::layer1::ruins\b', 'crate::layer1::infrastructure::ruins'),
        (r'crate::layer1::construction\b', 'crate::layer1::infrastructure::construction'),
        (r'crate::layer1::building_gate_test\b', 'crate::layer1::infrastructure::building_gate_test'),
        (r'crate::layer1::structure_fragile_tests\b', 'crate::layer1::infrastructure::structure_fragile_tests'),
        (r'crate::layer1::structure_jury_rig_tests\b', 'crate::layer1::infrastructure::structure_jury_rig_tests'),
        (r'crate::layer1::structure_maintenance_tests\b', 'crate::layer1::infrastructure::structure_maintenance_tests'),
        (r'crate::layer1::work_building_tests\b', 'crate::layer1::infrastructure::work_building_tests'),
        (r'crate::layer1::shift_integration_tests\b', 'crate::layer1::infrastructure::shift_integration_tests'),
        (r'super::building\b', 'super::infrastructure::building'),
        (r'super::structure\b', 'super::infrastructure::structure'),
        (r'super::housing\b', 'super::infrastructure::housing'),
        (r'super::room_quality\b', 'super::infrastructure::room_quality'),
        (r'super::window\b', 'super::infrastructure::window'),
        (r'super::parasitic_architecture\b', 'super::infrastructure::parasitic_architecture'),
        (r'super::spontaneous_architecture\b', 'super::infrastructure::spontaneous_architecture'),
        (r'super::ruins\b', 'super::infrastructure::ruins'),
        (r'super::construction\b', 'super::infrastructure::construction'),
    ]

    for pattern, replacement in replacements:
        new_content = re.sub(pattern, replacement, content)
        if new_content != content:
            content = new_content
            modified = True

    if modified:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"Updated {filepath}")

for root, _, files in os.walk(dir_path):
    for file in files:
        process_file(os.path.join(root, file))
