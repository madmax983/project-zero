cat << 'INNER_EOF' >> src/shared/narrative.rs

impl NarrativeError {
    /// Returns a helpful message suggesting how to resolve the error.
    pub fn help(&self) -> &'static str {
        match self {
            Self::DirectoryNotFound(_) | Self::NoLoreFiles(_) | Self::IoError(_, _) => {
                "Check Lore Directory. Verify the folder path exists and contains markdown files."
            }
            Self::MissingFragmentOptions(_) => {
                "Check Fragment Options. Verify the fragment options in your FRAGMENTS.md are not empty."
            }
            Self::TemplateNotFound(_) | Self::NoPatternsForTemplate(_) => {
                "Check Template ID. Verify the name exists in your TEMPLATES.md."
            }
            Self::MissingContext(_) => {
                "Check Context variables. Verify that you are calling `NarrativeContext::insert` for the missing variable in your Rust code."
            }
        }
    }
}
INNER_EOF
