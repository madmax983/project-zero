import sys

def main():
    with open("README.md", "r") as f:
        content = f.read()

    content = content.replace("```rust,ignore\n// In Cargo.toml", "```rust,compile_fail\n// In Cargo.toml")

    with open("README.md", "w") as f:
        f.write(content)

if __name__ == "__main__":
    main()
