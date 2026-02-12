import os
import glob

def main():
    files = glob.glob('src/layer1/actions/*.rs')
    for filepath in files:
        with open(filepath, 'r') as f:
            content = f.read()

        # Replace utility_ai with utility_types for imports
        new_content = content.replace('crate::layer1::utility_ai', 'crate::layer1::utility_types')

        if new_content != content:
            with open(filepath, 'w') as f:
                f.write(new_content)
            print(f"Updated {filepath}")

if __name__ == "__main__":
    main()
