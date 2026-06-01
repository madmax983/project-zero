# Bolt Learnings

## System Parameter Local Optimization
**Learning:** Instantiating new HashMaps or Vecs inside a hot system causes frame-by-frame heap allocations which can degrade performance. Using `Local<T>` stores the data across frames so its capacity can be reused by simply calling `.clear()` each frame.
**Action:** Always check systems for `HashMap::new()` or `Vec::new()` and consider extracting them into a `Local<T>` system parameter.

## Documentation Comments on Parameters
**Learning:** Rust does not allow `///` (doc comments) on function parameters. This will cause a compilation error.
**Action:** When commenting inside a function signature to explain an optimization to a parameter, use standard comments (`//`) instead of doc comments (`///`).
