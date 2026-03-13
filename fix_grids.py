import glob
import re

files = [
    "src/layer1/structural_integrity.rs",
    "src/layer1/void_stare.rs",
    "src/layer1/hum.rs",
    "src/layer1/beauty.rs",
    "src/layer1/geology.rs",
    "src/layer1/loci.rs",
    "src/layer1/nature/water.rs",
    "src/layer1/nature/fertility.rs",
    "src/layer1/nature/erosion.rs",
    "src/layer1/clutter.rs",
    "src/layer1/acoustic.rs",
    "src/layer1/pressure.rs"
]

for fpath in files:
    with open(fpath, "r") as f:
        content = f.read()

    # We want to replace pub fn new(width: usize, height: usize) -> Self { ... vec![...; width * height] ... }
    # with checked bounds.
    # The actual structs are various Grids

    new_content = re.sub(
        r'pub fn new\(width: usize, height: usize\) -> Self \{\s*Self \{\s*width,\s*height,\s*([a-zA-Z_]+):\s*vec!\[([^;]+);\s*width \* height\],\s*\}\s*\}',
        r'pub fn new(width: usize, height: usize) -> Self {\n        let size = width\n            .checked_mul(height)\n            .expect("Grid size overflow or too large");\n        assert!(size <= 10_000_000, "Grid size overflow or too large");\n\n        Self {\n            width,\n            height,\n            \1: vec![\2; size],\n        }\n    }',
        content
    )

    if new_content != content:
        with open(fpath, "w") as f:
            f.write(new_content)
        print(f"Fixed {fpath}")

# Handle lighting.rs which has a cast
with open("src/layer1/lighting.rs", "r") as f:
    content = f.read()

new_content = re.sub(
    r'pub fn new\(width: u32, height: u32\) -> Self \{\s*Self \{\s*width,\s*height,\s*tiles:\s*vec!\[0\.0;\s*\(width \* height\) as usize\],\s*\}\s*\}',
    r'pub fn new(width: u32, height: u32) -> Self {\n        let size = (width as usize)\n            .checked_mul(height as usize)\n            .expect("Grid size overflow or too large");\n        assert!(size <= 10_000_000, "Grid size overflow or too large");\n\n        Self {\n            width,\n            height,\n            tiles: vec![0.0; size],\n        }\n    }',
    content
)
if new_content != content:
    with open("src/layer1/lighting.rs", "w") as f:
        f.write(new_content)
    print(f"Fixed src/layer1/lighting.rs")
