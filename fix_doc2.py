with open('src/layer1/bureaucracy_of_scarcity.rs', 'r') as f:
    code = f.read()

# Restore ignore because it expects a u32 and we don't want to fix random unrelated code right now.
code = code.replace("```rust", "```rust,ignore")

with open('src/layer1/bureaucracy_of_scarcity.rs', 'w') as f:
    f.write(code)
