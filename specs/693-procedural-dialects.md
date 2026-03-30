# Spec 693: Procedural Dialects

## 1. Overview
A community forging its own identity through language. The game logs and chatter generate slang based on colony events. If a "Fire" killed 10 people, "Fire" becomes a curse word. If "Miner Bob" found the motherlode, "Pulling a Bob" means getting lucky. New immigrants arrive speaking "Core Common," while veterans speak a strange, localized creole of mining terms and tragedy.

## 2. Dependencies
- Chronicle/Event system to track significant events
- Social interaction/chatter system
- Lexicon/Grammar generation system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_event_generates_new_slang_term() {
        let mut app = App::new();
        // Setup DialectDictionary resource
        // Trigger a massive fire event
        // Run systems
        // Assert that the DialectDictionary now contains a slang term related to the fire event
    }

    #[test]
    fn test_veteran_pops_use_slang() {
        let mut app = App::new();
        // Setup a Pop with high tenure/veteran status
        // Setup DialectDictionary with a known slang term
        // Generate a chatter dialogue line for the Pop
        // Assert that the dialogue line contains the slang term
    }

    #[test]
    fn test_new_immigrants_do_not_use_slang() {
        let mut app = App::new();
        // Setup a newly spawned immigrant Pop
        // Generate chatter
        // Assert that the dialogue uses standard vocabulary without local slang
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal resource to store the dialect
#[derive(Resource, Default)]
pub struct DialectDictionary {
    pub slang_terms: HashMap<String, String>, // mapping context/trigger to slang string
}

// Minimal system listening for major events and adding entries to DialectDictionary
```

## 5. REFACTOR Phase: Quality & Design
- Integrate smoothly with the existing `lore` generation templates and grammar.
- Provide a clear UI or log interface where the player can see definitions of the newly formed slang.
- Ensure the probability of a Pop using slang scales correctly with their time spent in the colony.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Slang generation is tied dynamically to actual in-game events recorded in the chronicle.

## 7. Technical Guidance
- You will likely need to hook into `AddChronicleEvent` or similar event buses to detect when to mint new slang.
- Consider creating a `PopDialect` component or using `PopMemory` to track if a pop knows the local slang.

## 8. Questions
*Builder: add questions here if spec is unclear.*
