use anyhow::{Context, Result};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Context for story generation, holding values for slots.
#[derive(Debug, Default, Clone)]
pub struct NarrativeContext {
    slots: HashMap<String, String>,
}

impl NarrativeContext {
    /// Create a new context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a value for a slot (e.g., "`CIV_NAME`" -> "The Empire").
    pub fn insert(&mut self, key: &str, value: &str) {
        self.slots.insert(key.to_string(), value.to_string());
    }

    /// Get a value for a slot.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&String> {
        self.slots.get(key)
    }
}

/// A narrative template with multiple possible patterns.
#[derive(Debug, Clone)]
pub struct Template {
    /// Unique identifier for the template.
    pub id: String,
    /// List of pattern strings with slots.
    pub patterns: Vec<String>,
}

/// A collection of fragments for a specific type.
#[derive(Debug, Clone)]
pub struct FragmentType {
    /// Unique identifier for the fragment type (e.g., "`CIV_EPITHET`").
    pub id: String,
    /// List of possible text values.
    pub options: Vec<String>,
}

/// The main generator system.
#[derive(Debug, Default)]
pub struct NarrativeGenerator {
    templates: HashMap<String, Template>,
    fragments: HashMap<String, FragmentType>,
}

impl NarrativeGenerator {
    /// Load templates and fragments from the given directory.
    ///
    /// # Errors
    /// Returns an error if reading the template or fragment files fails.
    pub fn load_from_files<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();

        let templates_path = path.join("TEMPLATES.md");
        if templates_path.exists() {
            let content = fs::read_to_string(&templates_path)
                .with_context(|| format!("Failed to read {}", templates_path.display()))?;
            self.parse_templates(&content);
        }

        let fragments_path = path.join("FRAGMENTS.md");
        if fragments_path.exists() {
            let content = fs::read_to_string(&fragments_path)
                .with_context(|| format!("Failed to read {}", fragments_path.display()))?;
            self.parse_fragments(&content);
        }

        Ok(())
    }

    /// Add a template programmatically.
    pub fn add_template(&mut self, id: String, patterns: Vec<String>) {
        self.templates.insert(id.clone(), Template { id, patterns });
    }

    /// Add a fragment type programmatically.
    pub fn add_fragment(&mut self, id: String, options: Vec<String>) {
        self.fragments
            .insert(id.clone(), FragmentType { id, options });
    }

    /// Parse templates from Markdown content.
    pub fn parse_templates(&mut self, content: &str) {
        let mut current_id: Option<String> = None;
        let mut current_patterns: Vec<String> = Vec::new();
        let mut capturing_code_block = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Detect Template Header: "### TEMPLATE_NAME"
            if let Some(id_part) = trimmed.strip_prefix("### ") {
                // If we were parsing a previous template, save it
                if let Some(id) = current_id.take()
                    && !current_patterns.is_empty()
                {
                    self.templates.insert(
                        id.clone(),
                        Template {
                            id,
                            patterns: current_patterns.clone(),
                        },
                    );
                }

                // Start new template
                if !id_part.contains("Templates") && !id_part.contains("Fragments") {
                    current_id = Some(id_part.trim().to_string());
                    current_patterns = Vec::new();
                    capturing_code_block = false;
                }
                continue;
            }

            // Detect Code Block for patterns
            if trimmed.starts_with("```") {
                capturing_code_block = !capturing_code_block;
                continue;
            }

            // Capture patterns inside code blocks
            if capturing_code_block && !trimmed.is_empty() {
                // Remove quotes if present
                let pattern =
                    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
                        &trimmed[1..trimmed.len() - 1]
                    } else {
                        trimmed
                    };

                // Ignore empty lines or comments
                if !pattern.is_empty() && !pattern.starts_with("//") {
                    current_patterns.push(pattern.to_string());
                }
            }
        }

        // Save the last one
        if let Some(id) = current_id
            && !current_patterns.is_empty()
        {
            self.templates.insert(
                id.clone(),
                Template {
                    id,
                    patterns: current_patterns,
                },
            );
        }
    }

    /// Parse fragments from Markdown content.
    pub fn parse_fragments(&mut self, content: &str) {
        let mut current_id: Option<String> = None;
        let mut current_options: Vec<String> = Vec::new();
        let mut capturing_code_block = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Detect Fragment Header: "### [FRAGMENT_NAME]"
            if let Some(header) = trimmed.strip_prefix("### ") {
                if let Some(id) = current_id.take()
                    && !current_options.is_empty()
                {
                    self.fragments.insert(
                        id.clone(),
                        FragmentType {
                            id,
                            options: current_options.clone(),
                        },
                    );
                }

                // Extract name between brackets
                if let Some(start) = header.find('[')
                    && let Some(end) = header.find(']')
                {
                    current_id = Some(header[start + 1..end].to_string());
                    current_options = Vec::new();
                    capturing_code_block = false;
                }
                continue;
            }

            // Detect Code Block
            if trimmed.starts_with("```") {
                capturing_code_block = !capturing_code_block;
                continue;
            }

            // Capture options
            if capturing_code_block && !trimmed.is_empty() {
                current_options.push(trimmed.to_string());
            } else if !capturing_code_block && trimmed.starts_with("- ") {
                current_options.push(trimmed[2..].to_string());
            }
        }

        // Save last
        if let Some(id) = current_id
            && !current_options.is_empty()
        {
            self.fragments.insert(
                id.clone(),
                FragmentType {
                    id,
                    options: current_options,
                },
            );
        }
    }

    /// Return the number of loaded templates.
    #[must_use]
    pub fn template_count(&self) -> usize {
        self.templates.len()
    }

    /// Return the number of loaded fragment types.
    #[must_use]
    pub fn fragment_count(&self) -> usize {
        self.fragments.len()
    }

    /// Get a random option from a fragment type.
    #[must_use]
    pub fn get_random_fragment(&self, fragment_id: &str) -> Option<&String> {
        self.fragments
            .get(fragment_id)?
            .options
            .choose(&mut rand::thread_rng())
    }

    /// Generate a story string from a template ID and context.
    ///
    /// # Errors
    /// Returns an error if the template ID is not found or if the template has no patterns.
    pub fn generate(&self, template_id: &str, context: &NarrativeContext) -> Result<String> {
        let template = self
            .templates
            .get(template_id)
            .ok_or_else(|| anyhow::anyhow!("Template not found: {template_id}"))?;

        // Pick a random pattern
        let pattern = template
            .patterns
            .choose(&mut rand::thread_rng())
            .ok_or_else(|| anyhow::anyhow!("Template {template_id} has no patterns"))?;

        // We'll iterate through the string and build the output
        let mut output = String::new();
        let mut char_iter = pattern.chars().peekable();

        while let Some(c) = char_iter.next() {
            if c == '[' {
                // Possible slot start
                let mut slot_name = String::new();
                let mut closed = false;

                // Peek ahead to capture slot name
                while let Some(&next_c) = char_iter.peek() {
                    char_iter.next(); // Consume
                    if next_c == ']' {
                        closed = true;
                        break;
                    }
                    slot_name.push(next_c);
                }

                if closed {
                    // Check for optional marker '?' at end of slot name
                    let is_optional = slot_name.ends_with('?');
                    let key = if is_optional {
                        &slot_name[0..slot_name.len() - 1]
                    } else {
                        &slot_name
                    };

                    // Resolve slot
                    if let Some(val) = context.get(key) {
                        output.push_str(val);
                    } else if let Some(fragment) = self.fragments.get(key) {
                        // Pick random fragment
                        if let Some(option) = fragment.options.choose(&mut rand::thread_rng()) {
                            output.push_str(option);
                        } else {
                            output.push_str("[MISSING_FRAGMENT_OPTIONS:");
                            output.push_str(key);
                            output.push(']');
                        }
                    } else {
                        // Not found in context or fragments
                        if !is_optional {
                            output.push('[');
                            output.push_str(&slot_name);
                            output.push(']');
                        }
                    }
                } else {
                    // Malformed bracket, just push what we collected
                    output.push('[');
                    output.push_str(&slot_name);
                }
            } else {
                output.push(c);
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_insert_get() {
        let mut ctx = NarrativeContext::new();
        ctx.insert("KEY", "VALUE");
        assert_eq!(ctx.get("KEY").unwrap(), "VALUE");
    }

    #[test]
    fn test_template_parsing() {
        let content = r#"
### TEST_TEMPLATE
**Slots:** [SLOT]
```
"Pattern with [SLOT]"
"Another [SLOT]"
```
"#;
        let mut generator = NarrativeGenerator::default();
        generator.parse_templates(content);

        assert_eq!(generator.template_count(), 1);
        let tmpl = generator.templates.get("TEST_TEMPLATE").unwrap();
        assert_eq!(tmpl.patterns.len(), 2);
        assert_eq!(tmpl.patterns[0], "Pattern with [SLOT]");
    }

    #[test]
    fn test_fragment_parsing() {
        let content = r"
### [TEST_FRAGMENT]
- option1
- option2
";
        let mut generator = NarrativeGenerator::default();
        generator.parse_fragments(content);

        assert_eq!(generator.fragment_count(), 1);
        let frag = generator.fragments.get("TEST_FRAGMENT").unwrap();
        assert_eq!(frag.options.len(), 2);
        assert_eq!(frag.options[0], "option1");
    }

    #[test]
    fn test_generation_simple() {
        let mut generator = NarrativeGenerator::default();
        generator.templates.insert(
            "SIMPLE".to_string(),
            Template {
                id: "SIMPLE".to_string(),
                patterns: vec!["Hello [NAME]!".to_string()],
            },
        );

        let mut ctx = NarrativeContext::new();
        ctx.insert("NAME", "World");

        let result = generator.generate("SIMPLE", &ctx).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_generation_fragment_fallback() {
        let mut generator = NarrativeGenerator::default();
        generator.add_template("FRAG".to_string(), vec!["Value: [KEY]".to_string()]);
        generator.add_fragment("KEY".to_string(), vec!["FragmentValue".to_string()]);

        let ctx = NarrativeContext::new();
        // Context empty, should use fragment
        let result = generator.generate("FRAG", &ctx).unwrap();
        assert_eq!(result, "Value: FragmentValue");
    }

    #[test]
    fn test_panic_on_single_quote() {
        let content = r#"
### TEST_PANIC
```
"
```
"#;
        let mut generator = NarrativeGenerator::default();
        // Should not panic
        generator.parse_templates(content);
        // " starts with " and ends with " but len is 1. So else branch -> trimmed ("") -> pattern ("").
        // It captures " as a pattern.
        assert_eq!(generator.template_count(), 1);
        let tmpl = generator.templates.get("TEST_PANIC").unwrap();
        assert_eq!(tmpl.patterns[0], "\"");
    }
}
