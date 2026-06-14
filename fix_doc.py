with open('src/layer1/bureaucracy_of_scarcity.rs', 'r') as f:
    code = f.read()

# Replace available_jobs with available_bureaucrat_jobs in doc test
code = code.replace("available_jobs: vec![]", "available_bureaucrat_jobs: vec![]")

# Remove ignore from doc test since it should work now
code = code.replace("```rust,ignore", "```rust")

with open('src/layer1/bureaucracy_of_scarcity.rs', 'w') as f:
    f.write(code)
